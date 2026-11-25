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
pub struct Resource {
    pub id: i64,
    pub uid: String,
    pub creator_id: i64,
    pub created_ts: i64,
    pub updated_ts: i64,
    pub filename: String,
    pub r#type: String,
    pub size: i64,
    pub memo_id: Option<i64>,
    pub storage_type: String,
    pub reference: String,
    pub payload: String,
    pub has_blob: bool,
}

#[derive(Debug, Clone)]
pub struct CreateResourceInput {
    pub uid: String,
    pub creator_id: i64,
    pub filename: String,
    pub blob: Option<Vec<u8>>,
    pub mime_type: String,
    pub size: i64,
    pub memo_id: Option<i64>,
    pub storage_type: String,
    pub reference: String,
    pub payload: String,
}

fn map_resource(row: &rusqlite::Row<'_>) -> rusqlite::Result<Resource> {
    let has_blob_int: i64 = row.get(12)?;
    Ok(Resource {
        id: row.get(0)?,
        uid: row.get(1)?,
        creator_id: row.get(2)?,
        created_ts: row.get(3)?,
        updated_ts: row.get(4)?,
        filename: row.get(5)?,
        r#type: row.get(6)?,
        size: row.get(7)?,
        memo_id: row.get(8)?,
        storage_type: row.get(9)?,
        reference: row.get(10)?,
        payload: row.get(11)?,
        has_blob: has_blob_int != 0,
    })
}

pub fn create_resource(conn: &Connection, input: &CreateResourceInput) -> AppResult<Resource> {
    let now = chrono::Utc::now().timestamp();
    conn.execute(
        "INSERT INTO resource (uid, creator_id, created_ts, updated_ts, filename, blob, type, size, memo_id, storage_type, reference, payload)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![
            input.uid,
            input.creator_id,
            now,
            now,
            input.filename,
            input.blob,
            input.mime_type,
            input.size,
            input.memo_id,
            input.storage_type,
            input.reference,
            input.payload
        ],
    )?;
    let id = conn.last_insert_rowid();
    get_resource_by_id(conn, id)?
        .ok_or_else(|| AppError::Internal("Failed to fetch created resource".to_string()))
}

pub fn get_resource_by_id(conn: &Connection, id: i64) -> AppResult<Option<Resource>> {
    let row = conn
        .query_row(
            "SELECT id, uid, creator_id, created_ts, updated_ts, filename, type, size, memo_id, storage_type, reference, payload, (blob IS NOT NULL)
             FROM resource WHERE id = ?1",
            params![id],
            map_resource,
        )
        .optional()?;
    Ok(row)
}

pub fn get_resource_by_uid(conn: &Connection, uid: &str) -> AppResult<Option<Resource>> {
    let row = conn
        .query_row(
            "SELECT id, uid, creator_id, created_ts, updated_ts, filename, type, size, memo_id, storage_type, reference, payload, (blob IS NOT NULL)
             FROM resource WHERE uid = ?1",
            params![uid],
            map_resource,
        )
        .optional()?;
    Ok(row)
}

pub fn get_resource_blob(conn: &Connection, uid: &str) -> AppResult<Option<Vec<u8>>> {
    let blob = conn
        .query_row(
            "SELECT blob FROM resource WHERE uid = ?1",
            params![uid],
            |r| r.get::<_, Option<Vec<u8>>>(0),
        )
        .optional()?
        .flatten();
    Ok(blob)
}

pub fn list_resources_for_memo(conn: &Connection, memo_id: i64) -> AppResult<Vec<Resource>> {
    let mut stmt = conn.prepare(
        "SELECT id, uid, creator_id, created_ts, updated_ts, filename, type, size, memo_id, storage_type, reference, payload, (blob IS NOT NULL)
         FROM resource WHERE memo_id = ?1 ORDER BY id ASC",
    )?;
    let rows = stmt
        .query_map(params![memo_id], map_resource)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn list_resources_by_creator(conn: &Connection, creator_id: i64) -> AppResult<Vec<Resource>> {
    let mut stmt = conn.prepare(
        "SELECT id, uid, creator_id, created_ts, updated_ts, filename, type, size, memo_id, storage_type, reference, payload, (blob IS NOT NULL)
         FROM resource WHERE creator_id = ?1 ORDER BY created_ts DESC, id DESC",
    )?;
    let rows = stmt
        .query_map(params![creator_id], map_resource)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn list_all_resources(conn: &Connection) -> AppResult<Vec<Resource>> {
    let mut stmt = conn.prepare(
        "SELECT id, uid, creator_id, created_ts, updated_ts, filename, type, size, memo_id, storage_type, reference, payload, (blob IS NOT NULL)
         FROM resource ORDER BY created_ts DESC, id DESC",
    )?;
    let rows = stmt
        .query_map([], map_resource)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn bind_resources_to_memo(
    conn: &Connection,
    memo_id: i64,
    resource_ids: &[i64],
    creator_id: i64,
) -> AppResult<()> {
    let now = chrono::Utc::now().timestamp();
    for rid in resource_ids {
        conn.execute(
            "UPDATE resource SET memo_id = ?1, updated_ts = ?2 WHERE id = ?3 AND creator_id = ?4",
            params![memo_id, now, rid, creator_id],
        )?;
    }
    Ok(())
}

pub fn externalize_resource_storage(
    conn: &Connection,
    id: i64,
    reference: &str,
) -> AppResult<()> {
    let now = chrono::Utc::now().timestamp();
    conn.execute(
        "UPDATE resource SET storage_type = 'LOCAL', reference = ?1, blob = NULL, updated_ts = ?2 WHERE id = ?3",
        params![reference, now, id],
    )?;
    Ok(())
}

pub fn delete_resource(conn: &Connection, id: i64) -> AppResult<()> {
    conn.execute("DELETE FROM resource WHERE id = ?1", params![id])?;
    Ok(())
}
