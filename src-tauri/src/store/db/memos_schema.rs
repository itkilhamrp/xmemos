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

use std::collections::HashSet;

use rusqlite::Connection;

use crate::error::{AppError, AppResult};

pub const MEMOS_COMPATIBLE_SCHEMA_VERSION: &str = "0.24.0";

pub const OFFICIAL_MEMOS_TABLES: &[&str] = &[
    "migration_history",
    "system_setting",
    "user",
    "user_setting",
    "memo",
    "memo_relation",
    "resource",
    "activity",
    "inbox",
    "webhook",
    "reaction",
    "idp",
];

pub const REQUIRED_CORE_TABLES: &[&str] = &["user", "memo", "resource", "system_setting"];

/// Exact 1:1 Official FOSS Memos SQLite schema (zero proprietary columns or tables).
pub const MEMOS_SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS migration_history (
  version TEXT NOT NULL PRIMARY KEY,
  created_ts BIGINT NOT NULL DEFAULT (strftime('%s', 'now'))
);

CREATE TABLE IF NOT EXISTS system_setting (
  name TEXT NOT NULL,
  value TEXT NOT NULL,
  description TEXT NOT NULL DEFAULT '',
  UNIQUE(name)
);

CREATE TABLE IF NOT EXISTS user (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  created_ts BIGINT NOT NULL DEFAULT (strftime('%s', 'now')),
  updated_ts BIGINT NOT NULL DEFAULT (strftime('%s', 'now')),
  row_status TEXT NOT NULL CHECK (row_status IN ('NORMAL', 'ARCHIVED')) DEFAULT 'NORMAL',
  username TEXT NOT NULL UNIQUE,
  role TEXT NOT NULL CHECK (role IN ('HOST', 'ADMIN', 'USER')) DEFAULT 'USER',
  email TEXT NOT NULL DEFAULT '',
  nickname TEXT NOT NULL DEFAULT '',
  password_hash TEXT NOT NULL,
  avatar_url TEXT NOT NULL DEFAULT '',
  description TEXT NOT NULL DEFAULT ''
);

CREATE TABLE IF NOT EXISTS user_setting (
  user_id INTEGER NOT NULL,
  key TEXT NOT NULL,
  value TEXT NOT NULL,
  UNIQUE(user_id, key)
);

CREATE TABLE IF NOT EXISTS memo (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  uid TEXT NOT NULL UNIQUE,
  creator_id INTEGER NOT NULL,
  created_ts BIGINT NOT NULL DEFAULT (strftime('%s', 'now')),
  updated_ts BIGINT NOT NULL DEFAULT (strftime('%s', 'now')),
  row_status TEXT NOT NULL CHECK (row_status IN ('NORMAL', 'ARCHIVED')) DEFAULT 'NORMAL',
  content TEXT NOT NULL DEFAULT '',
  visibility TEXT NOT NULL CHECK (visibility IN ('PUBLIC', 'PROTECTED', 'PRIVATE')) DEFAULT 'PRIVATE',
  pinned INTEGER NOT NULL CHECK (pinned IN (0, 1)) DEFAULT 0,
  payload TEXT NOT NULL DEFAULT '{}'
);

CREATE TABLE IF NOT EXISTS memo_relation (
  memo_id INTEGER NOT NULL,
  related_memo_id INTEGER NOT NULL,
  type TEXT NOT NULL,
  UNIQUE(memo_id, related_memo_id, type)
);

CREATE TABLE IF NOT EXISTS resource (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  uid TEXT NOT NULL UNIQUE,
  creator_id INTEGER NOT NULL,
  created_ts BIGINT NOT NULL DEFAULT (strftime('%s', 'now')),
  updated_ts BIGINT NOT NULL DEFAULT (strftime('%s', 'now')),
  filename TEXT NOT NULL DEFAULT '',
  blob BLOB DEFAULT NULL,
  type TEXT NOT NULL DEFAULT '',
  size INTEGER NOT NULL DEFAULT 0,
  memo_id INTEGER,
  storage_type TEXT NOT NULL DEFAULT '',
  reference TEXT NOT NULL DEFAULT '',
  payload TEXT NOT NULL DEFAULT '{}'
);

CREATE TABLE IF NOT EXISTS activity (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  creator_id INTEGER NOT NULL,
  created_ts BIGINT NOT NULL DEFAULT (strftime('%s', 'now')),
  type TEXT NOT NULL DEFAULT '',
  level TEXT NOT NULL CHECK (level IN ('INFO', 'WARN', 'ERROR')) DEFAULT 'INFO',
  payload TEXT NOT NULL DEFAULT '{}'
);

CREATE TABLE IF NOT EXISTS inbox (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  created_ts BIGINT NOT NULL DEFAULT (strftime('%s', 'now')),
  sender_id INTEGER NOT NULL,
  receiver_id INTEGER NOT NULL,
  status TEXT NOT NULL,
  message TEXT NOT NULL DEFAULT '{}'
);

CREATE TABLE IF NOT EXISTS webhook (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  created_ts BIGINT NOT NULL DEFAULT (strftime('%s', 'now')),
  updated_ts BIGINT NOT NULL DEFAULT (strftime('%s', 'now')),
  row_status TEXT NOT NULL CHECK (row_status IN ('NORMAL', 'ARCHIVED')) DEFAULT 'NORMAL',
  creator_id INTEGER NOT NULL,
  name TEXT NOT NULL DEFAULT '',
  url TEXT NOT NULL DEFAULT ''
);

CREATE TABLE IF NOT EXISTS reaction (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  created_ts BIGINT NOT NULL DEFAULT (strftime('%s', 'now')),
  creator_id INTEGER NOT NULL,
  content_id TEXT NOT NULL,
  reaction_type TEXT NOT NULL,
  UNIQUE(creator_id, content_id, reaction_type)
);

CREATE TABLE IF NOT EXISTS idp (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  type TEXT NOT NULL,
  identifier_filter TEXT NOT NULL DEFAULT '',
  config TEXT NOT NULL DEFAULT '{}'
);

CREATE INDEX IF NOT EXISTS idx_memo_creator_id ON memo(creator_id);
CREATE INDEX IF NOT EXISTS idx_memo_created_ts ON memo(created_ts DESC);
CREATE INDEX IF NOT EXISTS idx_memo_row_status_visibility ON memo(row_status, visibility);
CREATE INDEX IF NOT EXISTS idx_resource_memo_id ON resource(memo_id);
CREATE INDEX IF NOT EXISTS idx_resource_creator_id ON resource(creator_id);
CREATE INDEX IF NOT EXISTS idx_inbox_receiver_id ON inbox(receiver_id);
"#;

const MEMOS_SCHEMA_USER_VERSION: i64 = 240;

pub fn init_memos_schema(conn: &Connection) -> AppResult<()> {
    let current_ver: i64 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap_or(0);
    if current_ver == MEMOS_SCHEMA_USER_VERSION {
        return Ok(());
    }
    conn.execute_batch(MEMOS_SCHEMA_SQL)?;
    conn.execute(
        "INSERT OR IGNORE INTO migration_history (version) VALUES (?1)",
        [MEMOS_COMPATIBLE_SCHEMA_VERSION],
    )?;
    let _ = conn.execute_batch(&format!("PRAGMA user_version = {MEMOS_SCHEMA_USER_VERSION};"));
    Ok(())
}

/// Inspects an imported SQLite database to verify core official Memos tables exist.
pub fn validate_imported_memos_db(conn: &Connection) -> AppResult<Vec<String>> {
    let integrity: String = conn.query_row("PRAGMA integrity_check", [], |r| r.get(0))?;
    if integrity.to_lowercase() != "ok" {
        return Err(AppError::Backup(format!(
            "SQLite integrity check failed: {integrity}"
        )));
    }

    let mut stmt = conn.prepare(
        "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%'",
    )?;
    let tables: HashSet<String> = stmt
        .query_map([], |row| row.get::<_, String>(0))?
        .filter_map(Result::ok)
        .collect();

    for required in REQUIRED_CORE_TABLES {
        if !tables.contains(*required) {
            return Err(AppError::Backup(format!(
                "Invalid Memos database: missing required table '{required}'"
            )));
        }
    }

    let mut notes = Vec::new();
    if tables.contains("memo_organizer") {
        notes.push(
            "Detected legacy memo_organizer table; pinned flags will be adapted non-destructively."
                .to_string(),
        );
    }

    Ok(notes)
}

/// Non-destructively adapts minor upstream Memos schema variations on import
/// without introducing any proprietary Xmemos columns or tables.
pub fn adapt_imported_memos_db(conn: &Connection) -> AppResult<()> {
    init_memos_schema(conn)?;

    // Check if legacy `memo_organizer` exists and migrate pinned flags into `memo.pinned`
    let has_organizer: i64 = conn.query_row(
        "SELECT COUNT(1) FROM sqlite_master WHERE type = 'table' AND name = 'memo_organizer'",
        [],
        |r| r.get(0),
    )?;
    if has_organizer > 0 {
        let _ = conn.execute(
            "UPDATE memo SET pinned = 1 WHERE id IN (SELECT memo_id FROM memo_organizer WHERE pinned = 1)",
            [],
        );
    }

    Ok(())
}

/// Verifies that `memos_prod.db` contains ZERO proprietary Xmemos tables.
pub fn assert_zero_proprietary_tables(conn: &Connection) -> AppResult<()> {
    let allowed: HashSet<&str> = OFFICIAL_MEMOS_TABLES.iter().copied().collect();
    let mut stmt = conn.prepare(
        "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%'",
    )?;
    let tables: Vec<String> = stmt
        .query_map([], |row| row.get::<_, String>(0))?
        .filter_map(Result::ok)
        .collect();

    for table in tables {
        if !allowed.contains(table.as_str()) {
            return Err(AppError::Validation(format!(
                "Non-standard table detected in memos_prod.db: {table}"
            )));
        }
    }
    Ok(())
}
