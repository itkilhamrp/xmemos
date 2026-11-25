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

use std::fs;
use std::path::Path;
#[cfg(not(desktop))]
use std::path::PathBuf;

use base64::Engine;
use serde::Deserialize;
#[cfg(not(desktop))]
use tauri::Manager;
use tauri::State;
#[cfg(desktop)]
use tauri_plugin_xmemos_storage::XmemosStorageExt;

use crate::error::{AppError, AppResult};
use crate::service::auth::require_authenticated_user;
use crate::service::media::{self, CleanupResult, OrphanScanSummary, StorageBreakdown};
use crate::state::AppState;
use crate::store::memo;
use crate::store::resource::{self, Resource};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UploadResourceBytesPayload {
    pub filename: String,
    pub mime_type: Option<String>,
    pub base64_data: String,
    pub memo_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportResourceFilePayload {
    pub local_path: String,
    pub filename: String,
    pub mime_type: Option<String>,
    pub memo_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveResourceBytesPayload {
    pub filename: String,
    pub mime_type: Option<String>,
    pub base64_data: String,
}

#[tauri::command]
pub fn resource_upload_bytes(
    state: State<'_, AppState>,
    payload: UploadResourceBytesPayload,
) -> AppResult<Resource> {
    let current = require_authenticated_user(&state)?;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&payload.base64_data)
        .map_err(|e| AppError::Validation(format!("Invalid base64 payload: {e}")))?;

    media::save_new_resource(
        &state,
        current.id,
        &payload.filename,
        &bytes,
        payload.mime_type.as_deref(),
        payload.memo_id,
    )
}

#[tauri::command]
pub fn resource_import_file(
    state: State<'_, AppState>,
    payload: ImportResourceFilePayload,
) -> AppResult<Resource> {
    let current = require_authenticated_user(&state)?;
    let src = Path::new(&payload.local_path);
    if !src.exists() || !src.is_file() {
        return Err(AppError::NotFound(format!(
            "Picked file does not exist: {}",
            payload.local_path
        )));
    }
    let bytes = fs::read(src)?;
    let res = media::save_new_resource(
        &state,
        current.id,
        &payload.filename,
        &bytes,
        payload.mime_type.as_deref(),
        payload.memo_id,
    )?;

    // Clean up ephemeral temp file if it was staged in temp/
    if src.starts_with(state.temp_dir()) {
        let _ = fs::remove_file(src);
    }

    Ok(res)
}

#[tauri::command]
pub fn resource_list(state: State<'_, AppState>) -> AppResult<Vec<Resource>> {
    let current = require_authenticated_user(&state)?;
    state.with_memos_db(|conn| resource::list_resources_by_creator(conn, current.id))
}

#[tauri::command]
pub fn resource_delete(state: State<'_, AppState>, resource_id: i64) -> AppResult<()> {
    let current = require_authenticated_user(&state)?;
    let res = state.with_memos_db(|conn| {
        let item = resource::get_resource_by_id(conn, resource_id)?
            .ok_or_else(|| AppError::NotFound(format!("Resource {resource_id} not found")))?;
        if item.creator_id != current.id && current.role != "HOST" && current.role != "ADMIN" {
            return Err(AppError::Forbidden(
                "Only the resource owner or administrator can delete it".to_string(),
            ));
        }
        resource::delete_resource(conn, resource_id)?;
        Ok(item)
    })?;

    if !res.reference.is_empty() {
        if let Ok(path) = media::resolve_safe_asset_path(&state.media_root, &res.reference) {
            let _ = fs::remove_file(path);
        }
    }
    let thumb = state.thumbs_dir().join(format!("{}.webp", res.uid));
    let _ = fs::remove_file(thumb);
    Ok(())
}

#[tauri::command]
pub fn resource_storage_breakdown(state: State<'_, AppState>) -> AppResult<StorageBreakdown> {
    let _ = require_authenticated_user(&state)?;
    media::get_storage_breakdown(&state)
}

#[tauri::command]
pub fn resource_scan_orphans(state: State<'_, AppState>) -> AppResult<OrphanScanSummary> {
    let _ = require_authenticated_user(&state)?;
    media::scan_orphan_files(&state)
}

#[tauri::command]
pub fn resource_clean_orphans(state: State<'_, AppState>) -> AppResult<CleanupResult> {
    let _ = require_authenticated_user(&state)?;
    media::clean_orphan_files(&state)
}

#[tauri::command]
pub fn resource_externalize_blobs(state: State<'_, AppState>) -> AppResult<usize> {
    let _ = require_authenticated_user(&state)?;
    media::externalize_database_blobs(&state)
}

fn sanitize_download_filename(raw: &str) -> String {
    let base = Path::new(raw)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("file");
    let cleaned: String = base
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | '\0' => '_',
            _ => c,
        })
        .collect();
    let trimmed = cleaned.trim().trim_matches('.');
    if trimmed.is_empty() {
        "file".to_string()
    } else {
        trimmed.to_string()
    }
}

#[cfg(not(desktop))]
fn unique_download_path(dir: &Path, filename: &str) -> PathBuf {
    let candidate = dir.join(filename);
    if !candidate.exists() {
        return candidate;
    }
    let stem = Path::new(filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("file");
    let ext = Path::new(filename)
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("");
    for idx in 1..=9999 {
        let name = if ext.is_empty() {
            format!("{stem} ({idx})")
        } else {
            format!("{stem} ({idx}).{ext}")
        };
        let next = dir.join(name);
        if !next.exists() {
            return next;
        }
    }
    dir.join(format!("{stem}-{}", chrono::Utc::now().timestamp()))
}

#[cfg(desktop)]
fn save_bytes_for_platform(
    app: &tauri::AppHandle,
    safe_name: &str,
    mime: &str,
    bytes: &[u8],
) -> AppResult<Option<String>> {
    let res = app
        .xmemos_storage()
        .save_bytes_with_dialog(safe_name, mime, bytes)
        .map_err(|e| AppError::Internal(format!("Failed to save file: {e}")))?;

    if res.cancelled {
        Ok(None)
    } else {
        Ok(Some(res.saved_uri.unwrap_or_else(|| safe_name.to_string())))
    }
}

#[cfg(not(desktop))]
fn save_bytes_for_platform(
    app: &tauri::AppHandle,
    safe_name: &str,
    _mime: &str,
    bytes: &[u8],
) -> AppResult<Option<String>> {
    let target_dir = app
        .path()
        .download_dir()
        .or_else(|_| app.path().picture_dir())
        .or_else(|_| app.path().document_dir())
        .or_else(|_| app.path().app_data_dir())
        .map_err(|e| AppError::Internal(format!("Unable to resolve Downloads directory: {e}")))?;

    fs::create_dir_all(&target_dir)?;
    let dest = unique_download_path(&target_dir, safe_name);
    fs::write(&dest, bytes)?;

    Ok(Some(
        dest.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(safe_name)
            .to_string(),
    ))
}

#[tauri::command]
pub async fn resource_export_to_downloads(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    uid: String,
) -> AppResult<Option<String>> {
    let clean_uid = uid.trim();
    if clean_uid.is_empty()
        || !clean_uid
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    {
        return Err(AppError::Validation("Invalid resource identifier".to_string()));
    }

    let viewer_id = state.current_user_id()?;
    let res = state.with_memos_db(|conn| {
        let item = resource::get_resource_by_uid(conn, clean_uid)?
            .ok_or_else(|| AppError::NotFound(format!("Resource '{clean_uid}' not found")))?;
        if let Some(memo_id) = item.memo_id {
            if let Some(m) = memo::get_memo_by_id(conn, memo_id)? {
                if !memo::can_view_memo(&m, viewer_id) {
                    return Err(AppError::Forbidden(
                        "You do not have permission to download this media".to_string(),
                    ));
                }
            }
        } else if viewer_id != Some(item.creator_id) {
            return Err(AppError::Forbidden(
                "Unbound resource is private to creator".to_string(),
            ));
        }
        Ok(item)
    })?;

    let (bytes, mime) = media::read_resource_bytes(&state, &res, false)?;
    let safe_name = sanitize_download_filename(&res.filename);
    save_bytes_for_platform(&app, &safe_name, &mime, &bytes)
}

#[tauri::command]
pub async fn resource_save_bytes(
    app: tauri::AppHandle,
    payload: SaveResourceBytesPayload,
) -> AppResult<Option<String>> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&payload.base64_data)
        .map_err(|e| AppError::Validation(format!("Invalid base64 payload: {e}")))?;

    let safe_name = sanitize_download_filename(&payload.filename);
    let mime = payload
        .mime_type
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or("application/octet-stream");

    save_bytes_for_platform(&app, &safe_name, mime, &bytes)
}
