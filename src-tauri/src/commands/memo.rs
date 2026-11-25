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

use std::collections::BTreeMap;

use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::error::{AppError, AppResult};
use crate::service::auth::require_authenticated_user;
use crate::service::search;
use crate::state::AppState;
use crate::store::inbox;
use crate::store::memo::{
    self, can_view_memo, CreateMemoInput, Memo, MemoFilter, MemoLocation, UpdateMemoInput,
};
use crate::store::memo_relation::{self, MemoRelation};
use crate::store::reaction::{self, Reaction};
use crate::store::resource::{self, Resource};
use crate::store::user::{self, User};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoView {
    pub memo: Memo,
    pub creator: Option<User>,
    pub resources: Vec<Resource>,
    pub relations: Vec<MemoRelation>,
    pub reactions: Vec<Reaction>,
    pub comment_count: usize,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateMemoPayload {
    pub content: String,
    pub visibility: String,
    pub pinned: Option<bool>,
    pub created_ts: Option<i64>,
    pub location: Option<MemoLocation>,
    pub resource_ids: Option<Vec<i64>>,
    pub relation_ids: Option<Vec<i64>>,
    pub parent_memo_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateMemoPayload {
    pub id: i64,
    pub content: Option<String>,
    pub visibility: Option<String>,
    pub pinned: Option<bool>,
    pub row_status: Option<String>,
    pub created_ts: Option<i64>,
    pub location: Option<MemoLocation>,
    pub clear_location: Option<bool>,
    pub resource_ids: Option<Vec<i64>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListMemosPayload {
    pub creator_id: Option<i64>,
    pub row_status: Option<String>,
    pub visibility_list: Option<Vec<String>>,
    pub tag: Option<String>,
    pub query: Option<String>,
    pub has_task_list: Option<bool>,
    pub has_link: Option<bool>,
    pub has_code: Option<bool>,
    pub has_media: Option<bool>,
    pub start_ts: Option<i64>,
    pub end_ts: Option<i64>,
    pub order_by: Option<String>,
    pub order_direction: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TagCount {
    pub name: String,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HeatmapDay {
    pub date: String,
    pub count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoDraft {
    pub content: String,
    pub visibility: String,
    pub resource_ids: Vec<i64>,
    pub relation_ids: Vec<i64>,
    pub updated_ts: i64,
}

fn hydrate_memo_view(conn: &rusqlite::Connection, m: Memo) -> AppResult<MemoView> {
    let creator = user::get_user_by_id(conn, m.creator_id)?;
    let resources = resource::list_resources_for_memo(conn, m.id)?;
    let relations = memo_relation::list_relations_for_memo(conn, m.id)?;
    let content_id = format!("memos/{}", m.uid);
    let reactions = reaction::list_reactions_for_content(conn, &content_id)?;
    let comment_count = relations
        .iter()
        .filter(|r| r.related_memo_id == m.id && r.r#type == "COMMENT")
        .count();
    Ok(MemoView {
        memo: m,
        creator,
        resources,
        relations,
        reactions,
        comment_count,
    })
}

fn validate_visibility(v: &str) -> AppResult<()> {
    match v {
        "PRIVATE" | "PROTECTED" | "PUBLIC" => Ok(()),
        _ => Err(AppError::Validation(format!("Invalid visibility: {v}"))),
    }
}

#[tauri::command]
pub fn memo_create(state: State<'_, AppState>, payload: CreateMemoPayload) -> AppResult<MemoView> {
    let current = require_authenticated_user(&state)?;
    validate_visibility(&payload.visibility)?;
    if payload.content.trim().is_empty() && payload.resource_ids.as_ref().is_none_or(|r| r.is_empty())
    {
        return Err(AppError::Validation(
            "Memo must have content or at least one attachment".to_string(),
        ));
    }

    let uid = uuid::Uuid::new_v4().simple().to_string();
    let created = state.with_memos_db(|conn| {
        let m = memo::create_memo(
            conn,
            &CreateMemoInput {
                uid,
                creator_id: current.id,
                content: payload.content,
                visibility: payload.visibility,
                pinned: payload.pinned.unwrap_or(false),
                created_ts: payload.created_ts,
                location: payload.location,
            },
        )?;

        if let Some(ref res_ids) = payload.resource_ids {
            resource::bind_resources_to_memo(conn, m.id, res_ids, current.id)?;
        }

        if let Some(ref rel_ids) = payload.relation_ids {
            for target_id in rel_ids {
                if *target_id != m.id {
                    memo_relation::upsert_relation(conn, m.id, *target_id, "REFERENCE")?;
                }
            }
        }

        if let Some(parent_id) = payload.parent_memo_id {
            memo_relation::upsert_relation(conn, m.id, parent_id, "COMMENT")?;
            if let Some(parent) = memo::get_memo_by_id(conn, parent_id)? {
                if parent.creator_id != current.id {
                    let msg = serde_json::json!({
                        "type": "MEMO_COMMENT",
                        "memoId": parent.id,
                        "memoUid": parent.uid,
                        "commentId": m.id,
                        "commentUid": m.uid
                    })
                    .to_string();
                    inbox::create_inbox(conn, current.id, parent.creator_id, &msg)?;
                }
            }
        }

        hydrate_memo_view(conn, m)
    })?;

    search::upsert_memo_fts(&state, &created.memo)?;
    Ok(created)
}

#[tauri::command]
pub fn memo_list(state: State<'_, AppState>, payload: ListMemosPayload) -> AppResult<Vec<MemoView>> {
    let viewer_id = state.current_user_id()?;

    let fts_ids = if let Some(ref q) = payload.query {
        let trimmed = q.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(search::search_memo_ids(&state, trimmed, 500)?)
        }
    } else {
        None
    };

    let filter = MemoFilter {
        creator_id: payload.creator_id,
        row_status: payload.row_status,
        visibility_list: payload.visibility_list.unwrap_or_default(),
        tag: payload.tag,
        has_task_list: payload.has_task_list,
        has_link: payload.has_link,
        has_code: payload.has_code,
        has_media: payload.has_media,
        start_ts: payload.start_ts,
        end_ts: payload.end_ts,
        memo_ids: fts_ids,
        exclude_comments: true,
        order_by: payload.order_by,
        order_direction: payload.order_direction,
        limit: payload.limit.unwrap_or(30),
        offset: payload.offset.unwrap_or(0),
    };

    state.with_memos_db(|conn| {
        let memos = memo::list_memos(conn, viewer_id, &filter)?;
        let mut views = Vec::with_capacity(memos.len());
        for m in memos {
            views.push(hydrate_memo_view(conn, m)?);
        }
        Ok(views)
    })
}

#[tauri::command]
pub fn memo_get_by_uid(state: State<'_, AppState>, uid: String) -> AppResult<MemoView> {
    let viewer_id = state.current_user_id()?;
    state.with_memos_db(|conn| {
        let m = memo::get_memo_by_uid(conn, &uid)?
            .ok_or_else(|| AppError::NotFound(format!("Memo '{uid}' not found")))?;
        if !can_view_memo(&m, viewer_id) {
            return Err(AppError::Forbidden(
                "You do not have permission to view this memo".to_string(),
            ));
        }
        hydrate_memo_view(conn, m)
    })
}

#[tauri::command]
pub fn memo_list_comments(state: State<'_, AppState>, memo_id: i64) -> AppResult<Vec<MemoView>> {
    let viewer_id = state.current_user_id()?;
    state.with_memos_db(|conn| {
        let comment_ids = memo_relation::list_comment_ids(conn, memo_id)?;
        let mut views = Vec::new();
        for cid in comment_ids {
            if let Some(m) = memo::get_memo_by_id(conn, cid)? {
                if m.row_status == "NORMAL" && can_view_memo(&m, viewer_id) {
                    views.push(hydrate_memo_view(conn, m)?);
                }
            }
        }
        views.sort_by_key(|v| v.memo.created_ts);
        Ok(views)
    })
}

#[tauri::command]
pub fn memo_update(state: State<'_, AppState>, payload: UpdateMemoPayload) -> AppResult<MemoView> {
    let current = require_authenticated_user(&state)?;
    if let Some(ref vis) = payload.visibility {
        validate_visibility(vis)?;
    }

    let updated = state.with_memos_db(|conn| {
        let existing = memo::get_memo_by_id(conn, payload.id)?
            .ok_or_else(|| AppError::NotFound(format!("Memo {} not found", payload.id)))?;
        if existing.creator_id != current.id && current.role != "HOST" && current.role != "ADMIN" {
            return Err(AppError::Forbidden(
                "Only the memo creator or an administrator can edit this memo".to_string(),
            ));
        }

        let m = memo::update_memo(
            conn,
            &UpdateMemoInput {
                id: payload.id,
                content: payload.content,
                visibility: payload.visibility,
                pinned: payload.pinned,
                row_status: payload.row_status,
                created_ts: payload.created_ts,
                location: payload.location,
                clear_location: payload.clear_location.unwrap_or(false),
            },
        )?;

        if let Some(ref res_ids) = payload.resource_ids {
            resource::bind_resources_to_memo(conn, m.id, res_ids, current.id)?;
        }

        hydrate_memo_view(conn, m)
    })?;

    search::upsert_memo_fts(&state, &updated.memo)?;
    Ok(updated)
}

#[tauri::command]
pub fn memo_toggle_task(
    state: State<'_, AppState>,
    memo_id: i64,
    task_index: usize,
) -> AppResult<MemoView> {
    let current = require_authenticated_user(&state)?;
    let updated = state.with_memos_db(|conn| {
        let existing = memo::get_memo_by_id(conn, memo_id)?
            .ok_or_else(|| AppError::NotFound(format!("Memo {memo_id} not found")))?;
        if existing.creator_id != current.id {
            return Err(AppError::Forbidden(
                "Only the memo creator can toggle task checkboxes".to_string(),
            ));
        }
        let next_content = memo::toggle_task_checkbox(&existing.content, task_index)?;
        let m = memo::update_memo(
            conn,
            &UpdateMemoInput {
                id: memo_id,
                content: Some(next_content),
                visibility: None,
                pinned: None,
                row_status: None,
                created_ts: None,
                location: None,
                clear_location: false,
            },
        )?;
        hydrate_memo_view(conn, m)
    })?;

    search::upsert_memo_fts(&state, &updated.memo)?;
    Ok(updated)
}

#[tauri::command]
pub fn memo_delete(state: State<'_, AppState>, memo_id: i64) -> AppResult<()> {
    let current = require_authenticated_user(&state)?;
    state.with_memos_db(|conn| {
        let existing = memo::get_memo_by_id(conn, memo_id)?
            .ok_or_else(|| AppError::NotFound(format!("Memo {memo_id} not found")))?;
        if existing.creator_id != current.id && current.role != "HOST" && current.role != "ADMIN" {
            return Err(AppError::Forbidden(
                "Only the memo creator or an administrator can delete this memo".to_string(),
            ));
        }
        memo::delete_memo(conn, memo_id)
    })?;
    search::delete_memo_fts(&state, memo_id)?;
    Ok(())
}

#[tauri::command]
pub fn memo_toggle_reaction(
    state: State<'_, AppState>,
    memo_uid: String,
    reaction_type: String,
) -> AppResult<Vec<Reaction>> {
    let current = require_authenticated_user(&state)?;
    let content_id = format!("memos/{memo_uid}");
    state.with_memos_db(|conn| {
        let target = memo::get_memo_by_uid(conn, &memo_uid)?
            .ok_or_else(|| AppError::NotFound(format!("Memo '{memo_uid}' not found")))?;
        if !can_view_memo(&target, Some(current.id)) {
            return Err(AppError::Forbidden("Cannot react to private memo".to_string()));
        }
        let (added, list) = reaction::toggle_reaction(conn, current.id, &content_id, &reaction_type)?;
        if added && target.creator_id != current.id {
            let msg = serde_json::json!({
                "type": "MEMO_REACTION",
                "memoId": target.id,
                "memoUid": target.uid,
                "reactionType": reaction_type
            })
            .to_string();
            inbox::create_inbox(conn, current.id, target.creator_id, &msg)?;
        }
        Ok(list)
    })
}

#[tauri::command]
pub fn memo_list_tags(state: State<'_, AppState>, creator_id: Option<i64>) -> AppResult<Vec<TagCount>> {
    let viewer_id = state.current_user_id()?;
    let filter = MemoFilter {
        creator_id,
        row_status: Some("NORMAL".to_string()),
        exclude_comments: true,
        limit: 10_000,
        offset: 0,
        ..Default::default()
    };

    state.with_memos_db(|conn| {
        let memos = memo::list_memos(conn, viewer_id, &filter)?;
        let mut counts: BTreeMap<String, usize> = BTreeMap::new();
        for m in memos {
            for tag in m.payload.tags {
                *counts.entry(tag).or_insert(0) += 1;
            }
        }
        Ok(counts
            .into_iter()
            .map(|(name, count)| TagCount { name, count })
            .collect())
    })
}

#[tauri::command]
pub fn memo_rename_tag(
    state: State<'_, AppState>,
    old_tag: String,
    new_tag: String,
) -> AppResult<usize> {
    let current = require_authenticated_user(&state)?;
    let old_clean = old_tag.trim().trim_start_matches('#');
    let new_clean = new_tag.trim().trim_start_matches('#');
    if old_clean.is_empty() || new_clean.is_empty() {
        return Err(AppError::Validation("Tag names cannot be empty".to_string()));
    }

    let updated_memos = state.with_memos_db(|conn| {
        let filter = MemoFilter {
            creator_id: Some(current.id),
            row_status: Some("NORMAL".to_string()),
            tag: Some(old_clean.to_string()),
            limit: 10_000,
            ..Default::default()
        };
        let memos = memo::list_memos(conn, Some(current.id), &filter)?;
        let needle = format!("#{old_clean}");
        let replacement = format!("#{new_clean}");
        let mut changed = Vec::new();
        for m in memos {
            let next_content = m.content.replace(&needle, &replacement);
            if next_content != m.content {
                let upd = memo::update_memo(
                    conn,
                    &UpdateMemoInput {
                        id: m.id,
                        content: Some(next_content),
                        visibility: None,
                        pinned: None,
                        row_status: None,
                        created_ts: None,
                        location: None,
                        clear_location: false,
                    },
                )?;
                changed.push(upd);
            }
        }
        Ok(changed)
    })?;

    for m in &updated_memos {
        search::upsert_memo_fts(&state, m)?;
    }
    Ok(updated_memos.len())
}

#[tauri::command]
pub fn memo_delete_tag(state: State<'_, AppState>, tag: String) -> AppResult<usize> {
    let current = require_authenticated_user(&state)?;
    let clean = tag.trim().trim_start_matches('#');
    if clean.is_empty() {
        return Err(AppError::Validation("Tag name cannot be empty".to_string()));
    }

    let updated_memos = state.with_memos_db(|conn| {
        let filter = MemoFilter {
            creator_id: Some(current.id),
            row_status: Some("NORMAL".to_string()),
            tag: Some(clean.to_string()),
            limit: 10_000,
            ..Default::default()
        };
        let memos = memo::list_memos(conn, Some(current.id), &filter)?;
        let needle = format!("#{clean}");
        let mut changed = Vec::new();
        for m in memos {
            let next_content = m.content.replace(&needle, "").trim().to_string();
            if next_content != m.content {
                let upd = memo::update_memo(
                    conn,
                    &UpdateMemoInput {
                        id: m.id,
                        content: Some(next_content),
                        visibility: None,
                        pinned: None,
                        row_status: None,
                        created_ts: None,
                        location: None,
                        clear_location: false,
                    },
                )?;
                changed.push(upd);
            }
        }
        Ok(changed)
    })?;

    for m in &updated_memos {
        search::upsert_memo_fts(&state, m)?;
    }
    Ok(updated_memos.len())
}

#[tauri::command]
pub fn memo_get_heatmap(
    state: State<'_, AppState>,
    creator_id: Option<i64>,
    days: Option<i64>,
) -> AppResult<Vec<HeatmapDay>> {
    let viewer_id = state.current_user_id()?;
    // When days is None or <= 0, include all historical and future custom-timestamped memos
    // so navigating the monthly calendar to any past or future year displays full activity.
    let start_ts = match days {
        Some(d) if d > 0 => {
            let lookback_days = d.clamp(7, 3650);
            Some(chrono::Utc::now().timestamp() - lookback_days * 86_400)
        }
        _ => None,
    };

    let filter = MemoFilter {
        creator_id,
        row_status: Some("NORMAL".to_string()),
        start_ts,
        exclude_comments: true,
        limit: 50_000,
        ..Default::default()
    };

    state.with_memos_db(|conn| {
        let memos = memo::list_memos(conn, viewer_id, &filter)?;
        let mut counts: BTreeMap<String, usize> = BTreeMap::new();
        for m in memos {
            if let Some(dt) = chrono::DateTime::from_timestamp(m.created_ts, 0) {
                let day_key = dt.with_timezone(&chrono::Local).format("%Y-%m-%d").to_string();
                *counts.entry(day_key).or_insert(0) += 1;
            }
        }
        Ok(counts
            .into_iter()
            .map(|(date, count)| HeatmapDay { date, count })
            .collect())
    })
}

#[tauri::command]
pub fn memo_save_draft(
    state: State<'_, AppState>,
    content: String,
    visibility: String,
    resource_ids: Vec<i64>,
    relation_ids: Vec<i64>,
) -> AppResult<()> {
    let current = require_authenticated_user(&state)?;
    let local = state.lock_local_db()?;
    let now = chrono::Utc::now().timestamp();
    let res_json = serde_json::to_string(&resource_ids)?;
    let rel_json = serde_json::to_string(&relation_ids)?;
    local.execute(
        "INSERT INTO memo_draft (user_id, content, visibility, resource_ids, relation_ids, updated_ts)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)
         ON CONFLICT(user_id) DO UPDATE SET
           content = excluded.content,
           visibility = excluded.visibility,
           resource_ids = excluded.resource_ids,
           relation_ids = excluded.relation_ids,
           updated_ts = excluded.updated_ts",
        params![current.id, content, visibility, res_json, rel_json, now],
    )?;
    Ok(())
}

#[tauri::command]
pub fn memo_load_draft(state: State<'_, AppState>) -> AppResult<Option<MemoDraft>> {
    let current = require_authenticated_user(&state)?;
    let local = state.lock_local_db()?;
    let row = local
        .query_row(
            "SELECT content, visibility, resource_ids, relation_ids, updated_ts FROM memo_draft WHERE user_id = ?1",
            params![current.id],
            |r| {
                let content: String = r.get(0)?;
                let visibility: String = r.get(1)?;
                let res_s: String = r.get(2)?;
                let rel_s: String = r.get(3)?;
                let updated_ts: i64 = r.get(4)?;
                Ok(MemoDraft {
                    content,
                    visibility,
                    resource_ids: serde_json::from_str(&res_s).unwrap_or_default(),
                    relation_ids: serde_json::from_str(&rel_s).unwrap_or_default(),
                    updated_ts,
                })
            },
        )
        .optional()?;
    Ok(row)
}
