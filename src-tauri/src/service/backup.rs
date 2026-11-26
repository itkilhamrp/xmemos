// Copyright (C) 2026 Xmemos Contributors
//
// This file is part of Xmemos.
//
// Xmemos is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Xmemos is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with Xmemos. If not, see https://www.gnu.org/licenses/.
//
// SPDX-License-Identifier: AGPL-3.0-or-later

use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

use crate::error::{AppError, AppResult};
use crate::service::{media, search};
use crate::state::AppState;
use crate::store::db::{
    memos_schema::{adapt_imported_memos_db, validate_imported_memos_db},
    open_memos_db,
};

const SQLITE_MAGIC: &[u8; 16] = b"SQLite format 3\0";
const MAX_DECOMPRESSION_BYTES: u64 = 2 * 1024 * 1024 * 1024; // 2 GiB safety ceiling

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestFileEntry {
    pub path: String,
    pub size: u64,
    pub sha256: String,
}

#[derive(Debug, Clone)]
pub enum BackupScope {
    AllUsers,
    ActiveUser { user_id: i64, username: String },
}

impl BackupScope {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::AllUsers => "ALL_USERS",
            Self::ActiveUser { .. } => "ACTIVE_USER",
        }
    }

    pub fn filename_tag(&self) -> String {
        match self {
            Self::AllUsers => "all".to_string(),
            Self::ActiveUser { username, .. } => {
                let clean: String = username
                    .chars()
                    .map(|c| {
                        if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                            c
                        } else {
                            '_'
                        }
                    })
                    .take(24)
                    .collect();
                if clean.is_empty() {
                    "user".to_string()
                } else {
                    format!("user-{clean}")
                }
            }
        }
    }
}

fn default_scope_str() -> String {
    "ALL_USERS".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleManifest {
    pub format: String,
    pub format_version: u32,
    pub app_version: String,
    pub exported_at_utc: String,
    #[serde(default = "default_scope_str")]
    pub scope: String,
    pub memos_schema_compatible: bool,
    pub db_filename: String,
    pub db_sha256: String,
    pub media_count: usize,
    pub media_total_bytes: u64,
    pub files: Vec<ManifestFileEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportResult {
    pub mode: String,
    pub scope: String,
    pub filename: String,
    pub path: String,
    pub size_bytes: u64,
    pub sha256: String,
    pub media_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportPreview {
    pub source_path: String,
    pub detected_format: String,
    pub user_count: i64,
    pub memo_count: i64,
    pub resource_count: i64,
    pub media_file_count: usize,
    pub archive_size_bytes: u64,
    pub checksum_verified: bool,
    pub compatibility_notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResult {
    pub user_count: i64,
    pub memo_count: i64,
    pub resource_count: i64,
    pub restored_media_files: usize,
    pub fts_indexed_memos: usize,
    pub compatibility_notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupHistoryEntry {
    pub id: i64,
    pub mode: String,
    pub filename: String,
    pub path: String,
    pub size_bytes: i64,
    pub sha256: String,
    pub created_ts: i64,
}

fn sha256_file(path: &Path) -> AppResult<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 65536];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex::encode(hasher.finalize()))
}

fn is_already_compressed_media(path: &Path) -> bool {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_ascii_lowercase())
        .unwrap_or_default();
    matches!(
        ext.as_str(),
        "jpg" | "jpeg" | "webp" | "png" | "gif" | "mp4" | "m4a" | "mp3" | "webm" | "ogg" | "opus" | "heic" | "avif" | "zip"
    )
}

fn collect_relative_assets(root: &Path, current: &Path, acc: &mut Vec<(String, PathBuf)>) {
    if let Ok(entries) = fs::read_dir(current) {
        let mut sorted: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
        sorted.sort();
        for p in sorted {
            if p.is_dir() {
                collect_relative_assets(root, &p, acc);
            } else if p.is_file() {
                if p.file_name().and_then(|s| s.to_str()) == Some(".nomedia") {
                    continue;
                }
                if let Ok(rel) = p.strip_prefix(root) {
                    let rel_posix = format!("assets/{}", rel.to_string_lossy().replace('\\', "/"));
                    acc.push((rel_posix, p));
                }
            }
        }
    }
}

/// Produces a clean, WAL-checkpointed, VACUUMed, single-file `memos_prod.db` snapshot.
/// When `BackupScope::ActiveUser` is requested, safely prunes only the exported copy (never touching the live DB)
/// so it remains a 100% valid standalone Official Memos SQLite database for that single user.
fn create_clean_db_snapshot(
    state: &AppState,
    dest_path: &Path,
    scope: &BackupScope,
) -> AppResult<()> {
    if dest_path.exists() {
        fs::remove_file(dest_path)?;
    }
    let escaped = dest_path.to_string_lossy().replace('\'', "''");
    state.with_memos_db(|conn| {
        conn.execute_batch("PRAGMA wal_checkpoint(TRUNCATE);")?;
        conn.execute_batch(&format!("VACUUM INTO '{escaped}';"))?;
        Ok(())
    })?;

    let mut export_conn = Connection::open(dest_path)?;
    if let BackupScope::ActiveUser { user_id, .. } = scope {
        let tx = export_conn.transaction()?;
        tx.execute("DELETE FROM user WHERE id != ?1", params![user_id])?;
        tx.execute(
            "UPDATE user SET role = 'HOST' WHERE id = ?1 AND role != 'HOST'",
            params![user_id],
        )?;
        tx.execute("DELETE FROM user_setting WHERE user_id != ?1", params![user_id])?;
        tx.execute("DELETE FROM memo WHERE creator_id != ?1", params![user_id])?;
        tx.execute(
            "DELETE FROM memo_relation
             WHERE memo_id NOT IN (SELECT id FROM memo)
                OR related_memo_id NOT IN (SELECT id FROM memo)",
            [],
        )?;
        tx.execute("DELETE FROM resource WHERE creator_id != ?1", params![user_id])?;
        tx.execute(
            "UPDATE resource SET memo_id = NULL
             WHERE memo_id IS NOT NULL AND memo_id NOT IN (SELECT id FROM memo)",
            [],
        )?;
        tx.execute("DELETE FROM activity WHERE creator_id != ?1", params![user_id])?;
        tx.execute(
            "DELETE FROM inbox WHERE receiver_id != ?1 OR sender_id != ?1",
            params![user_id],
        )?;
        tx.execute("DELETE FROM webhook WHERE creator_id != ?1", params![user_id])?;
        tx.execute("DELETE FROM reaction WHERE creator_id != ?1", params![user_id])?;
        tx.execute("DELETE FROM idp", [])?;
        tx.commit()?;
        export_conn.execute_batch("VACUUM;")?;
    }

    // Verify integrity of exported file and switch journal_mode to DELETE for universal server drop-in
    export_conn.execute_batch("PRAGMA journal_mode = DELETE;")?;
    let integrity: String = export_conn.query_row("PRAGMA integrity_check", [], |r| r.get(0))?;
    if integrity.to_lowercase() != "ok" {
        return Err(AppError::Backup(format!(
            "Exported database failed integrity check: {integrity}"
        )));
    }
    Ok(())
}

fn collect_scoped_assets(
    state: &AppState,
    staged_db: &Path,
    scope: &BackupScope,
) -> AppResult<Vec<(String, PathBuf)>> {
    match scope {
        BackupScope::AllUsers => {
            let mut entries = Vec::new();
            let assets_dir = state.assets_dir();
            if assets_dir.exists() {
                collect_relative_assets(&assets_dir, &assets_dir, &mut entries);
            }
            Ok(entries)
        }
        BackupScope::ActiveUser { .. } => {
            let conn = Connection::open(staged_db)?;
            let mut stmt = conn.prepare(
                "SELECT DISTINCT reference FROM resource WHERE reference IS NOT NULL AND reference != ''",
            )?;
            let refs = stmt
                .query_map([], |r| r.get::<_, String>(0))?
                .collect::<Result<Vec<_>, _>>()?;

            let assets_dir = state.assets_dir();
            let mut seen = HashSet::new();
            let mut entries = Vec::new();
            for reference in refs {
                if let Ok(abs_path) = media::resolve_safe_asset_path(&state.media_root, &reference) {
                    if abs_path.is_file() {
                        if let Ok(rel) = abs_path.strip_prefix(&assets_dir) {
                            let rel_posix = format!("assets/{}", rel.to_string_lossy().replace('\\', "/"));
                            if seen.insert(rel_posix.clone()) {
                                entries.push((rel_posix, abs_path));
                            }
                        }
                    }
                }
            }
            entries.sort_by(|a, b| a.0.cmp(&b.0));
            Ok(entries)
        }
    }
}

fn record_backup_history(state: &AppState, result: &ExportResult) -> AppResult<()> {
    let local = state.lock_local_db()?;
    let now = chrono::Utc::now().timestamp();
    local.execute(
        "INSERT INTO backup_history (mode, filename, path, size_bytes, sha256, created_ts)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            result.mode,
            result.filename,
            result.path,
            result.size_bytes as i64,
            result.sha256,
            now
        ],
    )?;
    Ok(())
}

pub fn list_backup_history(state: &AppState) -> AppResult<Vec<BackupHistoryEntry>> {
    let local = state.lock_local_db()?;
    let mut stmt = local.prepare(
        "SELECT id, mode, filename, path, size_bytes, sha256, created_ts
         FROM backup_history ORDER BY created_ts DESC, id DESC LIMIT 50",
    )?;
    let rows = stmt
        .query_map([], |r| {
            Ok(BackupHistoryEntry {
                id: r.get(0)?,
                mode: r.get(1)?,
                filename: r.get(2)?,
                path: r.get(3)?,
                size_bytes: r.get(4)?,
                sha256: r.get(5)?,
                created_ts: r.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

/// Mode 1: Database-Only Export (`.db`) — 1:1 Official Memos server drop-in ready (All Users default).
pub fn export_database_only(state: &AppState, output_dir: Option<&Path>) -> AppResult<ExportResult> {
    export_database_only_scoped(state, output_dir, &BackupScope::AllUsers)
}

fn staged_backup_exports_dir(state: &AppState) -> PathBuf {
    state.temp_dir().join("backup_exports")
}

/// Mode 1 (Scoped): Database-Only Export (`.db`) for either `AllUsers` (HOST/ADMIN) or `ActiveUser`.
pub fn export_database_only_scoped(
    state: &AppState,
    output_dir: Option<&Path>,
    scope: &BackupScope,
) -> AppResult<ExportResult> {
    export_database_only_internal(state, output_dir, scope, true)
}

/// Mode 1 (Staged): Writes `.db` strictly to ephemeral `temp/backup_exports/` without touching `backups/`
/// or recording history until the user confirms saving in the OS/SAF File Manager dialog.
pub fn export_database_only_staged(
    state: &AppState,
    scope: &BackupScope,
) -> AppResult<ExportResult> {
    let stage_dir = staged_backup_exports_dir(state);
    export_database_only_internal(state, Some(&stage_dir), scope, false)
}

fn export_database_only_internal(
    state: &AppState,
    output_dir: Option<&Path>,
    scope: &BackupScope,
    record_history: bool,
) -> AppResult<ExportResult> {
    let ts = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let scope_tag = scope.filename_tag();
    let filename = format!("Xmemos-{scope_tag}-db-v{}-{ts}.db", state.app_version);
    let target_dir = output_dir
        .map(Path::to_path_buf)
        .unwrap_or_else(|| state.backups_dir());
    fs::create_dir_all(&target_dir)?;
    let dest_path = target_dir.join(&filename);

    create_clean_db_snapshot(state, &dest_path, scope)?;

    let size_bytes = fs::metadata(&dest_path)?.len();
    let sha256 = sha256_file(&dest_path)?;
    let result = ExportResult {
        mode: "DB_ONLY".to_string(),
        scope: scope.as_str().to_string(),
        filename,
        path: dest_path.to_string_lossy().to_string(),
        size_bytes,
        sha256,
        media_count: 0,
    };
    if record_history {
        record_backup_history(state, &result)?;
    }
    Ok(result)
}

/// Mode 2: Full Bundle Export (`.xmemos` ZIP) — `memos_prod.db` + `assets/` + `manifest.json` (All Users default).
pub fn export_full_bundle(state: &AppState, output_dir: Option<&Path>) -> AppResult<ExportResult> {
    export_full_bundle_scoped(state, output_dir, &BackupScope::AllUsers)
}

/// Mode 2 (Scoped): Full Bundle Export (`.xmemos` ZIP) for either `AllUsers` (HOST/ADMIN) or `ActiveUser`.
pub fn export_full_bundle_scoped(
    state: &AppState,
    output_dir: Option<&Path>,
    scope: &BackupScope,
) -> AppResult<ExportResult> {
    export_full_bundle_internal(state, output_dir, scope, true)
}

/// Mode 2 (Staged): Writes `.xmemos` strictly to ephemeral `temp/backup_exports/` without touching `backups/`
/// or recording history until the user confirms saving in the OS/SAF File Manager dialog.
pub fn export_full_bundle_staged(
    state: &AppState,
    scope: &BackupScope,
) -> AppResult<ExportResult> {
    let stage_dir = staged_backup_exports_dir(state);
    export_full_bundle_internal(state, Some(&stage_dir), scope, false)
}

fn export_full_bundle_internal(
    state: &AppState,
    output_dir: Option<&Path>,
    scope: &BackupScope,
    record_history: bool,
) -> AppResult<ExportResult> {
    let ts = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let scope_tag = scope.filename_tag();
    let filename = format!("Xmemos-{scope_tag}-full-v{}-{ts}.xmemos", state.app_version);
    let target_dir = output_dir
        .map(Path::to_path_buf)
        .unwrap_or_else(|| state.backups_dir());
    fs::create_dir_all(&target_dir)?;
    let dest_path = target_dir.join(&filename);

    let stage_dir = state
        .temp_dir()
        .join(format!("export_{}", uuid::Uuid::new_v4().simple()));
    fs::create_dir_all(&stage_dir)?;
    let staged_db = stage_dir.join("memos_prod.db");
    create_clean_db_snapshot(state, &staged_db, scope)?;
    let db_sha256 = sha256_file(&staged_db)?;

    let asset_entries = collect_scoped_assets(state, &staged_db, scope)?;

    let zip_file = File::create(&dest_path)?;
    let mut writer = ZipWriter::new(zip_file);

    // 1. Write memos_prod.db (Deflate)
    let deflate_opts =
        SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    writer.start_file("memos_prod.db", deflate_opts)?;
    {
        let mut db_reader = File::open(&staged_db)?;
        std::io::copy(&mut db_reader, &mut writer)?;
    }

    // 2. Write assets/* (Stored for pre-compressed media, Deflate for compressible docs)
    let mut manifest_files = Vec::with_capacity(asset_entries.len());
    let mut media_total_bytes = 0u64;

    for (rel_posix, abs_path) in &asset_entries {
        let meta = fs::metadata(abs_path)?;
        let file_sha = sha256_file(abs_path)?;
        let method = if is_already_compressed_media(abs_path) {
            CompressionMethod::Stored
        } else {
            CompressionMethod::Deflated
        };
        let opts = SimpleFileOptions::default().compression_method(method);
        writer.start_file(rel_posix, opts)?;
        let mut f = File::open(abs_path)?;
        std::io::copy(&mut f, &mut writer)?;

        media_total_bytes += meta.len();
        manifest_files.push(ManifestFileEntry {
            path: rel_posix.clone(),
            size: meta.len(),
            sha256: file_sha,
        });
    }

    // 3. Write manifest.json
    let manifest = BundleManifest {
        format: "xmemos-bundle".to_string(),
        format_version: 1,
        app_version: state.app_version.clone(),
        exported_at_utc: chrono::Utc::now().to_rfc3339(),
        scope: scope.as_str().to_string(),
        memos_schema_compatible: true,
        db_filename: "memos_prod.db".to_string(),
        db_sha256,
        media_count: manifest_files.len(),
        media_total_bytes,
        files: manifest_files,
    };
    let manifest_json = serde_json::to_string_pretty(&manifest)?;
    writer.start_file("manifest.json", deflate_opts)?;
    writer.write_all(manifest_json.as_bytes())?;
    writer.finish()?;

    let _ = fs::remove_dir_all(&stage_dir);

    let size_bytes = fs::metadata(&dest_path)?.len();
    let sha256 = sha256_file(&dest_path)?;
    let result = ExportResult {
        mode: "FULL_BUNDLE".to_string(),
        scope: scope.as_str().to_string(),
        filename,
        path: dest_path.to_string_lossy().to_string(),
        size_bytes,
        sha256,
        media_count: manifest.media_count,
    };
    if record_history {
        record_backup_history(state, &result)?;
    }
    Ok(result)
}

/// Cleans up an ephemeral staged backup file in `temp/backup_exports/` after the OS/SAF Save dialog completes
/// or is cancelled. Only records an entry in `backup_history` when `saved_destination` is provided (not cancelled).
pub fn finalize_staged_export(
    state: &AppState,
    staged_path: &Path,
    saved_destination: Option<&str>,
    result: Option<&ExportResult>,
) -> AppResult<()> {
    let stage_root = staged_backup_exports_dir(state);
    if staged_path.exists() && stage_root.exists() {
        if let (Ok(canon_root), Ok(canon_file)) =
            (stage_root.canonicalize(), staged_path.canonicalize())
        {
            if canon_file.starts_with(&canon_root) && canon_file.is_file() {
                let _ = fs::remove_file(&canon_file);
            }
        }
    }

    if let (Some(dest), Some(res)) = (
        saved_destination.map(str::trim).filter(|s| !s.is_empty()),
        result,
    ) {
        let mut recorded = res.clone();
        recorded.path = dest.to_string();
        record_backup_history(state, &recorded)?;
    }
    Ok(())
}

fn validate_zip_entry_path(name: &str) -> AppResult<PathBuf> {
    let p = Path::new(name);
    if p.is_absolute() {
        return Err(AppError::Security(format!(
            "ZIP-slip blocked: absolute entry path '{name}'"
        )));
    }
    for comp in p.components() {
        match comp {
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(AppError::Security(format!(
                    "ZIP-slip blocked: unsafe entry path '{name}'"
                )));
            }
            Component::CurDir | Component::Normal(_) => {}
        }
    }
    Ok(p.to_path_buf())
}

struct StagedImport {
    stage_dir: PathBuf,
    staged_db: PathBuf,
    staged_assets: Option<PathBuf>,
    detected_format: String,
    checksum_verified: bool,
    media_file_count: usize,
    compatibility_notes: Vec<String>,
    user_count: i64,
    memo_count: i64,
    resource_count: i64,
    archive_size_bytes: u64,
}

fn stage_and_validate_import(state: &AppState, source_path: &Path) -> AppResult<StagedImport> {
    if !source_path.exists() || !source_path.is_file() {
        return Err(AppError::NotFound(format!(
            "Import source file does not exist: {}",
            source_path.display()
        )));
    }

    let archive_size_bytes = fs::metadata(source_path)?.len();
    let mut header = [0u8; 16];
    {
        let mut f = File::open(source_path)?;
        let n = f.read(&mut header)?;
        if n < 4 {
            return Err(AppError::Backup(
                "Import file is too small or empty".to_string(),
            ));
        }
    }

    let stage_dir = state
        .temp_dir()
        .join(format!("import_{}", uuid::Uuid::new_v4().simple()));
    fs::create_dir_all(&stage_dir)?;

    let is_sqlite = &header == SQLITE_MAGIC;
    let is_zip = header[0] == b'P' && header[1] == b'K';

    let (staged_db, staged_assets, detected_format, checksum_verified, media_file_count) = if is_sqlite
    {
        let db_dst = stage_dir.join("memos_prod.db");
        fs::copy(source_path, &db_dst)?;
        (db_dst, None, "SQLITE_DB".to_string(), true, 0usize)
    } else if is_zip {
        let file = File::open(source_path)?;
        let mut archive = ZipArchive::new(file)?;
        let mut total_uncompressed = 0u64;
        let mut media_count = 0usize;

        for i in 0..archive.len() {
            let mut entry = archive.by_index(i)?;
            let entry_name = entry.name()?;
            let safe_rel = validate_zip_entry_path(entry_name.as_ref())?;
            let out_path = stage_dir.join(&safe_rel);

            if entry.is_dir() {
                fs::create_dir_all(&out_path)?;
                continue;
            }

            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)?;
            }

            let mut out_file = File::create(&out_path)?;
            let mut buf = [0u8; 65536];
            loop {
                let n = entry.read(&mut buf)?;
                if n == 0 {
                    break;
                }
                total_uncompressed += n as u64;
                if total_uncompressed > MAX_DECOMPRESSION_BYTES {
                    let _ = fs::remove_dir_all(&stage_dir);
                    return Err(AppError::Security(
                        "Decompression limit exceeded; archive rejected".to_string(),
                    ));
                }
                out_file.write_all(&buf[..n])?;
            }

            if safe_rel.starts_with("assets") {
                media_count += 1;
            }
        }

        // Locate memos_prod.db (root or single nested .db)
        let candidate_db = stage_dir.join("memos_prod.db");
        let staged_db = if candidate_db.exists() {
            candidate_db
        } else {
            let found = fs::read_dir(&stage_dir)?
                .flatten()
                .map(|e| e.path())
                .find(|p| {
                    p.is_file()
                        && p.extension()
                            .and_then(|s| s.to_str())
                            .is_some_and(|e| e.eq_ignore_ascii_case("db") || e.eq_ignore_ascii_case("sqlite"))
                });
            found.ok_or_else(|| {
                let _ = fs::remove_dir_all(&stage_dir);
                AppError::Backup("Archive does not contain memos_prod.db".to_string())
            })?
        };

        // Verify SQLite magic bytes of extracted DB
        let mut db_magic = [0u8; 16];
        {
            let mut f = File::open(&staged_db)?;
            f.read_exact(&mut db_magic)?;
        }
        if &db_magic != SQLITE_MAGIC {
            let _ = fs::remove_dir_all(&stage_dir);
            return Err(AppError::Backup(
                "Extracted database file does not have a valid SQLite header".to_string(),
            ));
        }

        // Verify manifest.json SHA-256 hashes if present
        let manifest_path = stage_dir.join("manifest.json");
        let checksum_verified = if manifest_path.exists() {
            let manifest_raw = fs::read_to_string(&manifest_path)?;
            let manifest: BundleManifest = serde_json::from_str(&manifest_raw)?;
            let actual_db_sha = sha256_file(&staged_db)?;
            if actual_db_sha != manifest.db_sha256 {
                let _ = fs::remove_dir_all(&stage_dir);
                return Err(AppError::Backup(
                    "SHA-256 mismatch on memos_prod.db inside archive".to_string(),
                ));
            }
            for item in &manifest.files {
                let safe_rel = validate_zip_entry_path(&item.path)?;
                let target = stage_dir.join(safe_rel);
                if !target.exists() {
                    let _ = fs::remove_dir_all(&stage_dir);
                    return Err(AppError::Backup(format!(
                        "Missing media file referenced by manifest: {}",
                        item.path
                    )));
                }
                let actual_sha = sha256_file(&target)?;
                if actual_sha != item.sha256 {
                    let _ = fs::remove_dir_all(&stage_dir);
                    return Err(AppError::Backup(format!(
                        "SHA-256 mismatch on media file: {}",
                        item.path
                    )));
                }
            }
            true
        } else {
            false
        };

        let assets_candidate = stage_dir.join("assets");
        let staged_assets = if assets_candidate.exists() {
            Some(assets_candidate)
        } else {
            None
        };

        (
            staged_db,
            staged_assets,
            "XMEMOS_BUNDLE".to_string(),
            checksum_verified,
            media_count,
        )
    } else {
        let _ = fs::remove_dir_all(&stage_dir);
        return Err(AppError::Backup(
            "Unsupported file format: expected a .xmemos/.zip archive or a valid SQLite .db file"
                .to_string(),
        ));
    };

    // Inspect SQLite integrity and core Memos tables
    let conn = Connection::open(&staged_db)?;
    let quick: String = conn.query_row("PRAGMA quick_check", [], |r| r.get(0))?;
    if quick.to_lowercase() != "ok" {
        drop(conn);
        let _ = fs::remove_dir_all(&stage_dir);
        return Err(AppError::Backup(format!(
            "SQLite quick_check failed: {quick}"
        )));
    }
    let compatibility_notes = match validate_imported_memos_db(&conn) {
        Ok(notes) => notes,
        Err(err) => {
            drop(conn);
            let _ = fs::remove_dir_all(&stage_dir);
            return Err(err);
        }
    };

    let user_count: i64 = conn.query_row("SELECT COUNT(1) FROM user", [], |r| r.get(0))?;
    let memo_count: i64 = conn.query_row("SELECT COUNT(1) FROM memo", [], |r| r.get(0))?;
    let resource_count: i64 =
        conn.query_row("SELECT COUNT(1) FROM resource", [], |r| r.get(0))?;

    Ok(StagedImport {
        stage_dir,
        staged_db,
        staged_assets,
        detected_format,
        checksum_verified,
        media_file_count,
        compatibility_notes,
        user_count,
        memo_count,
        resource_count,
        archive_size_bytes,
    })
}

pub fn preview_import(state: &AppState, source_path: &Path) -> AppResult<ImportPreview> {
    let staged = stage_and_validate_import(state, source_path)?;
    let preview = ImportPreview {
        source_path: source_path.to_string_lossy().to_string(),
        detected_format: staged.detected_format,
        user_count: staged.user_count,
        memo_count: staged.memo_count,
        resource_count: staged.resource_count,
        media_file_count: staged.media_file_count,
        archive_size_bytes: staged.archive_size_bytes,
        checksum_verified: staged.checksum_verified,
        compatibility_notes: staged.compatibility_notes,
    };
    let _ = fs::remove_dir_all(&staged.stage_dir);
    Ok(preview)
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> AppResult<usize> {
    fs::create_dir_all(dst)?;
    let mut copied = 0usize;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if src_path.is_dir() {
            copied += copy_dir_recursive(&src_path, &dst_path)?;
        } else if src_path.is_file() {
            fs::copy(&src_path, &dst_path)?;
            copied += 1;
        }
    }
    Ok(copied)
}

/// Safe atomic import with pre-import rollback snapshot and automatic recovery on failure.
pub fn execute_atomic_import(state: &AppState, source_path: &Path) -> AppResult<ImportResult> {
    let staged = stage_and_validate_import(state, source_path)?;
    let temp_dir = state.temp_dir();
    fs::create_dir_all(&temp_dir)?;
    let rollback_path = temp_dir.join("rollback_snapshot.db");
    create_clean_db_snapshot(state, &rollback_path, &BackupScope::AllUsers)?;

    let swap_result: AppResult<usize> = (|| {
        // 1. Gracefully close active memos_prod.db connection
        {
            let mut guard = state
                .memos_conn
                .lock()
                .map_err(|e| AppError::Internal(format!("memos_conn lock poisoned: {e}")))?;
            *guard = None;
        }

        // 2. Remove WAL/SHM sidecars of old DB
        let wal_path = PathBuf::from(format!("{}-wal", state.memos_db_path.display()));
        let shm_path = PathBuf::from(format!("{}-shm", state.memos_db_path.display()));
        let _ = fs::remove_file(&wal_path);
        let _ = fs::remove_file(&shm_path);

        // 3. Swap staged memos_prod.db into place
        fs::copy(&staged.staged_db, &state.memos_db_path)?;

        // 4. Sync staged assets/ into media_root/assets/
        let restored_media = if let Some(ref src_assets) = staged.staged_assets {
            copy_dir_recursive(src_assets, &state.assets_dir())?
        } else {
            0usize
        };

        // 5. Re-open memos_prod.db in WAL mode and run non-destructive compatibility adapter
        let new_conn = open_memos_db(&state.memos_db_path)?;
        adapt_imported_memos_db(&new_conn)?;

        {
            let mut guard = state
                .memos_conn
                .lock()
                .map_err(|e| AppError::Internal(format!("memos_conn lock poisoned: {e}")))?;
            *guard = Some(new_conn);
        }

        Ok(restored_media)
    })();

    let restored_media_files = match swap_result {
        Ok(count) => count,
        Err(err) => {
            // Automatic Rollback: restore snapshot and re-open DB pool
            let _ = fs::copy(&rollback_path, &state.memos_db_path);
            let _ = fs::remove_file(format!("{}-wal", state.memos_db_path.display()));
            let _ = fs::remove_file(format!("{}-shm", state.memos_db_path.display()));
            if let Ok(restored_conn) = open_memos_db(&state.memos_db_path) {
                if let Ok(mut guard) = state.memos_conn.lock() {
                    *guard = Some(restored_conn);
                }
            }
            let _ = fs::remove_dir_all(&staged.stage_dir);
            let _ = fs::remove_file(&rollback_path);
            return Err(AppError::Backup(format!(
                "Import failed and was rolled back safely: {err}"
            )));
        }
    };

    // Reset active session to HOST/first user if previous session ID is invalid, and rebuild FTS5
    let first_user_id: Option<i64> = state
        .with_memos_db(|conn| {
            let id: Option<i64> = conn
                .query_row(
                    "SELECT id FROM user WHERE row_status = 'NORMAL' ORDER BY CASE role WHEN 'HOST' THEN 0 WHEN 'ADMIN' THEN 1 ELSE 2 END, id ASC LIMIT 1",
                    [],
                    |r| r.get(0),
                )
                .ok();
            Ok(id)
        })
        .unwrap_or(None);
    state.set_current_user_id(first_user_id)?;

    // Invalidate thumbnail cache
    if let Ok(entries) = fs::read_dir(state.thumbs_dir()) {
        for e in entries.flatten() {
            let _ = fs::remove_file(e.path());
        }
    }

    let fts_indexed_memos = search::rebuild_fts_index(state)?;
    let _ = fs::remove_dir_all(&staged.stage_dir);
    let _ = fs::remove_file(&rollback_path);

    Ok(ImportResult {
        user_count: staged.user_count,
        memo_count: staged.memo_count,
        resource_count: staged.resource_count,
        restored_media_files,
        fts_indexed_memos,
        compatibility_notes: staged.compatibility_notes,
    })
}
