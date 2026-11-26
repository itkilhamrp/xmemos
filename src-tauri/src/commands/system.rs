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
use serde::{Deserialize, Serialize};
use tauri::{window::Color, AppHandle, Emitter, Manager, State};

use crate::error::{AppError, AppResult};
#[cfg(desktop)]
use crate::service::window_pin;
use crate::service::window_pin::AlwaysOnTopResult;
use crate::service::auth::{
    require_authenticated_user, require_host, PREF_REQUIRE_PASSWORD_AFTER_SIGN_OUT,
};
use crate::state::AppState;
use crate::store::db::memos_schema::MEMOS_COMPATIBLE_SCHEMA_VERSION;
use crate::store::system_setting::{self, SystemSetting};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfo {
    pub app_version: String,
    pub memos_schema_version: String,
    pub license: String,
    pub sqlite_version: String,
    pub journal_mode: String,
    pub media_root: String,
    pub platform: String,
    pub target_arch: String,
    pub portable_mode: bool,
}

#[tauri::command]
pub fn system_get_info(state: State<'_, AppState>) -> AppResult<SystemInfo> {
    let (sqlite_version, journal_mode) = state.with_memos_db(|conn| {
        let ver: String = conn.query_row("SELECT sqlite_version()", [], |r| r.get(0))?;
        let jm: String = conn.query_row("PRAGMA journal_mode", [], |r| r.get(0))?;
        Ok((ver, jm))
    })?;

    #[cfg(desktop)]
    let portable_mode = tauri_plugin_xmemos_storage::resolve_portable_data_dir().is_some();
    #[cfg(not(desktop))]
    let portable_mode = false;

    Ok(SystemInfo {
        app_version: state.app_version.clone(),
        memos_schema_version: MEMOS_COMPATIBLE_SCHEMA_VERSION.to_string(),
        license: "AGPL-3.0-or-later".to_string(),
        sqlite_version,
        journal_mode,
        media_root: state.media_root.to_string_lossy().to_string(),
        platform: std::env::consts::OS.to_string(),
        target_arch: std::env::consts::ARCH.to_string(),
        portable_mode,
    })
}

#[tauri::command]
pub fn system_list_settings(state: State<'_, AppState>) -> AppResult<Vec<SystemSetting>> {
    state.with_memos_db(system_setting::list_system_settings)
}

#[tauri::command]
pub fn system_upsert_setting(
    state: State<'_, AppState>,
    name: String,
    value: String,
    description: Option<String>,
) -> AppResult<SystemSetting> {
    let _ = require_host(&state)?;
    state.with_memos_db(|conn| {
        system_setting::upsert_system_setting(
            conn,
            &name,
            &value,
            description.as_deref().unwrap_or(""),
        )
    })
}

#[tauri::command]
pub fn system_get_local_pref(state: State<'_, AppState>, key: String) -> AppResult<Option<String>> {
    let local = state.lock_local_db()?;
    let val = local
        .query_row(
            "SELECT value FROM local_pref WHERE key = ?1",
            params![key],
            |r| r.get::<_, String>(0),
        )
        .ok();
    Ok(val)
}

#[tauri::command]
pub fn system_set_local_pref(
    state: State<'_, AppState>,
    key: String,
    value: String,
) -> AppResult<()> {
    // Security & safety preferences cannot be mutated in Guest mode
    if key == "confirm_before_delete" || key == PREF_REQUIRE_PASSWORD_AFTER_SIGN_OUT {
        let _ = require_authenticated_user(&state)?;
    }

    let local = state.lock_local_db()?;
    local.execute(
        "INSERT INTO local_pref (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )?;
    Ok(())
}

/// Reveals and focuses the main window once the frontend has painted its first frame,
/// eliminating gray/white/black OS & WebView startup flashes.
#[tauri::command]
pub fn system_is_sticky_launch_mode(state: State<'_, AppState>) -> bool {
    state
        .sticky_launch_only
        .load(std::sync::atomic::Ordering::SeqCst)
}

#[tauri::command]
pub fn app_show_main_window(app: AppHandle, state: State<'_, AppState>) -> AppResult<()> {
    if state
        .sticky_launch_only
        .load(std::sync::atomic::Ordering::SeqCst)
    {
        return Ok(());
    }
    if let Some(window) = app.get_webview_window("main") {
        #[cfg(desktop)]
        let _ = window.set_skip_taskbar(false);
        let _ = window.show();
        let _ = window.set_focus();
    }
    Ok(())
}

/// Synchronizes native WebView/Window background color with the active Light/Dark theme.
#[tauri::command]
pub fn system_set_window_theme_bg(app: AppHandle, is_dark: bool) -> AppResult<()> {
    if let Some(window) = app.get_webview_window("main") {
        let color = if is_dark {
            Color(0, 0, 0, 255)
        } else {
            Color(242, 242, 247, 255)
        };
        let _ = window.set_background_color(Some(color));
    }
    Ok(())
}

#[cfg(desktop)]
const MAX_STICKY_WINDOWS: usize = 8;

fn validate_sticky_memo_uid(uid: &str) -> AppResult<&str> {
    let trimmed = uid.trim();
    if trimmed.is_empty()
        || trimmed.len() > 64
        || !trimmed
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        return Err(AppError::Validation("Invalid memo UID".to_string()));
    }
    Ok(trimmed)
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenStickyWindowPayload {
    pub memo_uid: String,
    pub title: Option<String>,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub always_on_top: Option<bool>,
    pub bg_rgb: Option<[u8; 3]>,
}

/// Opens or focuses a lightweight, RAM-efficient MS Sticky Notes-style floating Desktop window
/// for a specific memo (`/sticky/<uid>`). Async command per Tauri v2 docs to avoid WebView2 deadlock.
#[tauri::command]
pub async fn system_open_sticky_memo_window(
    app: AppHandle,
    payload: OpenStickyWindowPayload,
) -> AppResult<()> {
    let uid = validate_sticky_memo_uid(&payload.memo_uid)?;

    #[cfg(desktop)]
    {
        let label = format!("sticky-{uid}");
        if let Some(existing) = app.get_webview_window(&label) {
            let _ = existing.unminimize();
            let _ = existing.show();
            let _ = existing.set_focus();
            return Ok(());
        }

        // Smart RAM guard: cap concurrent floating sticky note windows
        let open_sticky_count = app
            .webview_windows()
            .keys()
            .filter(|k| k.starts_with("sticky-"))
            .count();
        if open_sticky_count >= MAX_STICKY_WINDOWS {
            return Err(AppError::Validation(format!(
                "Maximum {MAX_STICKY_WINDOWS} sticky note windows are already open. Close one to keep RAM usage low."
            )));
        }

        let base_title = payload
            .title
            .as_deref()
            .map(str::trim)
            .filter(|t| !t.is_empty())
            .unwrap_or("Xmemos Sticky Note");
        let short_uid: String = uid.chars().take(8).collect();
        let win_title = format!("{base_title} · Xmemos [{short_uid}]");
        let w = payload.width.unwrap_or(332.0).clamp(240.0, 720.0);
        let h = payload.height.unwrap_or(356.0).clamp(200.0, 860.0);

        let [r, g, b] = payload.bg_rgb.unwrap_or([28, 28, 30]);
        let mut builder = tauri::WebviewWindowBuilder::new(
            &app,
            label.clone(),
            tauri::WebviewUrl::App(format!("sticky/{uid}").into()),
        )
        .title(win_title)
        .inner_size(w, h)
        .min_inner_size(240.0, 200.0)
        .max_inner_size(720.0, 860.0)
        .resizable(true)
        .maximizable(false)
        .decorations(false)
        .always_on_top(payload.always_on_top.unwrap_or(false))
        .background_color(Color(r, g, b, 255))
        .visible(false);

        if let (Some(px), Some(py)) = (payload.x, payload.y) {
            if px.is_finite()
                && py.is_finite()
                && (-4000.0..=12000.0).contains(&px)
                && (-4000.0..=12000.0).contains(&py)
            {
                builder = builder.position(px, py);
            } else {
                builder = builder.center();
            }
        } else {
            builder = builder.center();
        }

        builder
            .build()
            .map_err(|e| AppError::Internal(format!("Failed to open sticky memo window: {e}")))?;

        // Failsafe reveal so the sticky window can never stay hidden if frontend reveal is delayed
        let failsafe_app = app.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(700));
            if let Some(win) = failsafe_app.get_webview_window(&label) {
                let _ = win.show();
                let _ = win.set_focus();
            }
        });
        Ok(())
    }

    #[cfg(not(desktop))]
    {
        let _ = (app, uid);
        Err(AppError::Validation(
            "Floating sticky windows are only available on Desktop".to_string(),
        ))
    }
}

fn validate_internal_route(route: &str) -> AppResult<&str> {
    let trimmed = route.trim();
    if !trimmed.starts_with('/')
        || trimmed.starts_with("//")
        || trimmed.len() > 256
        || trimmed.contains("..")
        || trimmed.chars().any(|c| c.is_control())
    {
        return Err(AppError::Validation("Invalid internal route".to_string()));
    }
    Ok(trimmed)
}

/// Focuses the main Xmemos window and navigates it to an internal route (e.g. `/memo/<uid>`, `/map?memo=<uid>`).
#[tauri::command]
pub async fn system_focus_main_route(
    app: AppHandle,
    state: State<'_, AppState>,
    route: String,
) -> AppResult<()> {
    let safe_route = validate_internal_route(&route)?;
    state
        .sticky_launch_only
        .store(false, std::sync::atomic::Ordering::SeqCst);
    if let Some(main_win) = app.get_webview_window("main") {
        #[cfg(desktop)]
        {
            let _ = main_win.set_skip_taskbar(false);
            let _ = main_win.unminimize();
        }
        let _ = main_win.show();
        let _ = main_win.set_focus();
        let _ = app.emit_to("main", "xmemos:navigate-main", safe_route);
    }
    Ok(())
}

/// Focuses the main Xmemos window and navigates straight to `/memo/<uid>`.
#[tauri::command]
pub async fn system_focus_main_memo(
    app: AppHandle,
    state: State<'_, AppState>,
    memo_uid: String,
) -> AppResult<()> {
    let uid = validate_sticky_memo_uid(&memo_uid)?;
    system_focus_main_route(app, state, format!("/memo/{uid}")).await
}

/// Cross-OS Always-on-Top toggle for floating Desktop Sticky Note windows (`sticky-*`).
/// Returns the verified OS always-on-top state after applying it.
#[tauri::command]
pub async fn system_set_sticky_always_on_top(
    app: AppHandle,
    label: String,
    always_on_top: bool,
) -> AppResult<AlwaysOnTopResult> {
    let trimmed = label.trim();
    if !trimmed.starts_with("sticky-") || trimmed.len() > 80 {
        return Err(AppError::Validation("Invalid sticky window label".to_string()));
    }
    #[cfg(desktop)]
    {
        if let Some(win) = app.get_webview_window(trimmed) {
            return window_pin::apply_window_always_on_top(&win, always_on_top);
        }
        Ok(AlwaysOnTopResult {
            active: false,
            supported: false,
            backend: "missing".to_string(),
        })
    }
    #[cfg(not(desktop))]
    {
        let _ = (app, always_on_top);
        Ok(AlwaysOnTopResult {
            active: false,
            supported: false,
            backend: "mobile".to_string(),
        })
    }
}
