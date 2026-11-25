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

pub mod commands;
pub mod error;
pub mod service;
pub mod state;
pub mod store;

use std::path::PathBuf;
#[cfg(desktop)]
use std::time::Duration;

use tauri::http::{header, Response, StatusCode};
use tauri::window::Color;
use tauri::Manager;
use tauri_plugin_xmemos_storage::XmemosStorageExt;

use crate::service::media;
use crate::state::AppState;
use crate::store::{memo, resource};

fn handle_media_protocol(
    app: &tauri::AppHandle,
    uri_path: &str,
    query: Option<&str>,
) -> Response<Vec<u8>> {
    let uid = uri_path.trim_start_matches('/').trim();
    if uid.is_empty() || !uid.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(b"Invalid resource identifier".to_vec())
            .unwrap_or_default();
    }

    let prefer_thumb = query.is_some_and(|q| q.contains("thumb=1"));
    let state = app.state::<AppState>();
    let viewer_id = match state.current_user_id() {
        Ok(v) => v,
        Err(_) => {
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Vec::new())
                .unwrap_or_default();
        }
    };

    let resource_opt = state.with_memos_db(|conn| {
        let Some(res) = resource::get_resource_by_uid(conn, uid)? else {
            return Ok(None);
        };
        // Verify visibility permissions against SQLite before streaming bytes
        if let Some(memo_id) = res.memo_id {
            if let Some(m) = memo::get_memo_by_id(conn, memo_id)? {
                if !memo::can_view_memo(&m, viewer_id) {
                    return Err(crate::error::AppError::Forbidden(
                        "Forbidden media access".to_string(),
                    ));
                }
            }
        } else if viewer_id != Some(res.creator_id) {
            return Err(crate::error::AppError::Forbidden(
                "Unbound resource is private to creator".to_string(),
            ));
        }
        Ok(Some(res))
    });

    match resource_opt {
        Ok(Some(res)) => match media::read_resource_bytes(&state, &res, prefer_thumb) {
            Ok((bytes, mime)) => Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime)
                .header(header::CACHE_CONTROL, "private, max-age=86400")
                .body(bytes)
                .unwrap_or_default(),
            Err(_) => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(b"Media file not found".to_vec())
                .unwrap_or_default(),
        },
        Ok(None) => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(b"Resource not found".to_vec())
            .unwrap_or_default(),
        Err(crate::error::AppError::Forbidden(_)) => Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body(b"Forbidden".to_vec())
            .unwrap_or_default(),
        Err(_) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Vec::new())
            .unwrap_or_default(),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_xmemos_storage::init())
        .register_uri_scheme_protocol("xmemos-media", |ctx, request| {
            let uri = request.uri();
            handle_media_protocol(ctx.app_handle(), uri.path(), uri.query())
        })
        .setup(|app| {
            let handle = app.handle();

            #[cfg(desktop)]
            let (db_dir, cache_dir) =
                if let Some(portable_base) = tauri_plugin_xmemos_storage::resolve_portable_data_dir() {
                    (portable_base.join("db"), portable_base.join("cache"))
                } else {
                    (
                        handle.path().app_data_dir()?.join("db"),
                        handle.path().app_cache_dir()?,
                    )
                };

            #[cfg(not(desktop))]
            let (db_dir, cache_dir) = (
                handle.path().app_data_dir()?.join("db"),
                handle.path().app_cache_dir()?,
            );

            let media_root = match handle.xmemos_storage().get_media_root() {
                Ok(info) => PathBuf::from(info.root_path),
                Err(_) => handle.path().app_data_dir()?.join("media"),
            };

            let app_version = app.package_info().version.to_string();
            let app_state = AppState::initialize(db_dir, media_root, cache_dir, app_version)
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

            // Match native WebView background to persisted Light theme before first show
            let saved_theme: Option<String> = app_state
                .lock_local_db()
                .ok()
                .and_then(|local| {
                    local
                        .query_row(
                            "SELECT value FROM local_pref WHERE key = 'theme_mode'",
                            [],
                            |r| r.get::<_, String>(0),
                        )
                        .ok()
                });
            if saved_theme.as_deref() == Some("light") {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.set_background_color(Some(Color(242, 242, 247, 255)));
                }
            }

            // Ensure frameless integrated macOS-style window header on GNU/Linux, macOS & Windows
            #[cfg(desktop)]
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_decorations(false);
            }

            app.manage(app_state);

            // Offload temp cleanup & FTS5 parity check to a background thread for instant launch
            let bg_handle = app.handle().clone();
            std::thread::spawn(move || {
                let state = bg_handle.state::<AppState>();
                state.run_background_startup_maintenance();
            });

            // Failsafe reveal on Desktop so the frameless window can never stay hidden if frontend reveal is delayed
            #[cfg(desktop)]
            {
                let failsafe_handle = app.handle().clone();
                std::thread::spawn(move || {
                    std::thread::sleep(Duration::from_millis(1200));
                    if let Some(window) = failsafe_handle.get_webview_window("main") {
                        let _ = window.show();
                    }
                });
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Auth
            commands::auth::auth_get_status,
            commands::auth::auth_setup_host,
            commands::auth::auth_sign_up,
            commands::auth::auth_sign_in,
            commands::auth::auth_sign_out,
            commands::auth::auth_switch_account,
            commands::auth::auth_set_lock_config,
            commands::auth::auth_verify_pin,
            // Memo
            commands::memo::memo_create,
            commands::memo::memo_list,
            commands::memo::memo_get_by_uid,
            commands::memo::memo_list_comments,
            commands::memo::memo_update,
            commands::memo::memo_toggle_task,
            commands::memo::memo_delete,
            commands::memo::memo_toggle_reaction,
            commands::memo::memo_list_tags,
            commands::memo::memo_rename_tag,
            commands::memo::memo_delete_tag,
            commands::memo::memo_get_heatmap,
            commands::memo::memo_save_draft,
            commands::memo::memo_load_draft,
            // Resource
            commands::resource::resource_upload_bytes,
            commands::resource::resource_import_file,
            commands::resource::resource_list,
            commands::resource::resource_delete,
            commands::resource::resource_storage_breakdown,
            commands::resource::resource_scan_orphans,
            commands::resource::resource_clean_orphans,
            commands::resource::resource_externalize_blobs,
            commands::resource::resource_export_to_downloads,
            commands::resource::resource_save_bytes,
            // User
            commands::user::user_list,
            commands::user::user_get_profile,
            commands::user::user_admin_create,
            commands::user::user_update,
            commands::user::user_delete,
            commands::user::user_list_settings,
            commands::user::user_upsert_setting,
            // Inbox
            commands::inbox::inbox_list,
            commands::inbox::inbox_update_status,
            commands::inbox::inbox_delete,
            // System
            commands::system::system_get_info,
            commands::system::system_list_settings,
            commands::system::system_upsert_setting,
            commands::system::system_get_local_pref,
            commands::system::system_set_local_pref,
            commands::system::app_show_main_window,
            commands::system::system_set_window_theme_bg,
            commands::system::system_open_sticky_memo_window,
            commands::system::system_focus_main_memo,
            commands::system::system_focus_main_route,
            commands::system::system_set_sticky_always_on_top,
            // Backup
            commands::backup::backup_export_db,
            commands::backup::backup_export_full,
            commands::backup::backup_list_history,
            commands::backup::backup_preview_import,
            commands::backup::backup_execute_import,
        ])
        .build(tauri::generate_context!())
        .expect("error while building Xmemos Tauri application");

    app.run(|app_handle, event| {
        if let tauri::RunEvent::Exit = event {
            if let Some(state) = app_handle.try_state::<AppState>() {
                state.prepare_fast_shutdown();
            }
        }
    });
}
