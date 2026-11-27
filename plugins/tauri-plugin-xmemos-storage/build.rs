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

const COMMANDS: &[&str] = &[
    "get_media_root",
    "set_nomedia",
    "get_nomedia_status",
    "pick_document",
    "save_document",
    "authenticate_biometric",
    "check_biometric_available",
    "create_memo_shortcut",
    "update_memo_shortcut_color",
    "consume_pending_memo_shortcut",
];

fn main() {
    tauri_plugin::Builder::new(COMMANDS)
        .android_path("android")
        .build();
}
