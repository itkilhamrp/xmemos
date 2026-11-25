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

use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

pub use models::*;

#[cfg(desktop)]
mod desktop;
#[cfg(desktop)]
mod desktop_dialog;
#[cfg(desktop)]
mod desktop_shortcut;
#[cfg(mobile)]
mod mobile;

mod commands;
mod error;
mod models;

pub use error::{Error, Result};

#[cfg(desktop)]
pub use desktop::resolve_portable_data_dir;
#[cfg(desktop)]
use desktop::XmemosStorage;
#[cfg(mobile)]
use mobile::XmemosStorage;

pub trait XmemosStorageExt<R: Runtime> {
    fn xmemos_storage(&self) -> &XmemosStorage<R>;
}

impl<R: Runtime, T: Manager<R>> XmemosStorageExt<R> for T {
    fn xmemos_storage(&self) -> &XmemosStorage<R> {
        self.state::<XmemosStorage<R>>().inner()
    }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("xmemos-storage")
        .invoke_handler(tauri::generate_handler![
            commands::get_media_root,
            commands::set_nomedia,
            commands::get_nomedia_status,
            commands::pick_document,
            commands::save_document,
            commands::authenticate_biometric,
            commands::check_biometric_available,
            commands::create_memo_shortcut,
            commands::consume_pending_memo_shortcut,
        ])
        .setup(|app, api| {
            #[cfg(mobile)]
            let storage = mobile::init(app, api)?;
            #[cfg(desktop)]
            let storage = desktop::init(app, api)?;
            app.manage(storage);
            Ok(())
        })
        .build()
}
