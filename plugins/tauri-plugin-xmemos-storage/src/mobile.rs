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

use serde::de::DeserializeOwned;
use tauri::{
    plugin::{PluginApi, PluginHandle},
    AppHandle, Runtime,
};

use crate::error::Result;
use crate::models::*;

#[cfg(target_os = "android")]
const PLUGIN_IDENTIFIER: &str = "org.xmemos.storage";

pub fn init<R: Runtime, C: DeserializeOwned>(
    _app: &AppHandle<R>,
    api: PluginApi<R, C>,
) -> Result<XmemosStorage<R>> {
    #[cfg(target_os = "android")]
    let handle = api.register_android_plugin(PLUGIN_IDENTIFIER, "XmemosStoragePlugin")?;
    Ok(XmemosStorage(handle))
}

pub struct XmemosStorage<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> XmemosStorage<R> {
    pub fn get_media_root(&self) -> Result<MediaRootInfo> {
        self.0
            .run_mobile_plugin("getMediaRoot", ())
            .map_err(Into::into)
    }

    pub fn set_nomedia(&self, req: NomediaRequest) -> Result<NomediaStatus> {
        self.0
            .run_mobile_plugin("setNomedia", req)
            .map_err(Into::into)
    }

    pub fn get_nomedia_status(&self) -> Result<NomediaStatus> {
        self.0
            .run_mobile_plugin("getNomediaStatus", ())
            .map_err(Into::into)
    }

    pub fn pick_document(&self, req: PickDocumentRequest) -> Result<PickDocumentResponse> {
        self.0
            .run_mobile_plugin("pickDocument", req)
            .map_err(Into::into)
    }

    pub fn save_document(&self, req: SaveDocumentRequest) -> Result<SaveDocumentResponse> {
        self.0
            .run_mobile_plugin("saveDocument", req)
            .map_err(Into::into)
    }

    pub fn authenticate_biometric(
        &self,
        req: BiometricAuthRequest,
    ) -> Result<BiometricAuthResponse> {
        self.0
            .run_mobile_plugin("authenticateBiometric", req)
            .map_err(Into::into)
    }

    pub fn check_biometric_available(&self) -> Result<BiometricAvailability> {
        self.0
            .run_mobile_plugin("checkBiometricAvailable", ())
            .map_err(Into::into)
    }

    pub fn create_memo_shortcut(
        &self,
        req: CreateMemoShortcutRequest,
    ) -> Result<CreateMemoShortcutResponse> {
        self.0
            .run_mobile_plugin("createMemoShortcut", req)
            .map_err(Into::into)
    }

    pub fn consume_pending_memo_shortcut(&self) -> Result<PendingMemoShortcutResponse> {
        self.0
            .run_mobile_plugin("consumePendingMemoShortcut", ())
            .map_err(Into::into)
    }

    pub fn save_to_downloads(
        &self,
        req: SaveToDownloadsRequest,
    ) -> Result<SaveToDownloadsResponse> {
        self.0
            .run_mobile_plugin("saveToDownloads", req)
            .map_err(Into::into)
    }
}
