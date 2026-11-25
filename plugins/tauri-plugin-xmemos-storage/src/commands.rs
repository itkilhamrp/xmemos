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

use tauri::{AppHandle, Runtime};

use crate::error::Result;
use crate::models::*;
use crate::XmemosStorageExt;

#[tauri::command]
pub(crate) async fn get_media_root<R: Runtime>(app: AppHandle<R>) -> Result<MediaRootInfo> {
    app.xmemos_storage().get_media_root()
}

#[tauri::command]
pub(crate) async fn set_nomedia<R: Runtime>(
    app: AppHandle<R>,
    payload: NomediaRequest,
) -> Result<NomediaStatus> {
    app.xmemos_storage().set_nomedia(payload)
}

#[tauri::command]
pub(crate) async fn get_nomedia_status<R: Runtime>(app: AppHandle<R>) -> Result<NomediaStatus> {
    app.xmemos_storage().get_nomedia_status()
}

#[tauri::command]
pub(crate) async fn pick_document<R: Runtime>(
    app: AppHandle<R>,
    payload: PickDocumentRequest,
) -> Result<PickDocumentResponse> {
    app.xmemos_storage().pick_document(payload)
}

#[tauri::command]
pub(crate) async fn save_document<R: Runtime>(
    app: AppHandle<R>,
    payload: SaveDocumentRequest,
) -> Result<SaveDocumentResponse> {
    app.xmemos_storage().save_document(payload)
}

#[tauri::command]
pub(crate) async fn authenticate_biometric<R: Runtime>(
    app: AppHandle<R>,
    payload: BiometricAuthRequest,
) -> Result<BiometricAuthResponse> {
    app.xmemos_storage().authenticate_biometric(payload)
}

#[tauri::command]
pub(crate) async fn check_biometric_available<R: Runtime>(
    app: AppHandle<R>,
) -> Result<BiometricAvailability> {
    app.xmemos_storage().check_biometric_available()
}

#[tauri::command]
pub(crate) async fn create_memo_shortcut<R: Runtime>(
    app: AppHandle<R>,
    payload: CreateMemoShortcutRequest,
) -> Result<CreateMemoShortcutResponse> {
    app.xmemos_storage().create_memo_shortcut(payload)
}

#[tauri::command]
pub(crate) async fn consume_pending_memo_shortcut<R: Runtime>(
    app: AppHandle<R>,
) -> Result<PendingMemoShortcutResponse> {
    app.xmemos_storage().consume_pending_memo_shortcut()
}
