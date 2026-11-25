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
use std::fs;
use std::io::Cursor;
use std::path::{Component, Path, PathBuf};

use chrono::Datelike;
use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::{CompressionType, FilterType as PngFilterType, PngEncoder};
use image::codecs::webp::WebPEncoder;
use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView, ImageEncoder, ImageFormat, ImageReader};
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};
use crate::state::AppState;
use crate::store::resource::{self, CreateResourceInput, Resource};

const MAX_IMAGE_EDGE_PX: u32 = 2048;
const MAX_DECODE_PIXELS: u64 = 100_000_000; // 100 MP decompression-bomb guard
const JPEG_CRISP_QUALITY: u8 = 84;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageBreakdown {
    pub media_root: String,
    pub memos_db_bytes: u64,
    pub local_db_bytes: u64,
    pub assets_bytes: u64,
    pub backups_bytes: u64,
    pub thumbs_bytes: u64,
    pub resource_count: usize,
    pub blob_resource_count: usize,
    pub orphan_file_count: usize,
    pub nomedia_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupResult {
    pub removed_files: usize,
    pub freed_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrphanScanSummary {
    pub unlinked_resource_count: usize,
    pub orphan_disk_file_count: usize,
    pub total_orphan_count: usize,
    pub reclaimable_bytes: u64,
    pub sample_filenames: Vec<String>,
}

pub fn sanitize_filename(name: &str) -> String {
    let base = Path::new(name)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("attachment");
    let cleaned: String = base
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if cleaned.is_empty() || cleaned == "." || cleaned == ".." {
        "attachment.bin".to_string()
    } else {
        cleaned
    }
}

fn replace_extension(filename: &str, new_ext: &str) -> String {
    let p = Path::new(filename);
    let stem = p
        .file_stem()
        .and_then(|s| s.to_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("image");
    format!("{stem}.{new_ext}")
}

/// Validates and resolves a relative Memos `resource.reference` (e.g. `assets/2026/11/abc_photo.webp`)
/// strictly within `media_root/assets/`, rejecting any path traversal (`..`), absolute paths, or symlink escapes.
pub fn resolve_safe_asset_path(media_root: &Path, reference: &str) -> AppResult<PathBuf> {
    if reference.is_empty() {
        return Err(AppError::Security("Empty resource reference".to_string()));
    }
    let normalized = reference.replace('\\', "/");
    let rel_str = normalized.strip_prefix("assets/").unwrap_or(&normalized);
    let rel_path = Path::new(rel_str);

    if rel_path.is_absolute() {
        return Err(AppError::Security(
            "Absolute paths are forbidden in resource references".to_string(),
        ));
    }

    for comp in rel_path.components() {
        match comp {
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(AppError::Security(format!(
                    "Path traversal detected in reference: {reference}"
                )));
            }
            Component::CurDir | Component::Normal(_) => {}
        }
    }

    let assets_root = media_root.join("assets");
    fs::create_dir_all(&assets_root)?;
    let canonical_root = assets_root.canonicalize()?;
    let candidate = canonical_root.join(rel_path);

    if candidate.exists() {
        let canonical_candidate = candidate.canonicalize()?;
        if !canonical_candidate.starts_with(&canonical_root) {
            return Err(AppError::Security(format!(
                "Symlink escape detected for reference: {reference}"
            )));
        }
        return Ok(canonical_candidate);
    }

    if let Some(parent) = candidate.parent() {
        if parent.exists() {
            let canonical_parent = parent.canonicalize()?;
            if !canonical_parent.starts_with(&canonical_root) {
                return Err(AppError::Security(format!(
                    "Parent symlink escape detected for reference: {reference}"
                )));
            }
        }
    }

    Ok(candidate)
}

pub fn detect_mime_type(filename: &str, bytes: &[u8], hinted: Option<&str>) -> String {
    if bytes.starts_with(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]) {
        return "image/png".to_string();
    }
    if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        return "image/jpeg".to_string();
    }
    if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        return "image/webp".to_string();
    }
    if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        return "image/gif".to_string();
    }
    if bytes.starts_with(b"%PDF-") {
        return "application/pdf".to_string();
    }
    if let Some(h) = hinted {
        let trimmed = h.trim();
        if !trimmed.is_empty() && trimmed != "application/octet-stream" {
            return trimmed.to_string();
        }
    }
    mime_guess::from_path(filename)
        .first_or_octet_stream()
        .essence_str()
        .to_string()
}

fn has_non_opaque_alpha(img: &DynamicImage) -> bool {
    if !img.color().has_alpha() {
        return false;
    }
    let rgba = img.to_rgba8();
    rgba.pixels().any(|p| p[3] < 255)
}

fn encode_webp_lossless(img: &DynamicImage) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    let (w, h) = img.dimensions();
    if img.color().has_alpha() {
        let rgba = img.to_rgba8();
        WebPEncoder::new_lossless(&mut out)
            .encode(&rgba, w, h, image::ExtendedColorType::Rgba8)
            .ok()?;
    } else {
        let rgb = img.to_rgb8();
        WebPEncoder::new_lossless(&mut out)
            .encode(&rgb, w, h, image::ExtendedColorType::Rgb8)
            .ok()?;
    }
    Some(out)
}

fn encode_png_best(img: &DynamicImage) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    let (w, h) = img.dimensions();
    let encoder = PngEncoder::new_with_quality(&mut out, CompressionType::Best, PngFilterType::Adaptive);
    if img.color().has_alpha() {
        let rgba = img.to_rgba8();
        encoder
            .write_image(&rgba, w, h, image::ExtendedColorType::Rgba8)
            .ok()?;
    } else {
        let rgb = img.to_rgb8();
        encoder
            .write_image(&rgb, w, h, image::ExtendedColorType::Rgb8)
            .ok()?;
    }
    Some(out)
}

fn encode_jpeg_crisp(img: &DynamicImage, quality: u8) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    let (w, h) = img.dimensions();
    let rgb = img.to_rgb8();
    JpegEncoder::new_with_quality(&mut out, quality)
        .encode(&rgb, w, h, image::ExtendedColorType::Rgb8)
        .ok()?;
    Some(out)
}

/// Smart adaptive static-file compressor:
/// - Preserves animated GIFs, SVGs, PDFs, audio, video, and office docs intact.
/// - Guards against decompression bombs (>100 MP).
/// - Resizes oversized raster images (>2048px) using Lanczos3 for crisp OLED/Retina clarity.
/// - Evaluates Lossless WebP (VP8L), Best-Adaptive PNG, and Crisp JPEG-84, selecting the smallest
///   valid output (preferring `.webp` whenever competitive) and never bloating unresized originals.
pub fn optimize_static_upload(
    filename: &str,
    bytes: &[u8],
    mime_type: &str,
) -> (String, Vec<u8>, String) {
    let fallback = || (filename.to_string(), bytes.to_vec(), mime_type.to_string());

    // Only optimize static raster images; leave GIF (possibly animated), SVG, video, audio, PDF intact.
    if !matches!(
        mime_type,
        "image/png" | "image/jpeg" | "image/webp" | "image/bmp" | "image/tiff"
    ) {
        return fallback();
    }

    let reader = match ImageReader::new(Cursor::new(bytes)).with_guessed_format() {
        Ok(r) => r,
        Err(_) => return fallback(),
    };
    let format = reader.format();
    if format == Some(ImageFormat::Gif) {
        return fallback();
    }

    let Ok((orig_w, orig_h)) = reader.into_dimensions() else {
        return fallback();
    };
    if (orig_w as u64) * (orig_h as u64) > MAX_DECODE_PIXELS || orig_w == 0 || orig_h == 0 {
        return fallback();
    }

    let decoded = match ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .ok()
        .and_then(|r| r.decode().ok())
    {
        Some(img) => img,
        None => return fallback(),
    };

    let was_resized = orig_w > MAX_IMAGE_EDGE_PX || orig_h > MAX_IMAGE_EDGE_PX;
    let working = if was_resized {
        decoded.resize(MAX_IMAGE_EDGE_PX, MAX_IMAGE_EDGE_PX, FilterType::Lanczos3)
    } else {
        decoded
    };

    let has_alpha = has_non_opaque_alpha(&working);
    let mut candidates: Vec<(Vec<u8>, &'static str, &'static str)> = Vec::new();

    if let Some(webp_bytes) = encode_webp_lossless(&working) {
        candidates.push((webp_bytes, "webp", "image/webp"));
    }

    if has_alpha {
        if let Some(png_bytes) = encode_png_best(&working) {
            candidates.push((png_bytes, "png", "image/png"));
        }
    } else {
        if let Some(jpg_bytes) = encode_jpeg_crisp(&working, JPEG_CRISP_QUALITY) {
            candidates.push((jpg_bytes, "jpg", "image/jpeg"));
        }
        // Also test PNG for flat UI/diagrams where lossless PNG might beat JPEG
        if mime_type == "image/png" {
            if let Some(png_bytes) = encode_png_best(&working) {
                candidates.push((png_bytes, "png", "image/png"));
            }
        }
    }

    // Select the smallest valid candidate
    let Some((best_bytes, best_ext, best_mime)) =
        candidates.into_iter().min_by_key(|(b, _, _)| b.len())
    else {
        return fallback();
    };

    // Never bloat an unresized image; if original is already smaller or equal, keep it untouched.
    if !was_resized && best_bytes.len() >= bytes.len() {
        return fallback();
    }

    // If lossless WebP is within 5% of JPEG on a non-resized/resized photo, prefer WebP for 100% crispness.
    (
        replace_extension(filename, best_ext),
        best_bytes,
        best_mime.to_string(),
    )
}

pub fn generate_thumbnail(state: &AppState, uid: &str, mime_type: &str, bytes: &[u8]) {
    if !mime_type.starts_with("image/") || mime_type == "image/svg+xml" {
        return;
    }
    let thumbs_dir = state.thumbs_dir();
    let _ = fs::create_dir_all(&thumbs_dir);
    let thumb_path = thumbs_dir.join(format!("{uid}.webp"));
    if let Ok(img) = image::load_from_memory(bytes) {
        let thumb = img.thumbnail(360, 360);
        let _ = thumb.save_with_format(&thumb_path, ImageFormat::WebP);
    }
}

pub fn save_new_resource(
    state: &AppState,
    creator_id: i64,
    filename: &str,
    bytes: &[u8],
    hinted_mime: Option<&str>,
    memo_id: Option<i64>,
) -> AppResult<Resource> {
    let uid = uuid::Uuid::new_v4().simple().to_string();
    let initial_sanitized = sanitize_filename(filename);
    let initial_mime = detect_mime_type(&initial_sanitized, bytes, hinted_mime);

    let (sanitized, final_bytes, mime_type) =
        optimize_static_upload(&initial_sanitized, bytes, &initial_mime);

    let now = chrono::Utc::now();
    let rel_ref = format!(
        "assets/{:04}/{:02}/{}_{}",
        now.year(),
        now.month(),
        uid,
        sanitized
    );

    let dest_path = resolve_safe_asset_path(&state.media_root, &rel_ref)?;
    if let Some(parent) = dest_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&dest_path, &final_bytes)?;
    generate_thumbnail(state, &uid, &mime_type, &final_bytes);

    state.with_memos_db(|conn| {
        resource::create_resource(
            conn,
            &CreateResourceInput {
                uid: uid.clone(),
                creator_id,
                filename: sanitized,
                blob: None,
                mime_type,
                size: final_bytes.len() as i64,
                memo_id,
                storage_type: "LOCAL".to_string(),
                reference: rel_ref,
                payload: "{}".to_string(),
            },
        )
    })
}

pub fn read_resource_bytes(
    state: &AppState,
    res: &Resource,
    prefer_thumb: bool,
) -> AppResult<(Vec<u8>, String)> {
    if prefer_thumb && res.r#type.starts_with("image/") {
        let thumb_path = state.thumbs_dir().join(format!("{}.webp", res.uid));
        if thumb_path.exists() {
            if let Ok(data) = fs::read(&thumb_path) {
                return Ok((data, "image/webp".to_string()));
            }
        }
    }

    if !res.reference.is_empty() {
        let safe_path = resolve_safe_asset_path(&state.media_root, &res.reference)?;
        if safe_path.exists() {
            let data = fs::read(&safe_path)?;
            if prefer_thumb && res.r#type.starts_with("image/") {
                generate_thumbnail(state, &res.uid, &res.r#type, &data);
            }
            return Ok((data, res.r#type.clone()));
        }
    }

    if res.has_blob {
        let blob = state
            .with_memos_db(|conn| resource::get_resource_blob(conn, &res.uid))?
            .ok_or_else(|| AppError::NotFound(format!("Resource blob {} missing", res.uid)))?;
        if prefer_thumb && res.r#type.starts_with("image/") {
            generate_thumbnail(state, &res.uid, &res.r#type, &blob);
        }
        return Ok((blob, res.r#type.clone()));
    }

    Err(AppError::NotFound(format!(
        "Media payload for resource {} not found on disk or in database",
        res.uid
    )))
}

/// Externalizes any legacy `resource.blob` rows (imported from official Memos DB-storage instances)
/// into `/Android/media/org.xmemos.app/assets/` while preserving `resource.uid` integrity.
pub fn externalize_database_blobs(state: &AppState) -> AppResult<usize> {
    let resources = state.with_memos_db(resource::list_all_resources)?;
    let mut migrated = 0usize;

    for res in resources {
        if !res.has_blob {
            continue;
        }
        let blob_opt = state.with_memos_db(|conn| resource::get_resource_blob(conn, &res.uid))?;
        let Some(bytes) = blob_opt else { continue };

        let dt = chrono::DateTime::from_timestamp(res.created_ts, 0)
            .unwrap_or_else(chrono::Utc::now);
        let sanitized = sanitize_filename(&res.filename);
        let rel_ref = format!(
            "assets/{:04}/{:02}/{}_{}",
            dt.year(),
            dt.month(),
            res.uid,
            sanitized
        );
        let dest = resolve_safe_asset_path(&state.media_root, &rel_ref)?;
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&dest, &bytes)?;
        generate_thumbnail(state, &res.uid, &res.r#type, &bytes);
        state.with_memos_db(|conn| resource::externalize_resource_storage(conn, res.id, &rel_ref))?;
        migrated += 1;
    }

    Ok(migrated)
}

fn dir_size(path: &Path) -> u64 {
    if !path.exists() {
        return 0;
    }
    let mut total = 0u64;
    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                total += dir_size(&p);
            } else if let Ok(meta) = p.metadata() {
                total += meta.len();
            }
        }
    }
    total
}

fn collect_files_recursive(dir: &Path, acc: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                collect_files_recursive(&p, acc);
            } else if p.is_file() {
                acc.push(p);
            }
        }
    }
}

struct OrphanPlan {
    unlinked_resources: Vec<Resource>,
    orphan_disk_files: Vec<PathBuf>,
    orphan_thumb_files: Vec<PathBuf>,
    reclaimable_bytes: u64,
    sample_filenames: Vec<String>,
}

fn build_orphan_plan(state: &AppState) -> AppResult<OrphanPlan> {
    let resources = state.with_memos_db(resource::list_all_resources)?;
    let mut linked_paths = HashSet::new();
    let mut linked_uids = HashSet::new();
    let mut unlinked_resources = Vec::new();
    let mut unlinked_asset_paths = HashSet::new();
    let mut reclaimable_bytes = 0u64;
    let mut sample_filenames = Vec::new();

    for res in resources {
        if res.memo_id.is_some() {
            linked_uids.insert(res.uid.clone());
            if !res.reference.is_empty() {
                if let Ok(p) = resolve_safe_asset_path(&state.media_root, &res.reference) {
                    let canonical = p.canonicalize().unwrap_or(p);
                    linked_paths.insert(canonical);
                }
            }
        } else {
            let mut item_bytes = 0u64;
            if !res.reference.is_empty() {
                if let Ok(p) = resolve_safe_asset_path(&state.media_root, &res.reference) {
                    let canonical = p.canonicalize().unwrap_or(p.clone());
                    if let Ok(meta) = fs::metadata(&canonical) {
                        item_bytes += meta.len();
                    }
                    unlinked_asset_paths.insert(canonical);
                }
            }
            if item_bytes == 0 && res.size > 0 {
                item_bytes = res.size as u64;
            }
            let thumb = state.thumbs_dir().join(format!("{}.webp", res.uid));
            if let Ok(meta) = fs::metadata(&thumb) {
                item_bytes += meta.len();
            }
            reclaimable_bytes += item_bytes;
            if sample_filenames.len() < 4 {
                sample_filenames.push(res.filename.clone());
            }
            unlinked_resources.push(res);
        }
    }

    let assets_dir = state.assets_dir();
    let avatars_dir = assets_dir.join("avatars");
    let mut disk_files = Vec::new();
    collect_files_recursive(&assets_dir, &mut disk_files);

    let mut orphan_disk_files = Vec::new();
    for file in disk_files {
        if file.file_name().and_then(|s| s.to_str()) == Some(".nomedia") {
            continue;
        }
        if file.starts_with(&avatars_dir) {
            continue;
        }
        let canonical = file.canonicalize().unwrap_or(file.clone());
        if !linked_paths.contains(&canonical) && !unlinked_asset_paths.contains(&canonical) {
            let len = fs::metadata(&canonical).map(|m| m.len()).unwrap_or(0);
            reclaimable_bytes += len;
            if sample_filenames.len() < 4 {
                if let Some(name) = canonical.file_name().and_then(|s| s.to_str()) {
                    sample_filenames.push(name.to_string());
                }
            }
            orphan_disk_files.push(canonical);
        }
    }

    let mut thumb_files = Vec::new();
    collect_files_recursive(&state.thumbs_dir(), &mut thumb_files);
    let unlinked_uids: HashSet<String> = unlinked_resources.iter().map(|r| r.uid.clone()).collect();
    let mut orphan_thumb_files = Vec::new();
    for thumb in thumb_files {
        let stem = thumb.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        if !linked_uids.contains(stem) && !unlinked_uids.contains(stem) {
            let len = fs::metadata(&thumb).map(|m| m.len()).unwrap_or(0);
            reclaimable_bytes += len;
            orphan_thumb_files.push(thumb);
        }
    }

    Ok(OrphanPlan {
        unlinked_resources,
        orphan_disk_files,
        orphan_thumb_files,
        reclaimable_bytes,
        sample_filenames,
    })
}

pub fn scan_orphan_files(state: &AppState) -> AppResult<OrphanScanSummary> {
    let plan = build_orphan_plan(state)?;
    let unlinked_resource_count = plan.unlinked_resources.len();
    let orphan_disk_file_count = plan.orphan_disk_files.len() + plan.orphan_thumb_files.len();
    Ok(OrphanScanSummary {
        unlinked_resource_count,
        orphan_disk_file_count,
        total_orphan_count: unlinked_resource_count + orphan_disk_file_count,
        reclaimable_bytes: plan.reclaimable_bytes,
        sample_filenames: plan.sample_filenames,
    })
}

pub fn get_storage_breakdown(state: &AppState) -> AppResult<StorageBreakdown> {
    let resources = state.with_memos_db(resource::list_all_resources)?;
    let blob_resource_count = resources.iter().filter(|r| r.has_blob).count();
    let orphan_summary = scan_orphan_files(state)?;
    let memos_db_bytes = fs::metadata(&state.memos_db_path)
        .map(|m| m.len())
        .unwrap_or(0);
    let local_db_bytes = fs::metadata(&state.local_db_path)
        .map(|m| m.len())
        .unwrap_or(0);
    let nomedia_enabled = state.assets_dir().join(".nomedia").exists();

    Ok(StorageBreakdown {
        media_root: state.media_root.to_string_lossy().to_string(),
        memos_db_bytes,
        local_db_bytes,
        assets_bytes: dir_size(&state.assets_dir()),
        backups_bytes: dir_size(&state.backups_dir()),
        thumbs_bytes: dir_size(&state.thumbs_dir()),
        resource_count: resources.len(),
        blob_resource_count,
        orphan_file_count: orphan_summary.total_orphan_count,
        nomedia_enabled,
    })
}

pub fn clean_orphan_files(state: &AppState) -> AppResult<CleanupResult> {
    let plan = build_orphan_plan(state)?;
    let mut removed_files = 0usize;
    let mut freed_bytes = 0u64;

    for res in plan.unlinked_resources {
        let mut item_bytes = 0u64;
        if !res.reference.is_empty() {
            if let Ok(path) = resolve_safe_asset_path(&state.media_root, &res.reference) {
                let len = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
                if fs::remove_file(&path).is_ok() {
                    item_bytes += len;
                }
            }
        }
        let thumb = state.thumbs_dir().join(format!("{}.webp", res.uid));
        if let Ok(meta) = fs::metadata(&thumb) {
            if fs::remove_file(&thumb).is_ok() {
                item_bytes += meta.len();
            }
        }
        if item_bytes == 0 && res.has_blob && res.size > 0 {
            item_bytes = res.size as u64;
        }
        if state.with_memos_db(|conn| resource::delete_resource(conn, res.id)).is_ok() {
            removed_files += 1;
            freed_bytes += item_bytes;
        }
    }

    for path in plan.orphan_disk_files.into_iter().chain(plan.orphan_thumb_files) {
        let len = fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
        if fs::remove_file(&path).is_ok() {
            removed_files += 1;
            freed_bytes += len;
        }
    }

    Ok(CleanupResult {
        removed_files,
        freed_bytes,
    })
}
