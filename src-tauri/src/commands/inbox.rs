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

use serde::Serialize;
use tauri::State;

use crate::error::{AppError, AppResult};
use crate::service::auth::require_authenticated_user;
use crate::state::AppState;
use crate::store::inbox::{self, InboxItem};
use crate::store::user::{self, User};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InboxView {
    pub item: InboxItem,
    pub sender: Option<User>,
}

#[tauri::command]
pub fn inbox_list(state: State<'_, AppState>) -> AppResult<Vec<InboxView>> {
    let current = require_authenticated_user(&state)?;
    state.with_memos_db(|conn| {
        let items = inbox::list_inbox(conn, current.id)?;
        let mut views = Vec::with_capacity(items.len());
        for item in items {
            let sender = user::get_user_by_id(conn, item.sender_id)?;
            views.push(InboxView { item, sender });
        }
        Ok(views)
    })
}

#[tauri::command]
pub fn inbox_update_status(state: State<'_, AppState>, id: i64, status: String) -> AppResult<()> {
    let current = require_authenticated_user(&state)?;
    if status != "UNREAD" && status != "ARCHIVED" {
        return Err(AppError::Validation(format!(
            "Invalid inbox status: {status}"
        )));
    }
    state.with_memos_db(|conn| inbox::update_inbox_status(conn, id, current.id, &status))
}

#[tauri::command]
pub fn inbox_delete(state: State<'_, AppState>, id: i64) -> AppResult<()> {
    let current = require_authenticated_user(&state)?;
    state.with_memos_db(|conn| inbox::delete_inbox(conn, id, current.id))
}
