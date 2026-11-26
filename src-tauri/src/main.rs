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
    #[cfg(target_os = "linux")]
    linux_de_startup::apply_native_compositor_hints();

    #[cfg(target_os = "windows")]
    if !windows_webview2::prepare_or_prompt() {
        return;
    }

    xmemos_lib::run()
}

#[cfg(target_os = "linux")]
mod linux_de_startup {
    use std::path::Path;

    /// Lightweight, zero-subprocess GNU/Linux Desktop Environment & display-server startup tuning.
    /// Prevents multi-second WebKitGTK blank-window stalls on NVIDIA + Wayland (KDE Plasma, GNOME,
    /// Hyprland, Sway) per official Tauri v2 Linux Graphics guidelines, while strictly honoring
    /// any explicit user environment overrides.
    pub fn apply_native_compositor_hints() {
        let is_wayland = std::env::var_os("WAYLAND_DISPLAY").is_some()
            || std::env::var("XDG_SESSION_TYPE")
                .map(|v| v.eq_ignore_ascii_case("wayland"))
                .unwrap_or(false);

        let has_nvidia = Path::new("/proc/driver/nvidia/version").exists();

        if is_wayland && has_nvidia && std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_none()
        {
            std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
        }
    }
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

    const HKEY_CURRENT_USER: isize = 0x8000_0001u32 as isize;
    const HKEY_LOCAL_MACHINE: isize = 0x8000_0002u32 as isize;
    const KEY_READ: u32 = 0x0002_0019;
    const ERROR_SUCCESS: u32 = 0;

    #[link(name = "user32")]
    extern "system" {
        fn MessageBoxW(
            hwnd: *mut c_void,
            text: *const u16,
            caption: *const u16,
            utype: u32,
        ) -> i32;
    }

    #[link(name = "advapi32")]
    extern "system" {
        fn RegOpenKeyExW(
            hkey: isize,
            lp_sub_key: *const u16,
            ul_options: u32,
            sam_desired: u32,
            phk_result: *mut isize,
        ) -> u32;

        fn RegQueryValueExW(
            hkey: isize,
            lp_value_name: *const u16,
            lp_reserved: *mut u32,
            lp_type: *mut u32,
            lp_data: *mut u8,
            lpcb_data: *mut u32,
        ) -> u32;

        fn RegCloseKey(hkey: isize) -> u32;
    }

    /// Adaptive, zero-subprocess Windows WebView2 startup resolver:
    /// 1. Applies lightweight WebView2 cold-start flags (if unset by user).
    /// 2. Honors an explicit `WEBVIEW2_BROWSER_EXECUTABLE_FOLDER` if set.
    /// 3. Auto-detects a local portable Fixed-Version WebView2 folder (`./webview2/` or `./WebView2/`
    ///    next to `Xmemos.exe`) so debloated or air-gapped Windows PCs run 100% offline with zero setup.
    /// 4. Queries Windows Registry directly via native Win32 `advapi32.dll` (<0.1ms, zero `reg.exe` spawn).
    /// 5. If WebView2 Runtime is missing, shows a clear native Win32 dialog with offline & Evergreen steps.
    pub fn prepare_or_prompt() -> bool {
        if std::env::var_os("WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS").is_none() {
            std::env::set_var(
                "WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS",
                "--disable-background-networking --no-first-run",
            );
        }

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

    fn read_reg_string(root: isize, subkey: &str, value_name: &str) -> Option<String> {
        let subkey_w = to_wide(subkey);
        let value_w = to_wide(value_name);
        let mut hkey: isize = 0;

        // SAFETY: `subkey_w` is a valid null-terminated UTF-16 string and `hkey` is stack-allocated.
        let status = unsafe {
            RegOpenKeyExW(root, subkey_w.as_ptr(), 0, KEY_READ, &mut hkey)
        };
        if status != ERROR_SUCCESS {
            return None;
        }

        let mut buf = [0u16; 128];
        let mut buf_bytes = (buf.len() * std::mem::size_of::<u16>()) as u32;
        let mut reg_type: u32 = 0;

        // SAFETY: `hkey` was opened cleanly above; `buf` is valid for `buf_bytes` bytes.
        let q_status = unsafe {
            RegQueryValueExW(
                hkey,
                value_w.as_ptr(),
                std::ptr::null_mut(),
                &mut reg_type,
                buf.as_mut_ptr() as *mut u8,
                &mut buf_bytes,
            )
        };
        // SAFETY: Closing the valid registry handle obtained from `RegOpenKeyExW`.
        unsafe {
            RegCloseKey(hkey);
        }

        if q_status != ERROR_SUCCESS || buf_bytes < 2 {
            return None;
        }
        let u16_len = (buf_bytes as usize / 2).min(buf.len());
        let slice = &buf[..u16_len];
        let trimmed_len = slice.iter().position(|&c| c == 0).unwrap_or(slice.len());
        String::from_utf16(&slice[..trimmed_len]).ok()
    }

    fn is_evergreen_webview2_installed() -> bool {
        const CLIENT_GUID: &str = "{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}";
        let lookups = [
            (
                HKEY_LOCAL_MACHINE,
                format!(r"SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{CLIENT_GUID}"),
            ),
            (
                HKEY_LOCAL_MACHINE,
                format!(r"SOFTWARE\Microsoft\EdgeUpdate\Clients\{CLIENT_GUID}"),
            ),
            (
                HKEY_CURRENT_USER,
                format!(r"Software\Microsoft\EdgeUpdate\Clients\{CLIENT_GUID}"),
            ),
        ];

        for (root, subkey) in &lookups {
            if let Some(version) = read_reg_string(*root, subkey, "pv") {
                let v = version.trim();
                if !v.is_empty() && v != "0.0.0.0" {
                    return true;
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
            "Xmemos could not find Microsoft Edge WebView2 Runtime on this Windows PC.

             Note: The Microsoft Edge browser is NOT required—only the lightweight WebView2 Runtime.

             How to run Xmemos on debloated or offline Windows:
             1. Install the official Microsoft Edge WebView2 Evergreen Standalone Runtime (one-time), OR
             2. For 100% portable / zero-install use, extract Microsoft's WebView2 Fixed Version Runtime into a 'webview2' folder next to Xmemos.exe and re-open Xmemos.

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
