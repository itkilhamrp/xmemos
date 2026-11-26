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
pub struct SystemSetting {
    pub name: String,
    pub value: String,
    pub description: String,
}

pub fn upsert_system_setting(
    conn: &Connection,
    name: &str,
    value: &str,
    description: &str,
) -> AppResult<SystemSetting> {
    conn.execute(
        "INSERT INTO system_setting (name, value, description) VALUES (?1, ?2, ?3)
         ON CONFLICT(name) DO UPDATE SET value = excluded.value, description = excluded.description",
        params![name, value, description],
    )?;
    Ok(SystemSetting {
        name: name.to_string(),
        value: value.to_string(),
        description: description.to_string(),
    })
}

pub fn get_system_setting(conn: &Connection, name: &str) -> AppResult<Option<SystemSetting>> {
    let row = conn
        .query_row(
            "SELECT name, value, description FROM system_setting WHERE name = ?1",
            params![name],
            |r| {
                Ok(SystemSetting {
                    name: r.get(0)?,
                    value: r.get(1)?,
                    description: r.get(2)?,
                })
            },
        )
        .optional()?;
    Ok(row)
}

pub fn list_system_settings(conn: &Connection) -> AppResult<Vec<SystemSetting>> {
    let mut stmt =
        conn.prepare("SELECT name, value, description FROM system_setting ORDER BY name ASC")?;
    let rows = stmt
        .query_map([], |r| {
            Ok(SystemSetting {
                name: r.get(0)?,
                value: r.get(1)?,
                description: r.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub const SETTING_MAX_UPLOAD_SIZE_MB: &str = "max-upload-size-mb";
pub const DEFAULT_MAX_UPLOAD_SIZE_MB: u64 = 32;
pub const MAX_ALLOWED_UPLOAD_SIZE_MB: u64 = 2048;

/// Reads the persisted `max-upload-size-mb` setting from `system_setting` (default: 32 MB, 0 = unlimited).
pub fn get_max_upload_size_mb(conn: &Connection) -> AppResult<u64> {
    let setting = get_system_setting(conn, SETTING_MAX_UPLOAD_SIZE_MB)?;
    match setting {
        Some(s) => Ok(s
            .value
            .trim()
            .parse::<u64>()
            .ok()
            .map(|v| v.min(MAX_ALLOWED_UPLOAD_SIZE_MB))
            .unwrap_or(DEFAULT_MAX_UPLOAD_SIZE_MB)),
        None => Ok(DEFAULT_MAX_UPLOAD_SIZE_MB),
    }
}
