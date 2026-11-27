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

use std::path::Path;

use tauri::State;

use crate::error::{AppError, AppResult};
use crate::service::auth::require_authenticated_user;
use crate::service::backup::{
    self, BackupHistoryEntry, BackupScope, ExportResult, ImportPreview, ImportResult,
};
use crate::state::AppState;
use crate::store::user;

fn resolve_backup_scope(state: &AppState, requested_scope: Option<String>) -> AppResult<BackupScope> {
    let current = require_authenticated_user(state)?;
    let is_admin = current.role == "HOST" || current.role == "ADMIN";
    let normalized = requested_scope
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_ascii_uppercase);

    match normalized.as_deref() {
        Some("ALL_USERS") => {
            if !is_admin {
                return Err(AppError::Forbidden(
                    "Only HOST or ADMIN accounts can export the full workspace database and all users' media."
                        .to_string(),
                ));
            }
            Ok(BackupScope::AllUsers)
        }
        Some("ACTIVE_USER") => Ok(BackupScope::ActiveUser {
            user_id: current.id,
            username: current.username,
        }),
        None => {
            if is_admin {
                Ok(BackupScope::AllUsers)
            } else {
                Ok(BackupScope::ActiveUser {
                    user_id: current.id,
                    username: current.username,
                })
            }
        }
        Some(other) => Err(AppError::Validation(format!(
            "Unsupported backup scope '{other}'. Expected 'ALL_USERS' or 'ACTIVE_USER'."
        ))),
    }
}

fn require_admin_if_initialized(state: &AppState) -> AppResult<()> {
    let user_count = state.with_memos_db(user::count_users)?;
    if user_count == 0 {
        return Ok(());
    }
    let current = require_authenticated_user(state)?;
    if current.role != "HOST" && current.role != "ADMIN" {
        return Err(AppError::Forbidden(
            "Only HOST or ADMIN accounts can restore a workspace backup.".to_string(),
        ));
    }
    Ok(())
}

#[tauri::command]
pub fn backup_export_db(
    state: State<'_, AppState>,
    scope: Option<String>,
) -> AppResult<ExportResult> {
    let resolved = resolve_backup_scope(&state, scope)?;
    backup::export_database_only_staged(&state, &resolved)
}

#[tauri::command]
pub fn backup_export_full(
    state: State<'_, AppState>,
    scope: Option<String>,
) -> AppResult<ExportResult> {
    let resolved = resolve_backup_scope(&state, scope)?;
    backup::export_full_bundle_staged(&state, &resolved)
}

#[tauri::command]
pub fn backup_finalize_export(
    state: State<'_, AppState>,
    staged_path: String,
    saved_destination: Option<String>,
    result: Option<ExportResult>,
) -> AppResult<()> {
    let _ = require_authenticated_user(&state)?;
    backup::finalize_staged_export(
        &state,
        Path::new(&staged_path),
        saved_destination.as_deref(),
        result.as_ref(),
    )
}

#[tauri::command]
pub fn backup_list_history(
    state: State<'_, AppState>,
    limit: Option<u32>,
    offset: Option<u32>,
) -> AppResult<Vec<BackupHistoryEntry>> {
    let _ = require_authenticated_user(&state)?;
    backup::list_backup_history(&state, limit, offset)
}

#[tauri::command]
pub fn backup_preview_import(
    state: State<'_, AppState>,
    source_path: String,
) -> AppResult<ImportPreview> {
    require_admin_if_initialized(&state)?;
    backup::preview_import(&state, Path::new(&source_path))
}

#[tauri::command]
pub fn backup_execute_import(
    state: State<'_, AppState>,
    source_path: String,
) -> AppResult<ImportResult> {
    require_admin_if_initialized(&state)?;
    backup::execute_atomic_import(&state, Path::new(&source_path))
}
