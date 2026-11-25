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
pub struct Reaction {
    pub id: i64,
    pub created_ts: i64,
    pub creator_id: i64,
    pub content_id: String,
    pub reaction_type: String,
}

pub fn toggle_reaction(
    conn: &Connection,
    creator_id: i64,
    content_id: &str,
    reaction_type: &str,
) -> AppResult<(bool, Vec<Reaction>)> {
    let existing: Option<i64> = conn
        .query_row(
            "SELECT id FROM reaction WHERE creator_id = ?1 AND content_id = ?2 AND reaction_type = ?3",
            params![creator_id, content_id, reaction_type],
            |r| r.get(0),
        )
        .optional()?;

    let added = if let Some(id) = existing {
        conn.execute("DELETE FROM reaction WHERE id = ?1", params![id])?;
        false
    } else {
        let now = chrono::Utc::now().timestamp();
        conn.execute(
            "INSERT INTO reaction (created_ts, creator_id, content_id, reaction_type) VALUES (?1, ?2, ?3, ?4)",
            params![now, creator_id, content_id, reaction_type],
        )?;
        true
    };

    let list = list_reactions_for_content(conn, content_id)?;
    Ok((added, list))
}

pub fn list_reactions_for_content(conn: &Connection, content_id: &str) -> AppResult<Vec<Reaction>> {
    let mut stmt = conn.prepare(
        "SELECT id, created_ts, creator_id, content_id, reaction_type
         FROM reaction WHERE content_id = ?1 ORDER BY id ASC",
    )?;
    let rows = stmt
        .query_map(params![content_id], |r| {
            Ok(Reaction {
                id: r.get(0)?,
                created_ts: r.get(1)?,
                creator_id: r.get(2)?,
                content_id: r.get(3)?,
                reaction_type: r.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}
