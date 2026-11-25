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

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    #[cfg(target_os = "windows")]
    if !windows_webview2::prepare_or_prompt() {
        return;
    }

    xmemos_lib::run()
}

#[cfg(target_os = "windows")]
mod windows_webview2 {
    use std::ffi::c_void;
    use std::os::windows::process::CommandExt;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    const MB_YESNO: u32 = 0x0000_0004;
    const MB_ICONINFORMATION: u32 = 0x0000_0040;
    const IDYES: i32 = 6;
    const WEBVIEW2_DOWNLOAD_URL: &str =
        "https://developer.microsoft.com/en-us/microsoft-edge/webview2/#download-section";

    #[link(name = "user32")]
    extern "system" {
        fn MessageBoxW(
            hwnd: *mut c_void,
            text: *const u16,
            caption: *const u16,
            utype: u32,
        ) -> i32;
    }

    /// Adaptive Windows WebView2 startup resolver:
    /// 1. Honors an explicit `WEBVIEW2_BROWSER_EXECUTABLE_FOLDER` if set.
    /// 2. Auto-detects a local portable Fixed-Version WebView2 folder (`./webview2/` or `./WebView2/`
    ///    next to `Xmemos.exe`) so debloated or air-gapped Windows PCs run 100% offline with zero setup.
    /// 3. Verifies system/user Evergreen WebView2 Runtime registration (independent of Edge browser).
    /// 4. If WebView2 Runtime was stripped by Windows debloating, displays a clear native dialog
    ///    with both offline portable (`webview2\`) and official Evergreen Runtime instructions.
    pub fn prepare_or_prompt() -> bool {
        if let Ok(existing) = std::env::var("WEBVIEW2_BROWSER_EXECUTABLE_FOLDER") {
            let trimmed = existing.trim();
            if !trimmed.is_empty() && Path::new(trimmed).join("msedgewebview2.exe").is_file() {
                return true;
            }
        }

        if let Some(local_dir) = find_local_fixed_webview2() {
            std::env::set_var("WEBVIEW2_BROWSER_EXECUTABLE_FOLDER", local_dir);
            return true;
        }

        if is_evergreen_webview2_installed() {
            return true;
        }

        prompt_missing_webview2();
        false
    }

    fn find_local_fixed_webview2() -> Option<PathBuf> {
        let exe = std::env::current_exe().ok()?;
        let exe_dir = exe.parent()?;

        for folder_name in ["webview2", "WebView2"] {
            let candidate = exe_dir.join(folder_name);
            if candidate.join("msedgewebview2.exe").is_file() {
                return Some(candidate);
            }
            if let Ok(entries) = std::fs::read_dir(&candidate) {
                for entry in entries.flatten() {
                    let sub = entry.path();
                    if sub.is_dir() && sub.join("msedgewebview2.exe").is_file() {
                        return Some(sub);
                    }
                }
            }
        }
        None
    }

    fn is_evergreen_webview2_installed() -> bool {
        const CLIENT_GUID: &str = "{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}";
        let keys = [
            format!(r"HKLM\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{CLIENT_GUID}"),
            format!(r"HKLM\SOFTWARE\Microsoft\EdgeUpdate\Clients\{CLIENT_GUID}"),
            format!(r"HKCU\Software\Microsoft\EdgeUpdate\Clients\{CLIENT_GUID}"),
        ];

        for key in &keys {
            let Ok(out) = Command::new("reg.exe")
                .creation_flags(CREATE_NO_WINDOW)
                .args(["query", key, "/v", "pv"])
                .output()
            else {
                continue;
            };
            if out.status.success() {
                let stdout = String::from_utf8_lossy(&out.stdout);
                if has_valid_pv_version(&stdout) {
                    return true;
                }
            }
        }
        false
    }

    fn has_valid_pv_version(reg_stdout: &str) -> bool {
        for line in reg_stdout.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("pv") {
                if let Some(version) = trimmed.split_whitespace().last() {
                    if !version.is_empty() && version != "0.0.0.0" && version != "REG_SZ" {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn to_wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    fn prompt_missing_webview2() {
        let title = to_wide("Xmemos — WebView2 Runtime Required");
        let message = to_wide(
            "Xmemos could not find Microsoft Edge WebView2 Runtime on this Windows PC.\n\n\
             Note: The Microsoft Edge browser is NOT required—only the lightweight WebView2 Runtime.\n\n\
             How to run Xmemos on debloated or offline Windows:\n\
             1. Install the official Microsoft Edge WebView2 Evergreen Standalone Runtime (one-time), OR\n\
             2. For 100% portable / zero-install use, extract Microsoft's WebView2 Fixed Version Runtime into a 'webview2' folder next to Xmemos.exe and re-open Xmemos.\n\n\
             Open the official Microsoft WebView2 Runtime download page in your default browser now?",
        );

        // SAFETY: `message` and `title` are valid null-terminated UTF-16 buffers living across the call.
        let choice = unsafe {
            MessageBoxW(
                std::ptr::null_mut(),
                message.as_ptr(),
                title.as_ptr(),
                MB_YESNO | MB_ICONINFORMATION,
            )
        };

        if choice == IDYES {
            let _ = Command::new("cmd.exe")
                .creation_flags(CREATE_NO_WINDOW)
                .args(["/C", "start", "", WEBVIEW2_DOWNLOAD_URL])
                .spawn();
        }
    }
}
