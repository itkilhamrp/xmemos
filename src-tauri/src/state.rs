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

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Mutex, MutexGuard};

use rusqlite::Connection;

use crate::error::{AppError, AppResult};
use crate::store::db::{open_local_db, open_memos_db};

pub struct AppState {
    pub memos_db_path: PathBuf,
    pub local_db_path: PathBuf,
    pub media_root: PathBuf,
    pub cache_dir: PathBuf,
    pub app_version: String,
    pub memos_conn: Mutex<Option<Connection>>,
    pub local_conn: Mutex<Connection>,
    pub active_user_id: Mutex<Option<i64>>,
    pub sticky_launch_only: AtomicBool,
}

impl AppState {
    pub fn initialize(
        db_dir: PathBuf,
        media_root: PathBuf,
        cache_dir: PathBuf,
        app_version: String,
    ) -> AppResult<Self> {
        fs::create_dir_all(&db_dir)?;
        fs::create_dir_all(media_root.join("assets").join("avatars"))?;
        fs::create_dir_all(media_root.join("backups"))?;
        fs::create_dir_all(media_root.join("temp"))?;
        fs::create_dir_all(cache_dir.join("thumbs"))?;

        let memos_db_path = db_dir.join("memos_prod.db");
        let local_db_path = db_dir.join("xmemos_local.db");

        let memos_conn = open_memos_db(&memos_db_path)?;
        let local_conn = open_local_db(&local_db_path)?;

        // Restore last active user session from xmemos_local.db if still valid in memos_prod.db
        let restored_user_id: Option<i64> = local_conn
            .query_row(
                "SELECT active_user_id FROM local_session WHERE id = 1",
                [],
                |row| row.get(0),
            )
            .ok()
            .flatten()
            .and_then(|uid: i64| {
                let exists: i64 = memos_conn
                    .query_row(
                        "SELECT COUNT(1) FROM user WHERE id = ?1 AND row_status = 'NORMAL'",
                        [uid],
                        |r| r.get(0),
                    )
                    .unwrap_or(0);
                if exists > 0 {
                    Some(uid)
                } else {
                    None
                }
            });

        Ok(Self {
            memos_db_path,
            local_db_path,
            media_root,
            cache_dir,
            app_version,
            memos_conn: Mutex::new(Some(memos_conn)),
            local_conn: Mutex::new(local_conn),
            active_user_id: Mutex::new(restored_user_id),
            sticky_launch_only: AtomicBool::new(false),
        })
    }

    /// Runs non-blocking housekeeping (stale temp cleanup + FTS5 parity check) on a background thread
    /// after the application window is already open and interactive.
    pub fn run_background_startup_maintenance(&self) {
        let _ = fs::create_dir_all(self.media_root.join("backups"));
        let _ = fs::create_dir_all(self.media_root.join("temp"));
        let _ = fs::create_dir_all(self.cache_dir.join("thumbs"));
        let temp_dir = self.media_root.join("temp");
        if let Ok(entries) = fs::read_dir(&temp_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let _ = fs::remove_dir_all(&path);
                } else {
                    let _ = fs::remove_file(&path);
                }
            }
        }
        let _ = crate::service::search::ensure_fts_index_ready(self);
    }

    /// Performs a fast, non-blocking WAL passive checkpoint and SQLite query-planner optimize on exit
    /// so closing the app is instant without risking WAL bloat or lock contention.
    pub fn prepare_fast_shutdown(&self) {
        if let Ok(guard) = self.memos_conn.try_lock() {
            if let Some(conn) = guard.as_ref() {
                let _ = conn.execute_batch("PRAGMA wal_checkpoint(PASSIVE); PRAGMA optimize;");
            }
        }
        if let Ok(local) = self.local_conn.try_lock() {
            let _ = local.execute_batch("PRAGMA wal_checkpoint(PASSIVE); PRAGMA optimize;");
        }
    }

    pub fn with_memos_db<F, T>(&self, f: F) -> AppResult<T>
    where
        F: FnOnce(&Connection) -> AppResult<T>,
    {
        let guard = self
            .memos_conn
            .lock()
            .map_err(|e| AppError::Internal(format!("memos_conn lock poisoned: {e}")))?;
        let conn = guard
            .as_ref()
            .ok_or_else(|| AppError::Internal("memos_prod.db is temporarily closed".to_string()))?;
        f(conn)
    }

    pub fn lock_local_db(&self) -> AppResult<MutexGuard<'_, Connection>> {
        self.local_conn
            .lock()
            .map_err(|e| AppError::Internal(format!("local_conn lock poisoned: {e}")))
    }

    pub fn current_user_id(&self) -> AppResult<Option<i64>> {
        let guard = self
            .active_user_id
            .lock()
            .map_err(|e| AppError::Internal(format!("active_user_id lock poisoned: {e}")))?;
        Ok(*guard)
    }

    pub fn set_current_user_id(&self, user_id: Option<i64>) -> AppResult<()> {
        {
            let mut guard = self
                .active_user_id
                .lock()
                .map_err(|e| AppError::Internal(format!("active_user_id lock poisoned: {e}")))?;
            *guard = user_id;
        }
        let local = self.lock_local_db()?;
        let now = chrono::Utc::now().timestamp();
        local.execute(
            "INSERT INTO local_session (id, active_user_id, updated_ts) VALUES (1, ?1, ?2)
             ON CONFLICT(id) DO UPDATE SET active_user_id = excluded.active_user_id, updated_ts = excluded.updated_ts",
            rusqlite::params![user_id, now],
        )?;
        Ok(())
    }

    pub fn assets_dir(&self) -> PathBuf {
        self.media_root.join("assets")
    }

    pub fn backups_dir(&self) -> PathBuf {
        self.media_root.join("backups")
    }

    pub fn temp_dir(&self) -> PathBuf {
        self.media_root.join("temp")
    }

    pub fn thumbs_dir(&self) -> PathBuf {
        self.cache_dir.join("thumbs")
    }
}
