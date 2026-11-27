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
    self, get_require_password_after_sign_out, get_user_lock_config, hash_password,
    normalize_username, require_admin_or_host, require_authenticated_user, set_user_lock_config,
    validate_username,
    verify_password,
    verify_user_pin, UserLockConfig,
};
use crate::state::AppState;
use crate::store::system_setting;
use crate::store::user::{self, CreateUserInput, User};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthStatus {
    pub has_host: bool,
    pub disallow_user_registration: bool,
    pub current_user: Option<User>,
    pub lock_config: Option<UserLockConfig>,
    pub local_users: Vec<User>,
    pub require_password_after_sign_out: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignUpPayload {
    pub username: String,
    pub password: String,
    pub nickname: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignInPayload {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SwitchAccountPayload {
    pub user_id: i64,
    pub password: Option<String>,
    pub pin: Option<String>,
    pub biometric_verified: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetLockPayload {
    pub pin: Option<String>,
    pub clear_pin: bool,
    pub biometric_enabled: bool,
    pub require_unlock_on_switch: bool,
}

#[tauri::command]
pub fn auth_get_status(state: State<'_, AppState>) -> AppResult<AuthStatus> {
    let (has_host, disallow_reg, local_users) = state.with_memos_db(|conn| {
        let count = user::count_users(conn)?;
        let disallow = system_setting::get_system_setting(conn, "disallow-user-registration")?
            .is_some_and(|s| s.value == "true");
        let users = user::list_users(conn, false)?;
        Ok((count > 0, disallow, users))
    })?;

    let current_user = match state.current_user_id()? {
        Some(uid) => state.with_memos_db(|conn| user::get_user_by_id(conn, uid))?,
        None => None,
    };

    let lock_config = match &current_user {
        Some(u) => Some(get_user_lock_config(&state, u.id)?),
        None => None,
    };

    let require_password_after_sign_out = get_require_password_after_sign_out(&state)?;

    Ok(AuthStatus {
        has_host,
        disallow_user_registration: disallow_reg,
        current_user,
        lock_config,
        local_users,
        require_password_after_sign_out,
    })
}

#[tauri::command]
pub fn auth_setup_host(state: State<'_, AppState>, payload: SignUpPayload) -> AppResult<User> {
    let normalized_username = normalize_username(&payload.username);
    validate_username(&normalized_username)?;
    let password_hash = hash_password(&payload.password)?;

    let user = state.with_memos_db(|conn| {
        if user::count_users(conn)? > 0 {
            return Err(AppError::Forbidden(
                "Host account is already initialized".to_string(),
            ));
        }
        user::create_user(
            conn,
            &CreateUserInput {
                username: normalized_username.clone(),
                role: "HOST".to_string(),
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
    })?;

    state.set_current_user_id(Some(user.id))?;
    Ok(user)
}

#[tauri::command]
pub fn auth_sign_up(state: State<'_, AppState>, payload: SignUpPayload) -> AppResult<User> {
    let normalized_username = normalize_username(&payload.username);
    validate_username(&normalized_username)?;
    let password_hash = hash_password(&payload.password)?;

    let user = state.with_memos_db(|conn| {
        let existing_count = user::count_users(conn)?;
        let disallow = system_setting::get_system_setting(conn, "disallow-user-registration")?
            .is_some_and(|s| s.value == "true");
        if existing_count > 0 && disallow {
            return Err(AppError::Forbidden(
                "User registration is disabled by workspace host".to_string(),
            ));
        }
        if user::get_user_by_username(conn, &normalized_username)?.is_some() {
            return Err(AppError::Validation("Username already exists".to_string()));
        }

        let role = if existing_count == 0 { "HOST" } else { "USER" };
        user::create_user(
            conn,
            &CreateUserInput {
                username: normalized_username.clone(),
                role: role.to_string(),
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
    })?;

    state.set_current_user_id(Some(user.id))?;
    Ok(user)
}

#[tauri::command]
pub fn auth_sign_in(state: State<'_, AppState>, payload: SignInPayload) -> AppResult<User> {
    let record = state
        .with_memos_db(|conn| user::get_user_with_hash_by_username(conn, &normalize_username(&payload.username)))?
        .ok_or_else(|| AppError::Unauthorized("Invalid username or password".to_string()))?;

    if record.user.row_status != "NORMAL" {
        return Err(AppError::Forbidden("This account is archived".to_string()));
    }

    if !verify_password(&payload.password, &record.password_hash)? {
        return Err(AppError::Unauthorized(
            "Invalid username or password".to_string(),
        ));
    }

    state.set_current_user_id(Some(record.user.id))?;
    Ok(record.user)
}

#[tauri::command]
pub fn auth_sign_out(state: State<'_, AppState>) -> AppResult<()> {
    state.set_current_user_id(None)
}

#[tauri::command]
pub fn auth_switch_account(
    state: State<'_, AppState>,
    payload: SwitchAccountPayload,
) -> AppResult<User> {
    let record = state
        .with_memos_db(|conn| user::get_user_with_hash_by_id(conn, payload.user_id))?
        .ok_or_else(|| AppError::NotFound("Target account not found".to_string()))?;

    if record.user.row_status != "NORMAL" {
        return Err(AppError::Forbidden("Target account is archived".to_string()));
    }

    let is_signed_out = state.current_user_id()?.is_none();
    let require_pw_after_sign_out = get_require_password_after_sign_out(&state)?;

    if is_signed_out && require_pw_after_sign_out {
        let verified = match payload.password.as_deref() {
            Some(pw) if !pw.is_empty() => verify_password(pw, &record.password_hash)?,
            _ => false,
        };
        if !verified {
            return Err(AppError::Unauthorized(
                "Password is required to sign back in after signing out".to_string(),
            ));
        }
    } else {
        let lock = get_user_lock_config(&state, record.user.id)?;
        if lock.require_unlock_on_switch {
            let mut unlocked = false;
            if payload.biometric_verified == Some(true) && lock.biometric_enabled {
                unlocked = true;
            }
            if !unlocked {
                if let Some(ref pin) = payload.pin {
                    if verify_user_pin(&state, record.user.id, pin)? {
                        unlocked = true;
                    }
                }
            }
            if !unlocked {
                if let Some(ref pw) = payload.password {
                    if verify_password(pw, &record.password_hash)? {
                        unlocked = true;
                    }
                }
            }
            if !unlocked {
                return Err(AppError::Unauthorized(
                    "Unlock verification (PIN, biometric, or password) required for this account"
                        .to_string(),
                ));
            }
        }
    }

    state.set_current_user_id(Some(record.user.id))?;
    Ok(record.user)
}

#[tauri::command]
pub fn auth_set_lock_config(
    state: State<'_, AppState>,
    payload: SetLockPayload,
) -> AppResult<UserLockConfig> {
    let current = require_admin_or_host(&state)?;
    set_user_lock_config(
        &state,
        current.id,
        payload.pin.as_deref(),
        payload.clear_pin,
        payload.biometric_enabled,
        payload.require_unlock_on_switch,
    )
}

#[tauri::command]
pub fn auth_verify_pin(state: State<'_, AppState>, pin: String) -> AppResult<bool> {
    let current = require_authenticated_user(&state)?;
    auth::verify_user_pin(&state, current.id, &pin)
}
