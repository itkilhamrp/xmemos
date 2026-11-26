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

use rusqlite::Connection;

use crate::error::AppResult;

/// Device-only auxiliary SQLite schema (`xmemos_local.db`).
/// Never exported into `memos_prod.db`; rebuilt automatically after DB import.
pub const LOCAL_SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS local_session (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  active_user_id INTEGER,
  updated_ts BIGINT NOT NULL DEFAULT (strftime('%s', 'now'))
);

CREATE TABLE IF NOT EXISTS user_lock (
  user_id INTEGER PRIMARY KEY,
  pin_hash TEXT,
  biometric_enabled INTEGER NOT NULL DEFAULT 0,
  require_unlock_on_switch INTEGER NOT NULL DEFAULT 0,
  updated_ts BIGINT NOT NULL DEFAULT (strftime('%s', 'now'))
);

CREATE TABLE IF NOT EXISTS memo_draft (
  user_id INTEGER PRIMARY KEY,
  content TEXT NOT NULL DEFAULT '',
  visibility TEXT NOT NULL DEFAULT 'PRIVATE',
  resource_ids TEXT NOT NULL DEFAULT '[]',
  relation_ids TEXT NOT NULL DEFAULT '[]',
  updated_ts BIGINT NOT NULL DEFAULT (strftime('%s', 'now'))
);

CREATE TABLE IF NOT EXISTS local_pref (
  key TEXT PRIMARY KEY,
  value TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS backup_history (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  mode TEXT NOT NULL CHECK (mode IN ('DB_ONLY', 'FULL_BUNDLE')),
  filename TEXT NOT NULL,
  path TEXT NOT NULL,
  size_bytes INTEGER NOT NULL DEFAULT 0,
  sha256 TEXT NOT NULL DEFAULT '',
  created_ts BIGINT NOT NULL DEFAULT (strftime('%s', 'now'))
);

CREATE VIRTUAL TABLE IF NOT EXISTS memo_fts USING fts5(
  memo_id UNINDEXED,
  content,
  tags,
  tokenize='unicode61'
);
"#;

const LOCAL_SCHEMA_USER_VERSION: i64 = 1;

pub fn init_local_schema(conn: &Connection) -> AppResult<()> {
    let current_ver: i64 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap_or(0);
    if current_ver == LOCAL_SCHEMA_USER_VERSION {
        return Ok(());
    }
    conn.execute_batch(LOCAL_SCHEMA_SQL)?;
    let _ = conn.execute_batch(&format!("PRAGMA user_version = {LOCAL_SCHEMA_USER_VERSION};"));
    Ok(())
}
