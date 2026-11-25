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
pub struct InboxItem {
    pub id: i64,
    pub created_ts: i64,
    pub sender_id: i64,
    pub receiver_id: i64,
    pub status: String,
    pub message: String,
}

pub fn create_inbox(
    conn: &Connection,
    sender_id: i64,
    receiver_id: i64,
    message_json: &str,
) -> AppResult<InboxItem> {
    let now = chrono::Utc::now().timestamp();
    conn.execute(
        "INSERT INTO inbox (created_ts, sender_id, receiver_id, status, message)
         VALUES (?1, ?2, ?3, 'UNREAD', ?4)",
        params![now, sender_id, receiver_id, message_json],
    )?;
    let id = conn.last_insert_rowid();
    Ok(InboxItem {
        id,
        created_ts: now,
        sender_id,
        receiver_id,
        status: "UNREAD".to_string(),
        message: message_json.to_string(),
    })
}

pub fn list_inbox(conn: &Connection, receiver_id: i64) -> AppResult<Vec<InboxItem>> {
    let mut stmt = conn.prepare(
        "SELECT id, created_ts, sender_id, receiver_id, status, message
         FROM inbox WHERE receiver_id = ?1 ORDER BY created_ts DESC, id DESC",
    )?;
    let rows = stmt
        .query_map(params![receiver_id], |r| {
            Ok(InboxItem {
                id: r.get(0)?,
                created_ts: r.get(1)?,
                sender_id: r.get(2)?,
                receiver_id: r.get(3)?,
                status: r.get(4)?,
                message: r.get(5)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn update_inbox_status(
    conn: &Connection,
    id: i64,
    receiver_id: i64,
    status: &str,
) -> AppResult<()> {
    conn.execute(
        "UPDATE inbox SET status = ?1 WHERE id = ?2 AND receiver_id = ?3",
        params![status, id, receiver_id],
    )?;
    Ok(())
}

pub fn delete_inbox(conn: &Connection, id: i64, receiver_id: i64) -> AppResult<()> {
    conn.execute(
        "DELETE FROM inbox WHERE id = ?1 AND receiver_id = ?2",
        params![id, receiver_id],
    )?;
    Ok(())
}
