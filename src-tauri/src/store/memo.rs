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

use std::collections::BTreeSet;
use std::sync::OnceLock;

use regex::Regex;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::error::{AppError, AppResult};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoProperty {
    #[serde(default)]
    pub has_link: bool,
    #[serde(default)]
    pub has_task_list: bool,
    #[serde(default)]
    pub has_code: bool,
    #[serde(default)]
    pub has_incomplete_tasks: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoLocation {
    #[serde(default)]
    pub placeholder: String,
    #[serde(default)]
    pub latitude: f64,
    #[serde(default)]
    pub longitude: f64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoPayload {
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub property: MemoProperty,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<MemoLocation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Memo {
    pub id: i64,
    pub uid: String,
    pub creator_id: i64,
    pub created_ts: i64,
    pub updated_ts: i64,
    pub row_status: String,
    pub content: String,
    pub visibility: String,
    pub pinned: bool,
    pub payload: MemoPayload,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMemoInput {
    pub uid: String,
    pub creator_id: i64,
    pub content: String,
    pub visibility: String,
    pub pinned: bool,
    pub created_ts: Option<i64>,
    pub location: Option<MemoLocation>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMemoInput {
    pub id: i64,
    pub content: Option<String>,
    pub visibility: Option<String>,
    pub pinned: Option<bool>,
    pub row_status: Option<String>,
    pub created_ts: Option<i64>,
    pub location: Option<MemoLocation>,
    #[serde(default)]
    pub clear_location: bool,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoFilter {
    pub creator_id: Option<i64>,
    pub row_status: Option<String>,
    pub visibility_list: Vec<String>,
    pub tag: Option<String>,
    pub has_task_list: Option<bool>,
    pub has_link: Option<bool>,
    pub has_code: Option<bool>,
    pub has_media: Option<bool>,
    pub start_ts: Option<i64>,
    pub end_ts: Option<i64>,
    pub memo_ids: Option<Vec<i64>>,
    pub exclude_comments: bool,
    pub order_by: Option<String>,
    pub order_direction: Option<String>,
    pub limit: i64,
    pub offset: i64,
}

fn tag_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"(?:^|[\s(])#([\p{L}\p{N}_\-/]+)").expect("valid tag regex")
    })
}

fn link_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"https?://[^\s\)]+").expect("valid link regex"))
}

fn task_list_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?m)^\s*[-*+]\s+\[[ xX]\]\s+").expect("valid task regex"))
}

fn incomplete_task_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"(?m)^\s*[-*+]\s+\[ \]\s+").expect("valid incomplete task regex"))
}

fn code_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"```|`[^`\n]+`").expect("valid code regex"))
}

/// Computes official Memos-compatible `memo.payload` JSON metadata from Markdown content.
pub fn extract_payload(content: &str, location: Option<MemoLocation>) -> MemoPayload {
    // Strip fenced code blocks when extracting #tags so code comments aren't mistaken for tags
    let mut non_code = String::with_capacity(content.len());
    let mut in_fence = false;
    for line in content.lines() {
        if line.trim_start().starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if !in_fence {
            non_code.push_str(line);
            non_code.push('\n');
        }
    }

    let mut tags_set = BTreeSet::new();
    for cap in tag_regex().captures_iter(&non_code) {
        if let Some(m) = cap.get(1) {
            let tag = m.as_str().trim_matches('/');
            if !tag.is_empty() {
                tags_set.insert(tag.to_string());
            }
        }
    }

    MemoPayload {
        tags: tags_set.into_iter().collect(),
        property: MemoProperty {
            has_link: link_regex().is_match(content),
            has_task_list: task_list_regex().is_match(content),
            has_code: code_regex().is_match(content),
            has_incomplete_tasks: incomplete_task_regex().is_match(content),
        },
        location,
    }
}

/// Toggles the Nth (0-indexed) Markdown task checkbox `- [ ]` <-> `- [x]` in content.
pub fn toggle_task_checkbox(content: &str, task_index: usize) -> AppResult<String> {
    let mut current_index = 0usize;
    let mut lines: Vec<String> = Vec::new();
    let mut found = false;

    for line in content.lines() {
        if !found && task_list_regex().is_match(line) {
            if current_index == task_index {
                let updated = if line.contains("[ ]") {
                    line.replacen("[ ]", "[x]", 1)
                } else if line.contains("[x]") {
                    line.replacen("[x]", "[ ]", 1)
                } else {
                    line.replacen("[X]", "[ ]", 1)
                };
                lines.push(updated);
                found = true;
            } else {
                lines.push(line.to_string());
            }
            current_index += 1;
        } else {
            lines.push(line.to_string());
        }
    }

    if !found {
        return Err(AppError::Validation(format!(
            "Task checkbox at index {task_index} not found"
        )));
    }

    let mut result = lines.join("\n");
    if content.ends_with('\n') {
        result.push('\n');
    }
    Ok(result)
}

fn map_memo(row: &rusqlite::Row<'_>) -> rusqlite::Result<Memo> {
    let pinned_int: i64 = row.get(8)?;
    let payload_str: String = row.get(9)?;
    let payload: MemoPayload = serde_json::from_str(&payload_str).unwrap_or_default();
    Ok(Memo {
        id: row.get(0)?,
        uid: row.get(1)?,
        creator_id: row.get(2)?,
        created_ts: row.get(3)?,
        updated_ts: row.get(4)?,
        row_status: row.get(5)?,
        content: row.get(6)?,
        visibility: row.get(7)?,
        pinned: pinned_int != 0,
        payload,
    })
}

pub fn create_memo(conn: &Connection, input: &CreateMemoInput) -> AppResult<Memo> {
    let now = chrono::Utc::now().timestamp();
    let created_ts = input.created_ts.unwrap_or(now);
    let payload = extract_payload(&input.content, input.location.clone());
    let payload_json = serde_json::to_string(&payload)?;
    let pinned_int: i64 = if input.pinned { 1 } else { 0 };

    let updated_ts = created_ts;
    conn.execute(
        "INSERT INTO memo (uid, creator_id, created_ts, updated_ts, row_status, content, visibility, pinned, payload)
         VALUES (?1, ?2, ?3, ?4, 'NORMAL', ?5, ?6, ?7, ?8)",
        params![
            input.uid,
            input.creator_id,
            created_ts,
            updated_ts,
            input.content,
            input.visibility,
            pinned_int,
            payload_json
        ],
    )?;

    let id = conn.last_insert_rowid();
    get_memo_by_id(conn, id)?
        .ok_or_else(|| AppError::Internal("Failed to fetch created memo".to_string()))
}

pub fn get_memo_by_id(conn: &Connection, id: i64) -> AppResult<Option<Memo>> {
    let memo = conn
        .query_row(
            "SELECT id, uid, creator_id, created_ts, updated_ts, row_status, content, visibility, pinned, payload
             FROM memo WHERE id = ?1",
            params![id],
            map_memo,
        )
        .optional()?;
    Ok(memo)
}

pub fn get_memo_by_uid(conn: &Connection, uid: &str) -> AppResult<Option<Memo>> {
    let memo = conn
        .query_row(
            "SELECT id, uid, creator_id, created_ts, updated_ts, row_status, content, visibility, pinned, payload
             FROM memo WHERE uid = ?1",
            params![uid],
            map_memo,
        )
        .optional()?;
    Ok(memo)
}

pub fn update_memo(conn: &Connection, input: &UpdateMemoInput) -> AppResult<Memo> {
    let existing = get_memo_by_id(conn, input.id)?
        .ok_or_else(|| AppError::NotFound(format!("Memo {} not found", input.id)))?;

    let now = chrono::Utc::now().timestamp();
    let content = input.content.as_deref().unwrap_or(&existing.content);
    let visibility = input.visibility.as_deref().unwrap_or(&existing.visibility);
    let pinned = input.pinned.unwrap_or(existing.pinned);
    let row_status = input.row_status.as_deref().unwrap_or(&existing.row_status);
    let created_ts = input.created_ts.unwrap_or(existing.created_ts);
    let location = if input.clear_location {
        None
    } else {
        input.location.clone().or(existing.payload.location.clone())
    };

    let payload = extract_payload(content, location);
    let payload_json = serde_json::to_string(&payload)?;
    let pinned_int: i64 = if pinned { 1 } else { 0 };

    conn.execute(
        "UPDATE memo SET content = ?1, visibility = ?2, pinned = ?3, row_status = ?4,
         created_ts = ?5, updated_ts = ?6, payload = ?7 WHERE id = ?8",
        params![
            content,
            visibility,
            pinned_int,
            row_status,
            created_ts,
            now,
            payload_json,
            input.id
        ],
    )?;

    get_memo_by_id(conn, input.id)?
        .ok_or_else(|| AppError::NotFound(format!("Memo {} not found after update", input.id)))
}

pub fn delete_memo(conn: &Connection, id: i64) -> AppResult<()> {
    conn.execute(
        "DELETE FROM memo_relation WHERE memo_id = ?1 OR related_memo_id = ?1",
        params![id],
    )?;
    conn.execute("UPDATE resource SET memo_id = NULL WHERE memo_id = ?1", params![id])?;
    conn.execute("DELETE FROM memo WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn list_memos(
    conn: &Connection,
    viewer_id: Option<i64>,
    filter: &MemoFilter,
) -> AppResult<Vec<Memo>> {
    let mut clauses: Vec<String> = Vec::new();
    let mut bound: Vec<rusqlite::types::Value> = Vec::new();

    // Enforce 1:1 Official Memos visibility matrix strictly in SQL
    match viewer_id {
        Some(uid) => {
            clauses.push("(m.visibility IN ('PUBLIC', 'PROTECTED') OR m.creator_id = ?)".to_string());
            bound.push(rusqlite::types::Value::Integer(uid));
        }
        None => {
            clauses.push("m.visibility = 'PUBLIC'".to_string());
        }
    }

    if let Some(creator_id) = filter.creator_id {
        clauses.push("m.creator_id = ?".to_string());
        bound.push(rusqlite::types::Value::Integer(creator_id));
    }

    let status = filter.row_status.as_deref().unwrap_or("NORMAL");
    clauses.push("m.row_status = ?".to_string());
    bound.push(rusqlite::types::Value::Text(status.to_string()));

    if !filter.visibility_list.is_empty() {
        let placeholders = vec!["?"; filter.visibility_list.len()].join(", ");
        clauses.push(format!("m.visibility IN ({placeholders})"));
        for vis in &filter.visibility_list {
            bound.push(rusqlite::types::Value::Text(vis.clone()));
        }
    }

    if filter.exclude_comments {
        clauses.push(
            "NOT EXISTS (SELECT 1 FROM memo_relation mr WHERE mr.memo_id = m.id AND mr.type = 'COMMENT')"
                .to_string(),
        );
    }

    if let Some(start_ts) = filter.start_ts {
        clauses.push("m.created_ts >= ?".to_string());
        bound.push(rusqlite::types::Value::Integer(start_ts));
    }
    if let Some(end_ts) = filter.end_ts {
        clauses.push("m.created_ts <= ?".to_string());
        bound.push(rusqlite::types::Value::Integer(end_ts));
    }

    if filter.has_media == Some(true) {
        clauses.push("EXISTS (SELECT 1 FROM resource r WHERE r.memo_id = m.id)".to_string());
    }

    if let Some(ref ids) = filter.memo_ids {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        let placeholders = vec!["?"; ids.len()].join(", ");
        clauses.push(format!("m.id IN ({placeholders})"));
        for id in ids {
            bound.push(rusqlite::types::Value::Integer(*id));
        }
    }

    let order_col = match filter.order_by.as_deref() {
        Some("updated_ts") => "m.updated_ts",
        _ => "m.created_ts",
    };
    let order_dir = match filter.order_direction.as_deref() {
        Some("asc") => "ASC",
        _ => "DESC",
    };

    let sql = format!(
        "SELECT m.id, m.uid, m.creator_id, m.created_ts, m.updated_ts, m.row_status,
                m.content, m.visibility, m.pinned, m.payload
         FROM memo m
         WHERE {}
         ORDER BY m.pinned DESC, {order_col} {order_dir}, m.id {order_dir}",
        clauses.join(" AND ")
    );

    let mut stmt = conn.prepare(&sql)?;
    let all_rows = stmt
        .query_map(rusqlite::params_from_iter(bound.iter()), map_memo)?
        .collect::<Result<Vec<_>, _>>()?;

    // Filter payload-level attributes (tag prefix, hasTaskList, hasLink, hasCode)
    let filtered: Vec<Memo> = all_rows
        .into_iter()
        .filter(|m| {
            if let Some(ref tag) = filter.tag {
                let prefix = format!("{tag}/");
                if !m
                    .payload
                    .tags
                    .iter()
                    .any(|t| t == tag || t.starts_with(&prefix))
                {
                    return false;
                }
            }
            if filter.has_task_list == Some(true) && !m.payload.property.has_task_list {
                return false;
            }
            if filter.has_link == Some(true) && !m.payload.property.has_link {
                return false;
            }
            if filter.has_code == Some(true) && !m.payload.property.has_code {
                return false;
            }
            true
        })
        .skip(filter.offset.max(0) as usize)
        .take(if filter.limit > 0 {
            filter.limit as usize
        } else {
            50
        })
        .collect();

    Ok(filtered)
}

pub fn list_all_memos_raw(conn: &Connection) -> AppResult<Vec<Memo>> {
    let mut stmt = conn.prepare(
        "SELECT id, uid, creator_id, created_ts, updated_ts, row_status, content, visibility, pinned, payload
         FROM memo ORDER BY id ASC",
    )?;
    let rows = stmt
        .query_map([], map_memo)?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn can_view_memo(memo: &Memo, viewer_id: Option<i64>) -> bool {
    match memo.visibility.as_str() {
        "PUBLIC" => true,
        "PROTECTED" => viewer_id.is_some(),
        "PRIVATE" => viewer_id == Some(memo.creator_id),
        _ => false,
    }
}
