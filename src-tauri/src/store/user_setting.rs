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

use crate::error::AppResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSetting {
    pub user_id: i64,
    pub key: String,
    pub value: String,
}

pub fn upsert_user_setting(
    conn: &Connection,
    user_id: i64,
    key: &str,
    value: &str,
) -> AppResult<UserSetting> {
    conn.execute(
        "INSERT INTO user_setting (user_id, key, value) VALUES (?1, ?2, ?3)
         ON CONFLICT(user_id, key) DO UPDATE SET value = excluded.value",
        params![user_id, key, value],
    )?;
    Ok(UserSetting {
        user_id,
        key: key.to_string(),
        value: value.to_string(),
    })
}

pub fn get_user_setting(
    conn: &Connection,
    user_id: i64,
    key: &str,
) -> AppResult<Option<UserSetting>> {
    let row = conn
        .query_row(
            "SELECT user_id, key, value FROM user_setting WHERE user_id = ?1 AND key = ?2",
            params![user_id, key],
            |r| {
                Ok(UserSetting {
                    user_id: r.get(0)?,
                    key: r.get(1)?,
                    value: r.get(2)?,
                })
            },
        )
        .optional()?;
    Ok(row)
}

pub fn list_user_settings(conn: &Connection, user_id: i64) -> AppResult<Vec<UserSetting>> {
    let mut stmt =
        conn.prepare("SELECT user_id, key, value FROM user_setting WHERE user_id = ?1")?;
    let rows = stmt
        .query_map(params![user_id], |r| {
            Ok(UserSetting {
                user_id: r.get(0)?,
                key: r.get(1)?,
                value: r.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}
