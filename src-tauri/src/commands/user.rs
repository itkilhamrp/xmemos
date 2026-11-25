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

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::error::{AppError, AppResult};
use crate::service::auth::{
    hash_password, normalize_username, require_admin_or_host, require_authenticated_user,
    validate_username,
};
use crate::state::AppState;
use crate::store::memo::{self, MemoFilter};
use crate::store::user::{self, CreateUserInput, UpdateUserInput, User};
use crate::store::user_setting::{self, UserSetting};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdminCreateUserPayload {
    pub username: String,
    pub password: String,
    pub role: String,
    pub nickname: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProfilePayload {
    pub user_id: Option<i64>,
    pub username: Option<String>,
    pub nickname: Option<String>,
    pub email: Option<String>,
    pub description: Option<String>,
    pub avatar_url: Option<String>,
    pub password: Option<String>,
    pub role: Option<String>,
    pub row_status: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserProfileStats {
    pub user: User,
    pub memo_count: usize,
    pub tag_count: usize,
    pub active_days: usize,
}

#[tauri::command]
pub fn user_list(state: State<'_, AppState>, include_archived: Option<bool>) -> AppResult<Vec<User>> {
    state.with_memos_db(|conn| user::list_users(conn, include_archived.unwrap_or(false)))
}

#[tauri::command]
pub fn user_get_profile(state: State<'_, AppState>, username: String) -> AppResult<UserProfileStats> {
    let viewer_id = state.current_user_id()?;
    state.with_memos_db(|conn| {
        let normalized = normalize_username(&username);
        let u = user::get_user_by_username(conn, &normalized)?
            .ok_or_else(|| AppError::NotFound(format!("User '@{username}' not found")))?;
        let memos = memo::list_memos(
            conn,
            viewer_id,
            &MemoFilter {
                creator_id: Some(u.id),
                row_status: Some("NORMAL".to_string()),
                exclude_comments: true,
                limit: 50_000,
                ..Default::default()
            },
        )?;
        let mut tags = std::collections::HashSet::new();
        let mut days = std::collections::HashSet::new();
        for m in &memos {
            for t in &m.payload.tags {
                tags.insert(t.clone());
            }
            if let Some(dt) = chrono::DateTime::from_timestamp(m.created_ts, 0) {
                days.insert(dt.format("%Y-%m-%d").to_string());
            }
        }
        Ok(UserProfileStats {
            user: u,
            memo_count: memos.len(),
            tag_count: tags.len(),
            active_days: days.len(),
        })
    })
}

#[tauri::command]
pub fn user_admin_create(
    state: State<'_, AppState>,
    payload: AdminCreateUserPayload,
) -> AppResult<User> {
    let actor = require_admin_or_host(&state)?;
    let normalized_username = normalize_username(&payload.username);
    validate_username(&normalized_username)?;

    if payload.role != "USER" && payload.role != "ADMIN" {
        return Err(AppError::Validation(
            "Role must be USER or ADMIN".to_string(),
        ));
    }
    if payload.role == "ADMIN" && actor.role != "HOST" {
        return Err(AppError::Forbidden(
            "Only HOST can create ADMIN accounts".to_string(),
        ));
    }

    let password_hash = hash_password(&payload.password)?;
    state.with_memos_db(|conn| {
        if user::get_user_by_username(conn, &normalized_username)?.is_some() {
            return Err(AppError::Validation("Username already exists".to_string()));
        }
        user::create_user(
            conn,
            &CreateUserInput {
                username: normalized_username.clone(),
                role: payload.role,
                email: payload.email.unwrap_or_default(),
                nickname: payload
                    .nickname
                    .filter(|s| !s.trim().is_empty())
                    .unwrap_or_else(|| normalized_username.clone()),
                password_hash,
                avatar_url: String::new(),
                description: String::new(),
            },
        )
    })
}

#[tauri::command]
pub fn user_update(state: State<'_, AppState>, payload: UpdateProfilePayload) -> AppResult<User> {
    let actor = require_authenticated_user(&state)?;
    let target_id = payload.user_id.unwrap_or(actor.id);

    if target_id != actor.id && actor.role != "HOST" && actor.role != "ADMIN" {
        return Err(AppError::Forbidden(
            "You cannot modify another user's account".to_string(),
        ));
    }

    let normalized_username = payload.username.as_deref().map(normalize_username);
    if let Some(ref uname) = normalized_username {
        validate_username(uname)?;
    }

    let password_hash = match payload.password {
        Some(ref pw) if !pw.is_empty() => Some(hash_password(pw)?),
        _ => None,
    };

    let allowed_role = if actor.role == "HOST" {
        payload.role
    } else {
        None
    };

    state.with_memos_db(|conn| {
        if let Some(ref uname) = normalized_username {
            if let Some(existing) = user::get_user_by_username(conn, uname)? {
                if existing.id != target_id {
                    return Err(AppError::Validation("Username already exists".to_string()));
                }
            }
        }
        user::update_user(
            conn,
            &UpdateUserInput {
                id: target_id,
                username: normalized_username,
                role: allowed_role,
                email: payload.email,
                nickname: payload.nickname,
                password_hash,
                avatar_url: payload.avatar_url,
                description: payload.description,
                row_status: if actor.role == "HOST" || actor.role == "ADMIN" {
                    payload.row_status
                } else {
                    None
                },
            },
        )
    })
}

#[tauri::command]
pub fn user_delete(state: State<'_, AppState>, user_id: i64) -> AppResult<()> {
    let actor = require_admin_or_host(&state)?;
    if actor.id == user_id {
        return Err(AppError::Validation(
            "You cannot delete your own active account".to_string(),
        ));
    }
    state.with_memos_db(|conn| {
        let target = user::get_user_by_id(conn, user_id)?
            .ok_or_else(|| AppError::NotFound(format!("User {user_id} not found")))?;
        if target.role == "HOST" {
            return Err(AppError::Forbidden(
                "HOST account cannot be deleted".to_string(),
            ));
        }
        user::delete_user(conn, user_id)
    })
}

#[tauri::command]
pub fn user_list_settings(state: State<'_, AppState>) -> AppResult<Vec<UserSetting>> {
    let current = require_authenticated_user(&state)?;
    state.with_memos_db(|conn| user_setting::list_user_settings(conn, current.id))
}

#[tauri::command]
pub fn user_upsert_setting(
    state: State<'_, AppState>,
    key: String,
    value: String,
) -> AppResult<UserSetting> {
    let current = require_authenticated_user(&state)?;
    state.with_memos_db(|conn| user_setting::upsert_user_setting(conn, current.id, &key, &value))
}
