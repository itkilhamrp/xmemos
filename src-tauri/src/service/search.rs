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

use rusqlite::params;

use crate::error::AppResult;
use crate::state::AppState;
use crate::store::memo::{self, Memo};

pub fn upsert_memo_fts(state: &AppState, memo: &Memo) -> AppResult<()> {
    let local = state.lock_local_db()?;
    let tags_str = memo.payload.tags.join(" ");
    local.execute("DELETE FROM memo_fts WHERE memo_id = ?1", params![memo.id])?;
    local.execute(
        "INSERT INTO memo_fts (memo_id, content, tags) VALUES (?1, ?2, ?3)",
        params![memo.id, memo.content, tags_str],
    )?;
    Ok(())
}

pub fn delete_memo_fts(state: &AppState, memo_id: i64) -> AppResult<()> {
    let local = state.lock_local_db()?;
    local.execute("DELETE FROM memo_fts WHERE memo_id = ?1", params![memo_id])?;
    Ok(())
}

/// Checks if the FTS5 index row count matches `memo` in O(1) time.
/// Rebuilds the full FTS5 index only when out of sync (e.g. fresh install or external DB drop-in).
pub fn ensure_fts_index_ready(state: &AppState) -> AppResult<bool> {
    let memo_count: i64 = state.with_memos_db(|conn| {
        Ok(conn.query_row("SELECT COUNT(1) FROM memo", [], |r| r.get(0))?)
    })?;
    let fts_count: i64 = {
        let local = state.lock_local_db()?;
        local.query_row("SELECT COUNT(1) FROM memo_fts", [], |r| r.get(0))?
    };
    if memo_count == fts_count {
        return Ok(false);
    }
    rebuild_fts_index(state)?;
    Ok(true)
}

pub fn rebuild_fts_index(state: &AppState) -> AppResult<usize> {
    let all_memos = state.with_memos_db(memo::list_all_memos_raw)?;
    let mut local = state.lock_local_db()?;
    let tx = local.transaction()?;
    tx.execute("DELETE FROM memo_fts", [])?;
    {
        let mut stmt =
            tx.prepare("INSERT INTO memo_fts (memo_id, content, tags) VALUES (?1, ?2, ?3)")?;
        for m in &all_memos {
            let tags_str = m.payload.tags.join(" ");
            stmt.execute(params![m.id, m.content, tags_str])?;
        }
    }
    tx.commit()?;
    Ok(all_memos.len())
}

fn sanitize_fts_query(raw: &str) -> String {
    raw.split_whitespace()
        .filter_map(|token| {
            let cleaned: String = token
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-' || *c == '/')
                .collect();
            if cleaned.is_empty() {
                None
            } else {
                Some(format!("\"{cleaned}\"*"))
            }
        })
        .collect::<Vec<_>>()
        .join(" AND ")
}

pub fn search_memo_ids(state: &AppState, query: &str, limit: i64) -> AppResult<Vec<i64>> {
    let terms: Vec<String> = query
        .split_whitespace()
        .map(|t| t.trim().to_lowercase())
        .filter(|t| !t.is_empty())
        .collect();
    if terms.is_empty() {
        return Ok(Vec::new());
    }

    let cap = limit.max(1) as usize;
    let mut matched_ids: Vec<i64> = Vec::new();
    let mut seen = std::collections::HashSet::new();

    // Fast FTS5 bm25 prefix ranking first
    let fts_expr = sanitize_fts_query(query);
    if !fts_expr.is_empty() {
        let local = state.lock_local_db()?;
        let mut stmt = local.prepare(
            "SELECT memo_id FROM memo_fts WHERE memo_fts MATCH ?1 ORDER BY bm25(memo_fts) LIMIT ?2",
        )?;
        let fts_ids = stmt
            .query_map(params![fts_expr, limit.max(1)], |r| r.get::<_, i64>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        for id in fts_ids {
            if seen.insert(id) {
                matched_ids.push(id);
            }
        }
    }

    // Union exact case-insensitive substring matches (handles mid-word & punctuation matches like Official Memos)
    if matched_ids.len() < cap {
        let sub_ids = state.with_memos_db(|conn| {
            let clauses: Vec<String> = (0..terms.len())
                .map(|i| format!("INSTR(LOWER(content), ?{}) > 0", i + 1))
                .collect();
            let sql = format!(
                "SELECT id FROM memo WHERE {} ORDER BY created_ts DESC LIMIT ?{}",
                clauses.join(" AND "),
                terms.len() + 1
            );
            let mut bound: Vec<rusqlite::types::Value> = terms
                .iter()
                .map(|t| rusqlite::types::Value::Text(t.clone()))
                .collect();
            bound.push(rusqlite::types::Value::Integer(limit.max(1)));
            let mut stmt = conn.prepare(&sql)?;
            let rows = stmt
                .query_map(rusqlite::params_from_iter(bound.iter()), |r| r.get::<_, i64>(0))?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })?;
        for id in sub_ids {
            if matched_ids.len() >= cap {
                break;
            }
            if seen.insert(id) {
                matched_ids.push(id);
            }
        }
    }

    Ok(matched_ids)
}
