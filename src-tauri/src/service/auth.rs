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

use bcrypt::{hash, verify, DEFAULT_COST};
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};
use crate::state::AppState;
use crate::store::user::{self, User};

const BCRYPT_COST: u32 = 10; // Matches official Memos Go bcrypt.DefaultCost (10)
pub const PREF_REQUIRE_PASSWORD_AFTER_SIGN_OUT: &str = "require_password_after_sign_out";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserLockConfig {
    pub user_id: i64,
    pub has_pin: bool,
    pub biometric_enabled: bool,
    pub require_unlock_on_switch: bool,
}

pub fn hash_password(password: &str) -> AppResult<String> {
    if password.len() < 4 {
        return Err(AppError::Validation(
            "Password must be at least 4 characters long".to_string(),
        ));
    }
    let cost = BCRYPT_COST.min(DEFAULT_COST);
    Ok(hash(password, cost)?)
}

pub fn verify_password(password: &str, password_hash: &str) -> AppResult<bool> {
    Ok(verify(password, password_hash)?)
}

pub fn require_authenticated_user(state: &AppState) -> AppResult<User> {
    let uid = state
        .current_user_id()?
        .ok_or_else(|| AppError::Unauthorized("Authentication required".to_string()))?;
    state.with_memos_db(|conn| {
        let user = user::get_user_by_id(conn, uid)?
            .ok_or_else(|| AppError::Unauthorized("Active user no longer exists".to_string()))?;
        if user.row_status != "NORMAL" {
            return Err(AppError::Forbidden("Account is archived".to_string()));
        }
        Ok(user)
    })
}

pub fn require_admin_or_host(state: &AppState) -> AppResult<User> {
    let user = require_authenticated_user(state)?;
    if user.role != "HOST" && user.role != "ADMIN" {
        return Err(AppError::Forbidden(
            "HOST or ADMIN privileges required".to_string(),
        ));
    }
    Ok(user)
}

pub fn require_host(state: &AppState) -> AppResult<User> {
    let user = require_authenticated_user(state)?;
    if user.role != "HOST" {
        return Err(AppError::Forbidden("HOST privileges required".to_string()));
    }
    Ok(user)
}

pub fn normalize_username(username: &str) -> String {
    username.trim().to_ascii_lowercase()
}

pub fn validate_username(username: &str) -> AppResult<()> {
    let trimmed = username.trim();
    if trimmed.len() < 2 || trimmed.len() > 32 {
        return Err(AppError::Validation(
            "Username must be between 2 and 32 characters".to_string(),
        ));
    }
    if !trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err(AppError::Validation(
            "Username may only contain letters, numbers, underscores, and hyphens".to_string(),
        ));
    }
    Ok(())
}

pub fn get_require_password_after_sign_out(state: &AppState) -> AppResult<bool> {
    let local = state.lock_local_db()?;
    let val: Option<String> = local
        .query_row(
            "SELECT value FROM local_pref WHERE key = ?1",
            params![PREF_REQUIRE_PASSWORD_AFTER_SIGN_OUT],
            |r| r.get(0),
        )
        .optional()?;
    Ok(val.as_deref() == Some("true"))
}

pub fn get_user_lock_config(state: &AppState, user_id: i64) -> AppResult<UserLockConfig> {
    let local = state.lock_local_db()?;
    let row = local
        .query_row(
            "SELECT pin_hash, biometric_enabled, require_unlock_on_switch FROM user_lock WHERE user_id = ?1",
            params![user_id],
            |r| {
                let pin_hash: Option<String> = r.get(0)?;
                let bio: i64 = r.get(1)?;
                let req_switch: i64 = r.get(2)?;
                Ok(UserLockConfig {
                    user_id,
                    has_pin: pin_hash.as_deref().is_some_and(|p| !p.is_empty()),
                    biometric_enabled: bio != 0,
                    require_unlock_on_switch: req_switch != 0,
                })
            },
        )
        .optional()?;
    Ok(row.unwrap_or(UserLockConfig {
        user_id,
        has_pin: false,
        biometric_enabled: false,
        require_unlock_on_switch: false,
    }))
}

pub fn set_user_lock_config(
    state: &AppState,
    user_id: i64,
    pin: Option<&str>,
    clear_pin: bool,
    biometric_enabled: bool,
    require_unlock_on_switch: bool,
) -> AppResult<UserLockConfig> {
    let local = state.lock_local_db()?;
    let existing_hash: Option<String> = local
        .query_row(
            "SELECT pin_hash FROM user_lock WHERE user_id = ?1",
            params![user_id],
            |r| r.get(0),
        )
        .optional()?
        .flatten();

    let next_hash = if clear_pin {
        None
    } else if let Some(new_pin) = pin {
        if new_pin.len() != 6 || !new_pin.bytes().all(|b| b.is_ascii_digit()) {
            return Err(AppError::Validation(
                "Local app lock PIN must be exactly 6 numeric digits (0-9)".to_string(),
            ));
        }
        let cost = BCRYPT_COST.min(DEFAULT_COST);
        Some(hash(new_pin, cost)?)
    } else {
        existing_hash
    };

    let now = chrono::Utc::now().timestamp();
    local.execute(
        "INSERT INTO user_lock (user_id, pin_hash, biometric_enabled, require_unlock_on_switch, updated_ts)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(user_id) DO UPDATE SET
           pin_hash = excluded.pin_hash,
           biometric_enabled = excluded.biometric_enabled,
           require_unlock_on_switch = excluded.require_unlock_on_switch,
           updated_ts = excluded.updated_ts",
        params![
            user_id,
            next_hash,
            if biometric_enabled { 1 } else { 0 },
            if require_unlock_on_switch { 1 } else { 0 },
            now
        ],
    )?;

    Ok(UserLockConfig {
        user_id,
        has_pin: next_hash.is_some(),
        biometric_enabled,
        require_unlock_on_switch,
    })
}

pub fn verify_user_pin(state: &AppState, user_id: i64, pin: &str) -> AppResult<bool> {
    if pin.len() != 6 || !pin.bytes().all(|b| b.is_ascii_digit()) {
        return Ok(false);
    }
    let local = state.lock_local_db()?;
    let pin_hash: Option<String> = local
        .query_row(
            "SELECT pin_hash FROM user_lock WHERE user_id = ?1",
            params![user_id],
            |r| r.get(0),
        )
        .optional()?
        .flatten();
    match pin_hash {
        Some(h) => Ok(verify(pin, &h)?),
        None => Ok(false),
    }
}
