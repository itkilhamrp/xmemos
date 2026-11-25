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

pub mod local_schema;
pub mod memos_schema;

use std::path::Path;

use rusqlite::Connection;

use crate::error::AppResult;

pub fn apply_wal_pragmas(conn: &Connection) -> AppResult<()> {
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         PRAGMA foreign_keys = ON;
         PRAGMA busy_timeout = 5000;
         PRAGMA temp_store = MEMORY;
         PRAGMA cache_size = -16000;
         PRAGMA mmap_size = 67108864;
         PRAGMA wal_autocheckpoint = 1000;",
    )?;
    Ok(())
}

pub fn open_memos_db(path: &Path) -> AppResult<Connection> {
    let conn = Connection::open(path)?;
    apply_wal_pragmas(&conn)?;
    memos_schema::init_memos_schema(&conn)?;
    Ok(conn)
}

pub fn open_local_db(path: &Path) -> AppResult<Connection> {
    let conn = Connection::open(path)?;
    apply_wal_pragmas(&conn)?;
    local_schema::init_local_schema(&conn)?;
    Ok(conn)
}
