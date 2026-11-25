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

use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

use crate::error::AppResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Activity {
    pub id: i64,
    pub creator_id: i64,
    pub created_ts: i64,
    pub r#type: String,
    pub level: String,
    pub payload: String,
}

pub fn create_activity(
    conn: &Connection,
    creator_id: i64,
    activity_type: &str,
    level: &str,
    payload_json: &str,
) -> AppResult<Activity> {
    let now = chrono::Utc::now().timestamp();
    conn.execute(
        "INSERT INTO activity (creator_id, created_ts, type, level, payload)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![creator_id, now, activity_type, level, payload_json],
    )?;
    let id = conn.last_insert_rowid();
    Ok(Activity {
        id,
        creator_id,
        created_ts: now,
        r#type: activity_type.to_string(),
        level: level.to_string(),
        payload: payload_json.to_string(),
    })
}
