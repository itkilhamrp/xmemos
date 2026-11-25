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
pub struct MemoRelation {
    pub memo_id: i64,
    pub related_memo_id: i64,
    pub r#type: String,
}

pub fn upsert_relation(
    conn: &Connection,
    memo_id: i64,
    related_memo_id: i64,
    rel_type: &str,
) -> AppResult<MemoRelation> {
    conn.execute(
        "INSERT OR IGNORE INTO memo_relation (memo_id, related_memo_id, type) VALUES (?1, ?2, ?3)",
        params![memo_id, related_memo_id, rel_type],
    )?;
    Ok(MemoRelation {
        memo_id,
        related_memo_id,
        r#type: rel_type.to_string(),
    })
}

pub fn delete_relation(
    conn: &Connection,
    memo_id: i64,
    related_memo_id: i64,
    rel_type: &str,
) -> AppResult<()> {
    conn.execute(
        "DELETE FROM memo_relation WHERE memo_id = ?1 AND related_memo_id = ?2 AND type = ?3",
        params![memo_id, related_memo_id, rel_type],
    )?;
    Ok(())
}

pub fn list_relations_for_memo(conn: &Connection, memo_id: i64) -> AppResult<Vec<MemoRelation>> {
    let mut stmt = conn.prepare(
        "SELECT memo_id, related_memo_id, type FROM memo_relation
         WHERE memo_id = ?1 OR related_memo_id = ?1",
    )?;
    let rows = stmt
        .query_map(params![memo_id], |r| {
            Ok(MemoRelation {
                memo_id: r.get(0)?,
                related_memo_id: r.get(1)?,
                r#type: r.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn list_comment_ids(conn: &Connection, parent_memo_id: i64) -> AppResult<Vec<i64>> {
    let mut stmt = conn.prepare(
        "SELECT memo_id FROM memo_relation WHERE related_memo_id = ?1 AND type = 'COMMENT'",
    )?;
    let rows = stmt
        .query_map(params![parent_memo_id], |r| r.get(0))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}
