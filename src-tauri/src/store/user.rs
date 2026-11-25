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

use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: i64,
    pub created_ts: i64,
    pub updated_ts: i64,
    pub row_status: String,
    pub username: String,
    pub role: String,
    pub email: String,
    pub nickname: String,
    pub avatar_url: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct UserWithHash {
    pub user: User,
    pub password_hash: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserInput {
    pub username: String,
    pub role: String,
    pub email: String,
    pub nickname: String,
    pub password_hash: String,
    pub avatar_url: String,
    pub description: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserInput {
    pub id: i64,
    pub username: Option<String>,
    pub role: Option<String>,
    pub email: Option<String>,
    pub nickname: Option<String>,
    pub password_hash: Option<String>,
    pub avatar_url: Option<String>,
    pub description: Option<String>,
    pub row_status: Option<String>,
}

fn map_user(row: &rusqlite::Row<'_>) -> rusqlite::Result<User> {
    Ok(User {
        id: row.get(0)?,
        created_ts: row.get(1)?,
        updated_ts: row.get(2)?,
        row_status: row.get(3)?,
        username: row.get(4)?,
        role: row.get(5)?,
        email: row.get(6)?,
        nickname: row.get(7)?,
        avatar_url: row.get(8)?,
        description: row.get(9)?,
    })
}

pub fn count_users(conn: &Connection) -> AppResult<i64> {
    let count: i64 = conn.query_row("SELECT COUNT(1) FROM user", [], |r| r.get(0))?;
    Ok(count)
}

pub fn create_user(conn: &Connection, input: &CreateUserInput) -> AppResult<User> {
    let now = chrono::Utc::now().timestamp();
    conn.execute(
        "INSERT INTO user (created_ts, updated_ts, row_status, username, role, email, nickname, password_hash, avatar_url, description)
         VALUES (?1, ?2, 'NORMAL', ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            now,
            now,
            input.username,
            input.role,
            input.email,
            input.nickname,
            input.password_hash,
            input.avatar_url,
            input.description
        ],
    )?;
    let id = conn.last_insert_rowid();
    get_user_by_id(conn, id)?
        .ok_or_else(|| AppError::Internal("Failed to fetch newly created user".to_string()))
}

pub fn get_user_by_id(conn: &Connection, id: i64) -> AppResult<Option<User>> {
    let user = conn
        .query_row(
            "SELECT id, created_ts, updated_ts, row_status, username, role, email, nickname, avatar_url, description
             FROM user WHERE id = ?1",
            params![id],
            map_user,
        )
        .optional()?;
    Ok(user)
}

pub fn get_user_by_username(conn: &Connection, username: &str) -> AppResult<Option<User>> {
    let user = conn
        .query_row(
            "SELECT id, created_ts, updated_ts, row_status, username, role, email, nickname, avatar_url, description
             FROM user WHERE username = ?1",
            params![username],
            map_user,
        )
        .optional()?;
    Ok(user)
}

pub fn get_user_with_hash_by_username(
    conn: &Connection,
    username: &str,
) -> AppResult<Option<UserWithHash>> {
    let row = conn
        .query_row(
            "SELECT id, created_ts, updated_ts, row_status, username, role, email, nickname, avatar_url, description, password_hash
             FROM user WHERE username = ?1",
            params![username],
            |r| {
                Ok(UserWithHash {
                    user: map_user(r)?,
                    password_hash: r.get(10)?,
                })
            },
        )
        .optional()?;
    Ok(row)
}

pub fn get_user_with_hash_by_id(conn: &Connection, id: i64) -> AppResult<Option<UserWithHash>> {
    let row = conn
        .query_row(
            "SELECT id, created_ts, updated_ts, row_status, username, role, email, nickname, avatar_url, description, password_hash
             FROM user WHERE id = ?1",
            params![id],
            |r| {
                Ok(UserWithHash {
                    user: map_user(r)?,
                    password_hash: r.get(10)?,
                })
            },
        )
        .optional()?;
    Ok(row)
}

pub fn list_users(conn: &Connection, include_archived: bool) -> AppResult<Vec<User>> {
    let sql = if include_archived {
        "SELECT id, created_ts, updated_ts, row_status, username, role, email, nickname, avatar_url, description
         FROM user ORDER BY id ASC"
    } else {
        "SELECT id, created_ts, updated_ts, row_status, username, role, email, nickname, avatar_url, description
         FROM user WHERE row_status = 'NORMAL' ORDER BY id ASC"
    };
    let mut stmt = conn.prepare(sql)?;
    let rows = stmt
        .query_map([], map_user)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn update_user(conn: &Connection, input: &UpdateUserInput) -> AppResult<User> {
    let existing = get_user_with_hash_by_id(conn, input.id)?
        .ok_or_else(|| AppError::NotFound(format!("User {} not found", input.id)))?;

    let now = chrono::Utc::now().timestamp();
    let username = input.username.as_deref().unwrap_or(&existing.user.username);
    let role = input.role.as_deref().unwrap_or(&existing.user.role);
    let email = input.email.as_deref().unwrap_or(&existing.user.email);
    let nickname = input.nickname.as_deref().unwrap_or(&existing.user.nickname);
    let password_hash = input
        .password_hash
        .as_deref()
        .unwrap_or(&existing.password_hash);
    let avatar_url = input
        .avatar_url
        .as_deref()
        .unwrap_or(&existing.user.avatar_url);
    let description = input
        .description
        .as_deref()
        .unwrap_or(&existing.user.description);
    let row_status = input
        .row_status
        .as_deref()
        .unwrap_or(&existing.user.row_status);

    conn.execute(
        "UPDATE user SET updated_ts = ?1, username = ?2, role = ?3, email = ?4, nickname = ?5,
         password_hash = ?6, avatar_url = ?7, description = ?8, row_status = ?9
         WHERE id = ?10",
        params![
            now,
            username,
            role,
            email,
            nickname,
            password_hash,
            avatar_url,
            description,
            row_status,
            input.id
        ],
    )?;

    get_user_by_id(conn, input.id)?
        .ok_or_else(|| AppError::NotFound(format!("User {} not found after update", input.id)))
}

pub fn delete_user(conn: &Connection, id: i64) -> AppResult<()> {
    conn.execute("DELETE FROM user_setting WHERE user_id = ?1", params![id])?;
    conn.execute("DELETE FROM user WHERE id = ?1", params![id])?;
    Ok(())
}
