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

//! Cross-OS Always-on-Top engine for floating Desktop Sticky Note windows:
//! - Windows 10/11: native Win32 HWND_TOPMOST via Tauri/Tao `set_always_on_top`
//! - macOS 12+: native AppKit NSFloatingWindowLevel via Tauri/Tao `set_always_on_top`
//! - GNU/Linux X11 / XWayland: native GTK3 `_NET_WM_STATE_ABOVE` via `set_always_on_top`
//! - GNU/Linux Wayland (KDE Plasma 5/6 KWin): native `org.kde.KWin /Scripting` D-Bus `keepAbove`
//! - GNU/Linux Wayland (Hyprland): native `hyprctl dispatch pin`
//! - Other pure Wayland compositors (e.g. GNOME Mutter): honest `supported: false` status

use serde::Serialize;

#[cfg(desktop)]
use tauri::WebviewWindow;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AlwaysOnTopResult {
    pub active: bool,
    pub supported: bool,
    pub backend: String,
}

#[cfg(desktop)]
pub fn apply_window_always_on_top(
    win: &WebviewWindow,
    always_on_top: bool,
) -> crate::error::AppResult<AlwaysOnTopResult> {
    // Always invoke Tauri/Tao native API first (Windows, macOS, Linux X11/XWayland)
    let _ = win.set_always_on_top(always_on_top);
    if always_on_top {
        let _ = win.set_focus();
    }

    #[cfg(target_os = "linux")]
    {
        let is_wayland_native = is_native_wayland_session();
        if !is_wayland_native {
            return Ok(AlwaysOnTopResult {
                active: always_on_top,
                supported: true,
                backend: "x11".to_string(),
            });
        }

        let title = win.title().unwrap_or_default();
        if !title.is_empty() && try_kwin_dbus_keep_above(&title, always_on_top) {
            return Ok(AlwaysOnTopResult {
                active: always_on_top,
                supported: true,
                backend: "kwin-wayland".to_string(),
            });
        }

        if !title.is_empty() && try_hyprland_pin(&title, always_on_top) {
            return Ok(AlwaysOnTopResult {
                active: always_on_top,
                supported: true,
                backend: "hyprland-wayland".to_string(),
            });
        }

        Ok(AlwaysOnTopResult {
            active: false,
            supported: false,
            backend: "wayland-restricted".to_string(),
        })
    }

    #[cfg(not(target_os = "linux"))]
    {
        Ok(AlwaysOnTopResult {
            active: always_on_top,
            supported: true,
            backend: std::env::consts::OS.to_string(),
        })
    }
}

#[cfg(target_os = "linux")]
fn is_native_wayland_session() -> bool {
    if let Ok(gdk) = std::env::var("GDK_BACKEND") {
        let lower = gdk.trim().to_ascii_lowercase();
        if lower == "x11" {
            return false;
        }
        if lower == "wayland" {
            return true;
        }
    }
    let has_wayland = std::env::var("WAYLAND_DISPLAY")
        .map(|v| !v.trim().is_empty())
        .unwrap_or(false);
    let session_type = std::env::var("XDG_SESSION_TYPE")
        .unwrap_or_default()
        .to_ascii_lowercase();
    has_wayland || session_type == "wayland"
}

/// Applies `keepAbove` on KDE Plasma 5 & 6 Wayland via KWin's official D-Bus Scripting API
/// (`org.kde.KWin /Scripting`), matching the exact Sticky Note window caption.
#[cfg(target_os = "linux")]
fn try_kwin_dbus_keep_above(window_title: &str, keep_above: bool) -> bool {
    use std::fs;

    let Ok(escaped_title) = serde_json::to_string(window_title) else {
        return false;
    };
    let bool_lit = if keep_above { "true" } else { "false" };
    let script_body = format!(
        "(function () {{\n\
           var list = (typeof workspace.windowList === 'function') ? workspace.windowList() : workspace.clientList();\n\
           for (var i = 0; i < list.length; ++i) {{\n\
             var w = list[i];\n\
             if (w && w.caption === {escaped_title}) {{\n\
               w.keepAbove = {bool_lit};\n\
             }}\n\
           }}\n\
         }})();\n"
    );

    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let script_path = std::env::temp_dir().join(format!(
        "xmemos-kwin-pin-{}-{stamp}.js",
        std::process::id()
    ));

    if fs::write(&script_path, script_body).is_err() {
        return false;
    }

    let abs_path = script_path.to_string_lossy().to_string();
    let result = run_kwin_script_via_gdbus(&abs_path);
    let _ = fs::remove_file(&script_path);
    result
}

#[cfg(target_os = "linux")]
fn run_kwin_script_via_gdbus(abs_script_path: &str) -> bool {
    use std::process::Command;

    let load_out = Command::new("gdbus")
        .args([
            "call",
            "--session",
            "--dest",
            "org.kde.KWin",
            "--object-path",
            "/Scripting",
            "--method",
            "org.kde.kwin.Scripting.loadScript",
            abs_script_path,
        ])
        .output();

    let Ok(out) = load_out else {
        return false;
    };
    if !out.status.success() {
        return false;
    }

    // Parse gdbus return tuple like "(3,)" or "(int32 3,)"
    let stdout = String::from_utf8_lossy(&out.stdout);
    let script_id: Option<i64> = stdout
        .split(|c: char| !c.is_ascii_digit() && c != '-')
        .find_map(|tok| tok.parse::<i64>().ok())
        .filter(|id| *id >= 0);

    let Some(id) = script_id else {
        return false;
    };

    // Plasma 6 uses /Scripting/Script<id>; Plasma 5 uses /<id>
    let paths = [format!("/Scripting/Script{id}"), format!("/{id}")];
    let mut executed = false;
    for obj_path in &paths {
        let run_res = Command::new("gdbus")
            .args([
                "call",
                "--session",
                "--dest",
                "org.kde.KWin",
                "--object-path",
                obj_path,
                "--method",
                "org.kde.kwin.Script.run",
            ])
            .output();
        if let Ok(r) = run_res {
            if r.status.success() {
                executed = true;
                let _ = Command::new("gdbus")
                    .args([
                        "call",
                        "--session",
                        "--dest",
                        "org.kde.KWin",
                        "--object-path",
                        obj_path,
                        "--method",
                        "org.kde.kwin.Script.stop",
                    ])
                    .output();
                break;
            }
        }
    }

    executed
}

#[cfg(target_os = "linux")]
fn try_hyprland_pin(window_title: &str, _keep_above: bool) -> bool {
    use std::process::Command;

    if std::env::var("HYPRLAND_INSTANCE_SIGNATURE")
        .map(|v| v.trim().is_empty())
        .unwrap_or(true)
    {
        return false;
    }
    let escaped = regex::escape(window_title);
    Command::new("hyprctl")
        .args(["dispatch", "pin", &format!("title:^{escaped}$")])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}
