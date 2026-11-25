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

//! Zero-extra-crate native Desktop memo shortcut generator for GNU/Linux (FreeDesktop .desktop),
//! Windows 10/11 (native Shell .lnk), and macOS (native LSUIElement .app launcher bundle).

use std::fs;
use std::path::Path;
#[cfg(any(target_os = "windows", target_os = "macos"))]
use std::path::PathBuf;

use tauri::{AppHandle, Manager, Runtime};

use crate::error::{Error, Result};
use crate::models::{CreateMemoShortcutRequest, CreateMemoShortcutResponse};

const OPEN_MEMO_FLAG: &str = "--open-memo=";

/// Strictly validates a memo UID (`[A-Za-z0-9_-]`, 1..=64 chars) to prevent CLI/shell/path injection.
pub fn is_valid_memo_uid(uid: &str) -> bool {
    !uid.is_empty()
        && uid.len() <= 64
        && uid
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

/// Parses `--open-memo=<uid>` or `--open-memo <uid>` from process command-line arguments.
pub fn extract_open_memo_uid_from_args<I, S>(args: I) -> Option<String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        let s = arg.as_ref();
        if let Some(uid) = s.strip_prefix(OPEN_MEMO_FLAG) {
            if is_valid_memo_uid(uid) {
                return Some(uid.to_string());
            }
        } else if s == "--open-memo" {
            if let Some(next) = iter.next() {
                let uid = next.as_ref();
                if is_valid_memo_uid(uid) {
                    return Some(uid.to_string());
                }
            }
        }
    }
    None
}

fn sanitize_label(raw: &str, max_chars: usize, fallback: &str) -> String {
    let cleaned: String = raw
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect();
    let collapsed = cleaned.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.is_empty() {
        return fallback.to_string();
    }
    let truncated: String = collapsed.chars().take(max_chars).collect();
    truncated.trim().to_string()
}

fn sanitize_filename(raw: &str, uid: &str) -> String {
    let base: String = raw
        .chars()
        .map(|c| {
            if c.is_control()
                || matches!(
                    c,
                    '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | '`' | '$'
                )
            {
                ' '
            } else {
                c
            }
        })
        .collect();
    let collapsed = base
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim_matches('.')
        .trim()
        .to_string();
    let prefix: String = if collapsed.is_empty() {
        "Xmemos Memo".to_string()
    } else {
        collapsed.chars().take(42).collect::<String>().trim().to_string()
    };
    let short_uid: String = uid.chars().take(8).collect();
    format!("{prefix} ({short_uid})")
}

pub fn create_desktop_memo_shortcut<R: Runtime>(
    app: &AppHandle<R>,
    req: &CreateMemoShortcutRequest,
) -> Result<CreateMemoShortcutResponse> {
    if !is_valid_memo_uid(&req.memo_uid) {
        return Err(Error::Storage("Invalid memo UID for shortcut".to_string()));
    }

    let title = sanitize_label(&req.title, 48, "Xmemos Memo");
    let preview = sanitize_label(&req.preview, 120, "Open memo in Xmemos");
    let file_stem = sanitize_filename(&title, &req.memo_uid);

    let desktop_dir = app
        .path()
        .desktop_dir()
        .map_err(|e| Error::Storage(format!("Could not locate Desktop directory: {e}")))?;
    fs::create_dir_all(&desktop_dir)?;

    #[cfg(target_os = "linux")]
    {
        create_linux_desktop_entry(app, &desktop_dir, &file_stem, &title, &preview, &req.memo_uid)
    }

    #[cfg(target_os = "windows")]
    {
        create_windows_lnk_shortcut(&desktop_dir, &file_stem, &preview, &req.memo_uid)
    }

    #[cfg(target_os = "macos")]
    {
        create_macos_app_shortcut(&desktop_dir, &file_stem, &title, &req.memo_uid)
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        let _ = (desktop_dir, file_stem, title, preview);
        Ok(CreateMemoShortcutResponse {
            created: false,
            unsupported: true,
            path: None,
            message: Some("Desktop shortcuts are not supported on this OS".to_string()),
        })
    }
}

#[cfg(target_os = "linux")]
fn escape_desktop_exec_arg(arg: &str) -> String {
    let mut out = String::with_capacity(arg.len() + 2);
    out.push('"');
    for ch in arg.chars() {
        if matches!(ch, '"' | '`' | '$' | '\\') {
            out.push('\\');
        }
        out.push(ch);
    }
    out.push('"');
    out
}

#[cfg(target_os = "linux")]
fn create_linux_desktop_entry<R: Runtime>(
    app: &AppHandle<R>,
    desktop_dir: &Path,
    file_stem: &str,
    title: &str,
    preview: &str,
    memo_uid: &str,
) -> Result<CreateMemoShortcutResponse> {
    use std::os::unix::fs::PermissionsExt;

    let is_flatpak = Path::new("/.flatpak-info").exists();
    let flag = format!("{OPEN_MEMO_FLAG}{memo_uid}");

    let (exec_line, icon_value) = if is_flatpak {
        (
            format!("flatpak run org.xmemos.app {flag}"),
            "org.xmemos.app".to_string(),
        )
    } else if let Ok(appimage) = std::env::var("APPIMAGE") {
        let trimmed = appimage.trim();
        let icon = resolve_linux_icon(app).unwrap_or_else(|| "xmemos".to_string());
        (
            format!("{} {flag}", escape_desktop_exec_arg(trimmed)),
            icon,
        )
    } else {
        let exe = std::env::current_exe()
            .map_err(|e| Error::Storage(format!("Could not resolve executable path: {e}")))?;
        let icon = resolve_linux_icon(app).unwrap_or_else(|| "xmemos".to_string());
        (
            format!(
                "{} {flag}",
                escape_desktop_exec_arg(&exe.to_string_lossy())
            ),
            icon,
        )
    };

    let safe_name = title.replace(['\r', '\n'], " ");
    let safe_comment = preview.replace(['\r', '\n'], " ");

    let desktop_content = format!(
        "[Desktop Entry]\n\
         Version=1.0\n\
         Type=Application\n\
         Name={safe_name}\n\
         Comment={safe_comment}\n\
         Exec={exec_line}\n\
         Icon={icon_value}\n\
         Terminal=false\n\
         Categories=Office;Utility;\n\
         StartupNotify=true\n"
    );

    let dest = desktop_dir.join(format!("{file_stem}.desktop"));
    fs::write(&dest, desktop_content)?;
    fs::set_permissions(&dest, fs::Permissions::from_mode(0o755))?;

    Ok(CreateMemoShortcutResponse {
        created: true,
        unsupported: false,
        path: Some(dest.to_string_lossy().to_string()),
        message: None,
    })
}

#[cfg(target_os = "linux")]
fn resolve_linux_icon<R: Runtime>(app: &AppHandle<R>) -> Option<String> {
    let icon_candidates = [
        "/usr/share/icons/hicolor/scalable/apps/org.xmemos.app.svg",
        "/usr/share/icons/hicolor/512x512/apps/org.xmemos.app.png",
        "/usr/share/icons/hicolor/128x128/apps/xmemos.png",
    ];
    for c in icon_candidates {
        if Path::new(c).exists() {
            return Some(c.to_string());
        }
    }
    if let Ok(res_dir) = app.path().resource_dir() {
        let bundled = res_dir.join("icons").join("icon.png");
        if bundled.exists() {
            return Some(bundled.to_string_lossy().to_string());
        }
    }
    None
}

#[cfg(target_os = "windows")]
fn create_windows_lnk_shortcut(
    desktop_dir: &Path,
    file_stem: &str,
    preview: &str,
    memo_uid: &str,
) -> Result<CreateMemoShortcutResponse> {
    use std::os::windows::process::CommandExt;
    use std::process::Command;

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    let exe = std::env::current_exe()
        .map_err(|e| Error::Storage(format!("Could not resolve executable path: {e}")))?;
    let work_dir = exe
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| desktop_dir.to_path_buf());
    let dest = desktop_dir.join(format!("{file_stem}.lnk"));
    let args = format!("{OPEN_MEMO_FLAG}{memo_uid}");
    let icon_loc = format!("{},0", exe.to_string_lossy());

    // Pass all values strictly via environment variables to eliminate script/shell injection
    let ps_script = "$ws = New-Object -ComObject WScript.Shell; \
        $sc = $ws.CreateShortcut($env:XMEMOS_SC_PATH); \
        $sc.TargetPath = $env:XMEMOS_SC_TARGET; \
        $sc.Arguments = $env:XMEMOS_SC_ARGS; \
        $sc.WorkingDirectory = $env:XMEMOS_SC_WORKDIR; \
        $sc.Description = $env:XMEMOS_SC_DESC; \
        $sc.IconLocation = $env:XMEMOS_SC_ICON; \
        $sc.Save()";

    let output = Command::new("powershell.exe")
        .creation_flags(CREATE_NO_WINDOW)
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-WindowStyle",
            "Hidden",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            ps_script,
        ])
        .env("XMEMOS_SC_PATH", &dest)
        .env("XMEMOS_SC_TARGET", &exe)
        .env("XMEMOS_SC_ARGS", &args)
        .env("XMEMOS_SC_WORKDIR", &work_dir)
        .env("XMEMOS_SC_DESC", preview)
        .env("XMEMOS_SC_ICON", &icon_loc)
        .output()
        .map_err(|e| Error::Storage(format!("Failed to invoke Windows shortcut helper: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(Error::Storage(format!(
            "Failed to create Windows desktop shortcut: {stderr}"
        )));
    }

    Ok(CreateMemoShortcutResponse {
        created: true,
        unsupported: false,
        path: Some(dest.to_string_lossy().to_string()),
        message: None,
    })
}

#[cfg(target_os = "macos")]
fn xml_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(target_os = "macos")]
fn resolve_macos_app_bundle(exe: &Path) -> Option<PathBuf> {
    // Typical path: /Applications/Xmemos.app/Contents/MacOS/xmemos
    for ancestor in exe.ancestors() {
        if ancestor.extension().and_then(|e| e.to_str()) == Some("app") {
            return Some(ancestor.to_path_buf());
        }
    }
    None
}

#[cfg(target_os = "macos")]
fn create_macos_app_shortcut(
    desktop_dir: &Path,
    file_stem: &str,
    title: &str,
    memo_uid: &str,
) -> Result<CreateMemoShortcutResponse> {
    use std::os::unix::fs::PermissionsExt;

    let exe = std::env::current_exe()
        .map_err(|e| Error::Storage(format!("Could not resolve executable path: {e}")))?;
    let app_bundle = resolve_macos_app_bundle(&exe);

    let bundle_dir = desktop_dir.join(format!("{file_stem}.app"));
    let contents_dir = bundle_dir.join("Contents");
    let macos_dir = contents_dir.join("MacOS");
    let resources_dir = contents_dir.join("Resources");
    fs::create_dir_all(&macos_dir)?;
    fs::create_dir_all(&resources_dir)?;

    let mut has_icns = false;
    if let Some(ref bundle) = app_bundle {
        let src_icns = bundle.join("Contents").join("Resources").join("icon.icns");
        if src_icns.exists() {
            if fs::copy(&src_icns, resources_dir.join("AppIcon.icns")).is_ok() {
                has_icns = true;
            }
        }
    }

    let safe_bundle_id = format!(
        "org.xmemos.shortcut.{}",
        memo_uid
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .take(16)
            .collect::<String>()
    );
    let icon_xml = if has_icns {
        "  <key>CFBundleIconFile</key>\n  <string>AppIcon</string>\n"
    } else {
        ""
    };
    let info_plist = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
         <plist version=\"1.0\">\n\
         <dict>\n\
           <key>CFBundleDevelopmentRegion</key>\n\
           <string>en</string>\n\
           <key>CFBundleExecutable</key>\n\
           <string>launch</string>\n\
           <key>CFBundleIdentifier</key>\n\
           <string>{safe_bundle_id}</string>\n\
           <key>CFBundleName</key>\n\
           <string>{}</string>\n\
           <key>CFBundlePackageType</key>\n\
           <string>APPL</string>\n\
           <key>LSUIElement</key>\n\
           <true/>\n\
         {icon_xml}\
         </dict>\n\
         </plist>\n",
        xml_escape(title)
    );
    fs::write(contents_dir.join("Info.plist"), info_plist)?;

    let flag = format!("{OPEN_MEMO_FLAG}{memo_uid}");
    let launch_cmd = if let Some(bundle) = app_bundle {
        let escaped = bundle.to_string_lossy().replace('\'', "'\\''");
        format!("#!/bin/sh\nexec /usr/bin/open -a '{escaped}' --args '{flag}'\n")
    } else {
        let escaped = exe.to_string_lossy().replace('\'', "'\\''");
        format!("#!/bin/sh\nexec '{escaped}' '{flag}' >/dev/null 2>&1 &\n")
    };
    let launch_path = macos_dir.join("launch");
    fs::write(&launch_path, launch_cmd)?;
    fs::set_permissions(&launch_path, fs::Permissions::from_mode(0o755))?;

    Ok(CreateMemoShortcutResponse {
        created: true,
        unsupported: false,
        path: Some(bundle_dir.to_string_lossy().to_string()),
        message: None,
    })
}
