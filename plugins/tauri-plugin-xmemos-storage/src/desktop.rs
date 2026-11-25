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
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Manager, Runtime};

use crate::desktop_dialog;
use crate::desktop_shortcut;
use crate::error::{Error, Result};
use crate::models::*;

pub fn init<R: Runtime, C: DeserializeOwned>(
    app: &AppHandle<R>,
    _api: PluginApi<R, C>,
) -> Result<XmemosStorage<R>> {
    let initial_uid = desktop_shortcut::extract_open_memo_uid_from_args(std::env::args());
    Ok(XmemosStorage {
        app: app.clone(),
        pending_shortcut_uid: Mutex::new(initial_uid),
    })
}

/// Resolves a portable data base directory when `portable.flag` exists beside the executable
/// or `XMEMOS_PORTABLE=1` is set (used by Windows Portable & portable Linux builds).
pub fn resolve_portable_data_dir() -> Option<PathBuf> {
    let exe_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(Path::to_path_buf))?;

    let env_enabled = std::env::var("XMEMOS_PORTABLE")
        .map(|v| matches!(v.trim().to_ascii_lowercase().as_str(), "1" | "true" | "yes"))
        .unwrap_or(false);

    if env_enabled || exe_dir.join("portable.flag").exists() {
        Some(exe_dir.join("data"))
    } else {
        None
    }
}

fn infer_mime_from_path(path: &Path) -> String {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_ascii_lowercase())
        .unwrap_or_default();
    match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "mp3" => "audio/mpeg",
        "ogg" => "audio/ogg",
        "wav" => "audio/wav",
        "pdf" => "application/pdf",
        "zip" => "application/zip",
        "db" | "sqlite" | "sqlite3" => "application/x-sqlite3",
        "md" => "text/markdown",
        "txt" => "text/plain",
        "json" => "application/json",
        _ => "application/octet-stream",
    }
    .to_string()
}

pub struct XmemosStorage<R: Runtime> {
    app: AppHandle<R>,
    pending_shortcut_uid: Mutex<Option<String>>,
}

impl<R: Runtime> XmemosStorage<R> {
    fn resolve_root(&self) -> Result<PathBuf> {
        let base = if let Some(portable_base) = resolve_portable_data_dir() {
            portable_base
        } else {
            self.app
                .path()
                .app_data_dir()
                .map_err(|e| Error::Storage(e.to_string()))?
        };
        let root = base.join("media");
        fs::create_dir_all(root.join("assets").join("avatars"))?;
        fs::create_dir_all(root.join("backups"))?;
        fs::create_dir_all(root.join("temp"))?;
        Ok(root)
    }

    pub fn get_media_root(&self) -> Result<MediaRootInfo> {
        let root = self.resolve_root()?;
        Ok(MediaRootInfo {
            root_path: root.to_string_lossy().to_string(),
            assets_path: root.join("assets").to_string_lossy().to_string(),
            backups_path: root.join("backups").to_string_lossy().to_string(),
            temp_path: root.join("temp").to_string_lossy().to_string(),
            is_android_media_dir: false,
        })
    }

    pub fn set_nomedia(&self, req: NomediaRequest) -> Result<NomediaStatus> {
        let root = self.resolve_root()?;
        let nomedia_path = root.join("assets").join(".nomedia");
        if req.enabled {
            fs::write(&nomedia_path, b"")?;
        } else if nomedia_path.exists() {
            fs::remove_file(&nomedia_path)?;
        }
        Ok(NomediaStatus {
            enabled: nomedia_path.exists(),
            path: nomedia_path.to_string_lossy().to_string(),
        })
    }

    pub fn get_nomedia_status(&self) -> Result<NomediaStatus> {
        let root = self.resolve_root()?;
        let nomedia_path = root.join("assets").join(".nomedia");
        Ok(NomediaStatus {
            enabled: nomedia_path.exists(),
            path: nomedia_path.to_string_lossy().to_string(),
        })
    }

    pub fn pick_document(&self, req: PickDocumentRequest) -> Result<PickDocumentResponse> {
        let picked = desktop_dialog::open_file_dialog(&self.app, "Select File", &req.mime_types)?;
        let Some(src_path) = picked else {
            return Ok(PickDocumentResponse {
                cancelled: true,
                local_path: None,
                filename: None,
                mime_type: None,
                size: 0,
            });
        };

        let filename = src_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("file")
            .to_string();
        let mime_type = infer_mime_from_path(&src_path);
        let size = fs::metadata(&src_path).map(|m| m.len()).unwrap_or(0);

        let resolved_path = if req.copy_to_temp {
            let root = self.resolve_root()?;
            let stamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0);
            let safe_name: String = filename
                .chars()
                .map(|c| if matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|') { '_' } else { c })
                .collect();
            let staged = root.join("temp").join(format!("{stamp}-{safe_name}"));
            fs::copy(&src_path, &staged)?;
            staged
        } else {
            src_path
        };

        Ok(PickDocumentResponse {
            cancelled: false,
            local_path: Some(resolved_path.to_string_lossy().to_string()),
            filename: Some(filename),
            mime_type: Some(mime_type),
            size,
        })
    }

    pub fn save_document(&self, req: SaveDocumentRequest) -> Result<SaveDocumentResponse> {
        let picked = desktop_dialog::save_file_dialog(
            &self.app,
            "Save File As",
            &req.suggested_filename,
            &req.mime_type,
        )?;
        let Some(dest_path) = picked else {
            return Ok(SaveDocumentResponse {
                cancelled: true,
                saved_uri: None,
                bytes_written: 0,
            });
        };

        if let Some(parent) = dest_path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        let bytes = fs::copy(&req.source_path, &dest_path)?;
        Ok(SaveDocumentResponse {
            cancelled: false,
            saved_uri: Some(dest_path.to_string_lossy().to_string()),
            bytes_written: bytes,
        })
    }

    /// Prompts the user with the native Desktop OS File Manager Save dialog and writes raw bytes
    /// directly to the chosen path (used for static memo files, media, and share-card exports).
    pub fn save_bytes_with_dialog(
        &self,
        suggested_filename: &str,
        mime_type: &str,
        bytes: &[u8],
    ) -> Result<SaveDocumentResponse> {
        let picked = desktop_dialog::save_file_dialog(
            &self.app,
            "Save File As",
            suggested_filename,
            mime_type,
        )?;
        let Some(dest_path) = picked else {
            return Ok(SaveDocumentResponse {
                cancelled: true,
                saved_uri: None,
                bytes_written: 0,
            });
        };

        if let Some(parent) = dest_path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        fs::write(&dest_path, bytes)?;
        Ok(SaveDocumentResponse {
            cancelled: false,
            saved_uri: Some(dest_path.to_string_lossy().to_string()),
            bytes_written: bytes.len() as u64,
        })
    }

    pub fn authenticate_biometric(
        &self,
        _req: BiometricAuthRequest,
    ) -> Result<BiometricAuthResponse> {
        Ok(BiometricAuthResponse {
            authenticated: false,
            message: Some("BiometricPrompt is only available on Android devices".to_string()),
        })
    }

    pub fn check_biometric_available(&self) -> Result<BiometricAvailability> {
        Ok(BiometricAvailability {
            available: false,
            reason: Some("Desktop environment uses local PIN fallback".to_string()),
        })
    }

    pub fn create_memo_shortcut(
        &self,
        req: CreateMemoShortcutRequest,
    ) -> Result<CreateMemoShortcutResponse> {
        desktop_shortcut::create_desktop_memo_shortcut(&self.app, &req)
    }

    pub fn consume_pending_memo_shortcut(&self) -> Result<PendingMemoShortcutResponse> {
        let memo_uid = self
            .pending_shortcut_uid
            .lock()
            .ok()
            .and_then(|mut guard| guard.take());
        Ok(PendingMemoShortcutResponse { memo_uid })
    }
}
