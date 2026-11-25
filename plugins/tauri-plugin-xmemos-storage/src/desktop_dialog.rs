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

//! Zero-extra-dependency native OS File Manager dialogs for Desktop targets:
//! - GNU/Linux (AppImage & Flatpak): `GtkFileChooserNative` via GTK 3 (automatically routes through
//!   `org.freedesktop.portal.FileChooser` inside Flatpak sandboxes and modern Wayland/X11 sessions).
//! - Windows 10/11: Win32 `comdlg32.dll` (`GetSaveFileNameW` / `GetOpenFileNameW`) with full UTF-16
//!   Unicode path support and overwrite confirmation.
//! - macOS: Native Finder `choose file name` / `choose file` sheet via `/usr/bin/osascript` with
//!   strict `argv` parameter passing.

use std::path::PathBuf;
use tauri::{AppHandle, Manager, Runtime};

use crate::error::Result;

/// Opens the native OS File Manager Save dialog and returns `Ok(Some(path))` if confirmed,
/// or `Ok(None)` if the user cancelled.
pub fn save_file_dialog<R: Runtime>(
    app: &AppHandle<R>,
    title: &str,
    suggested_filename: &str,
    mime_type: &str,
) -> Result<Option<PathBuf>> {
    let initial_dir = app
        .path()
        .download_dir()
        .or_else(|_| app.path().document_dir())
        .or_else(|_| app.path().home_dir())
        .ok();

    platform::save_file_dialog(app, title, suggested_filename, mime_type, initial_dir.as_deref())
}

/// Opens the native OS File Manager Open dialog and returns `Ok(Some(path))` if a file was picked,
/// or `Ok(None)` if the user cancelled.
pub fn open_file_dialog<R: Runtime>(
    app: &AppHandle<R>,
    title: &str,
    mime_types: &[String],
) -> Result<Option<PathBuf>> {
    let initial_dir = app
        .path()
        .download_dir()
        .or_else(|_| app.path().document_dir())
        .or_else(|_| app.path().home_dir())
        .ok();

    platform::open_file_dialog(app, title, mime_types, initial_dir.as_deref())
}

#[cfg(target_os = "linux")]
mod platform {
    use std::ffi::{c_char, c_int, c_void, CStr, CString, OsString};
    use std::os::unix::ffi::{OsStrExt, OsStringExt};
    use std::path::{Path, PathBuf};
    use std::ptr;
    use std::sync::mpsc;

    use tauri::{AppHandle, Runtime};

    use crate::error::{Error, Result};

    const GTK_FILE_CHOOSER_ACTION_OPEN: c_int = 0;
    const GTK_FILE_CHOOSER_ACTION_SAVE: c_int = 1;
    const GTK_RESPONSE_ACCEPT: c_int = -3;

    #[link(name = "gtk-3")]
    extern "C" {
        fn gtk_file_chooser_native_new(
            title: *const c_char,
            parent: *mut c_void,
            action: c_int,
            accept_label: *const c_char,
            cancel_label: *const c_char,
        ) -> *mut c_void;
        fn gtk_file_chooser_set_current_name(chooser: *mut c_void, name: *const c_char);
        fn gtk_file_chooser_set_current_folder(chooser: *mut c_void, filename: *const c_char) -> c_int;
        fn gtk_file_chooser_set_do_overwrite_confirmation(chooser: *mut c_void, confirm: c_int);
        fn gtk_file_chooser_get_filename(chooser: *mut c_void) -> *mut c_char;
        fn gtk_native_dialog_run(dialog: *mut c_void) -> c_int;
    }

    #[link(name = "glib-2.0")]
    extern "C" {
        fn g_free(mem: *mut c_void);
    }

    #[link(name = "gobject-2.0")]
    extern "C" {
        fn g_object_unref(object: *mut c_void);
    }

    fn to_cstring(s: &str) -> CString {
        let clean: String = s.chars().filter(|&c| c != '\0').collect();
        CString::new(clean).unwrap_or_else(|_| CString::new("Xmemos").expect("static cstr"))
    }

    pub fn save_file_dialog<R: Runtime>(
        app: &AppHandle<R>,
        title: &str,
        suggested_filename: &str,
        _mime_type: &str,
        initial_dir: Option<&Path>,
    ) -> Result<Option<PathBuf>> {
        let title_c = to_cstring(title);
        let name_c = to_cstring(suggested_filename);
        let dir_c = initial_dir.and_then(|d| CString::new(d.as_os_str().as_bytes()).ok());
        let accept_c = CString::new("_Save").expect("static cstr");
        let cancel_c = CString::new("_Cancel").expect("static cstr");

        let (tx, rx) = mpsc::channel();
        app.run_on_main_thread(move || {
            // SAFETY: Dispatched onto the GTK main thread owned by Tauri/Tao on GNU/Linux.
            // GtkFileChooserNative automatically uses xdg-desktop-portal inside Flatpak.
            let result = unsafe {
                let native = gtk_file_chooser_native_new(
                    title_c.as_ptr(),
                    ptr::null_mut(),
                    GTK_FILE_CHOOSER_ACTION_SAVE,
                    accept_c.as_ptr(),
                    cancel_c.as_ptr(),
                );
                if native.is_null() {
                    None
                } else {
                    gtk_file_chooser_set_do_overwrite_confirmation(native, 1);
                    gtk_file_chooser_set_current_name(native, name_c.as_ptr());
                    if let Some(ref dir) = dir_c {
                        gtk_file_chooser_set_current_folder(native, dir.as_ptr());
                    }
                    let resp = gtk_native_dialog_run(native);
                    let picked = if resp == GTK_RESPONSE_ACCEPT {
                        let raw = gtk_file_chooser_get_filename(native);
                        if raw.is_null() {
                            None
                        } else {
                            let bytes = CStr::from_ptr(raw).to_bytes().to_vec();
                            g_free(raw.cast::<c_void>());
                            Some(PathBuf::from(OsString::from_vec(bytes)))
                        }
                    } else {
                        None
                    };
                    g_object_unref(native);
                    picked
                }
            };
            let _ = tx.send(result);
        })
        .map_err(|e| Error::Storage(format!("Failed to open Linux Save File dialog: {e}")))?;

        rx.recv()
            .map_err(|e| Error::Storage(format!("Linux Save File dialog channel closed: {e}")))
    }

    pub fn open_file_dialog<R: Runtime>(
        app: &AppHandle<R>,
        title: &str,
        _mime_types: &[String],
        initial_dir: Option<&Path>,
    ) -> Result<Option<PathBuf>> {
        let title_c = to_cstring(title);
        let dir_c = initial_dir.and_then(|d| CString::new(d.as_os_str().as_bytes()).ok());
        let accept_c = CString::new("_Open").expect("static cstr");
        let cancel_c = CString::new("_Cancel").expect("static cstr");

        let (tx, rx) = mpsc::channel();
        app.run_on_main_thread(move || {
            // SAFETY: Dispatched onto the GTK main thread owned by Tauri/Tao on GNU/Linux.
            let result = unsafe {
                let native = gtk_file_chooser_native_new(
                    title_c.as_ptr(),
                    ptr::null_mut(),
                    GTK_FILE_CHOOSER_ACTION_OPEN,
                    accept_c.as_ptr(),
                    cancel_c.as_ptr(),
                );
                if native.is_null() {
                    None
                } else {
                    if let Some(ref dir) = dir_c {
                        gtk_file_chooser_set_current_folder(native, dir.as_ptr());
                    }
                    let resp = gtk_native_dialog_run(native);
                    let picked = if resp == GTK_RESPONSE_ACCEPT {
                        let raw = gtk_file_chooser_get_filename(native);
                        if raw.is_null() {
                            None
                        } else {
                            let bytes = CStr::from_ptr(raw).to_bytes().to_vec();
                            g_free(raw.cast::<c_void>());
                            Some(PathBuf::from(OsString::from_vec(bytes)))
                        }
                    } else {
                        None
                    };
                    g_object_unref(native);
                    picked
                }
            };
            let _ = tx.send(result);
        })
        .map_err(|e| Error::Storage(format!("Failed to open Linux Open File dialog: {e}")))?;

        rx.recv()
            .map_err(|e| Error::Storage(format!("Linux Open File dialog channel closed: {e}")))
    }
}

#[cfg(target_os = "windows")]
mod platform {
    use std::ffi::{c_void, OsString};
    use std::os::windows::ffi::{OsStrExt, OsStringExt};
    use std::path::{Path, PathBuf};
    use std::ptr;

    use tauri::{AppHandle, Runtime};

    use crate::error::Result;

    const OFN_OVERWRITEPROMPT: u32 = 0x0000_0002;
    const OFN_NOCHANGEDIR: u32 = 0x0000_0008;
    const OFN_PATHMUSTEXIST: u32 = 0x0000_0800;
    const OFN_FILEMUSTEXIST: u32 = 0x0000_1000;
    const OFN_EXPLORER: u32 = 0x0008_0000;

    #[repr(C)]
    struct OpenFileNameW {
        l_struct_size: u32,
        hwnd_owner: *mut c_void,
        h_instance: *mut c_void,
        lpstr_filter: *const u16,
        lpstr_custom_filter: *mut u16,
        n_max_cust_filter: u32,
        n_filter_index: u32,
        lpstr_file: *mut u16,
        n_max_file: u32,
        lpstr_file_title: *mut u16,
        n_max_file_title: u32,
        lpstr_initial_dir: *const u16,
        lpstr_title: *const u16,
        flags: u32,
        n_file_offset: u16,
        n_file_extension: u16,
        lpstr_def_ext: *const u16,
        l_cust_data: isize,
        lpfn_hook: *const c_void,
        lp_template_name: *const u16,
        pv_reserved: *mut c_void,
        dw_reserved: u32,
        flags_ex: u32,
    }

    #[link(name = "comdlg32")]
    extern "system" {
        fn GetSaveFileNameW(lpofn: *mut OpenFileNameW) -> i32;
        fn GetOpenFileNameW(lpofn: *mut OpenFileNameW) -> i32;
    }

    fn to_wide_null(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    fn path_to_wide_null(p: &Path) -> Vec<u16> {
        p.as_os_str()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }

    fn build_save_filter(filename: &str) -> (Vec<u16>, Option<Vec<u16>>) {
        let ext = Path::new(filename)
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.trim().to_ascii_lowercase())
            .filter(|e| !e.is_empty() && e.chars().all(|c| c.is_ascii_alphanumeric()));

        let mut filter = Vec::new();
        let def_ext = if let Some(ref ext_str) = ext {
            let label = format!("{} File (*.{ext_str})", ext_str.to_ascii_uppercase());
            let pattern = format!("*.{ext_str}");
            filter.extend(label.encode_utf16());
            filter.push(0);
            filter.extend(pattern.encode_utf16());
            filter.push(0);
            Some(to_wide_null(ext_str))
        } else {
            None
        };
        filter.extend("All Files (*.*)".encode_utf16());
        filter.push(0);
        filter.extend("*.*".encode_utf16());
        filter.push(0);
        filter.push(0);
        (filter, def_ext)
    }

    fn wide_buf_to_path(buf: &[u16]) -> Option<PathBuf> {
        let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
        if end == 0 {
            None
        } else {
            Some(PathBuf::from(OsString::from_wide(&buf[..end])))
        }
    }

    pub fn save_file_dialog<R: Runtime>(
        _app: &AppHandle<R>,
        title: &str,
        suggested_filename: &str,
        _mime_type: &str,
        initial_dir: Option<&Path>,
    ) -> Result<Option<PathBuf>> {
        let mut file_buf = vec![0u16; 32_768];
        for (idx, unit) in suggested_filename.encode_utf16().take(file_buf.len() - 1).enumerate() {
            file_buf[idx] = unit;
        }
        let title_w = to_wide_null(title);
        let dir_w = initial_dir.map(path_to_wide_null);
        let (filter_w, def_ext_w) = build_save_filter(suggested_filename);

        let mut ofn = OpenFileNameW {
            l_struct_size: std::mem::size_of::<OpenFileNameW>() as u32,
            hwnd_owner: ptr::null_mut(),
            h_instance: ptr::null_mut(),
            lpstr_filter: filter_w.as_ptr(),
            lpstr_custom_filter: ptr::null_mut(),
            n_max_cust_filter: 0,
            n_filter_index: 1,
            lpstr_file: file_buf.as_mut_ptr(),
            n_max_file: file_buf.len() as u32,
            lpstr_file_title: ptr::null_mut(),
            n_max_file_title: 0,
            lpstr_initial_dir: dir_w.as_ref().map_or(ptr::null(), |v| v.as_ptr()),
            lpstr_title: title_w.as_ptr(),
            flags: OFN_EXPLORER | OFN_PATHMUSTEXIST | OFN_OVERWRITEPROMPT | OFN_NOCHANGEDIR,
            n_file_offset: 0,
            n_file_extension: 0,
            lpstr_def_ext: def_ext_w.as_ref().map_or(ptr::null(), |v| v.as_ptr()),
            l_cust_data: 0,
            lpfn_hook: ptr::null(),
            lp_template_name: ptr::null(),
            pv_reserved: ptr::null_mut(),
            dw_reserved: 0,
            flags_ex: 0,
        };

        // SAFETY: All pointers in OpenFileNameW remain valid for the duration of GetSaveFileNameW.
        let ok = unsafe { GetSaveFileNameW(&mut ofn) };
        if ok != 0 {
            Ok(wide_buf_to_path(&file_buf))
        } else {
            Ok(None)
        }
    }

    pub fn open_file_dialog<R: Runtime>(
        _app: &AppHandle<R>,
        title: &str,
        _mime_types: &[String],
        initial_dir: Option<&Path>,
    ) -> Result<Option<PathBuf>> {
        let mut file_buf = vec![0u16; 32_768];
        let title_w = to_wide_null(title);
        let dir_w = initial_dir.map(path_to_wide_null);
        let mut filter_w: Vec<u16> = Vec::new();
        filter_w.extend("All Files (*.*)".encode_utf16());
        filter_w.push(0);
        filter_w.extend("*.*".encode_utf16());
        filter_w.push(0);
        filter_w.push(0);

        let mut ofn = OpenFileNameW {
            l_struct_size: std::mem::size_of::<OpenFileNameW>() as u32,
            hwnd_owner: ptr::null_mut(),
            h_instance: ptr::null_mut(),
            lpstr_filter: filter_w.as_ptr(),
            lpstr_custom_filter: ptr::null_mut(),
            n_max_cust_filter: 0,
            n_filter_index: 1,
            lpstr_file: file_buf.as_mut_ptr(),
            n_max_file: file_buf.len() as u32,
            lpstr_file_title: ptr::null_mut(),
            n_max_file_title: 0,
            lpstr_initial_dir: dir_w.as_ref().map_or(ptr::null(), |v| v.as_ptr()),
            lpstr_title: title_w.as_ptr(),
            flags: OFN_EXPLORER | OFN_PATHMUSTEXIST | OFN_FILEMUSTEXIST | OFN_NOCHANGEDIR,
            n_file_offset: 0,
            n_file_extension: 0,
            lpstr_def_ext: ptr::null(),
            l_cust_data: 0,
            lpfn_hook: ptr::null(),
            lp_template_name: ptr::null(),
            pv_reserved: ptr::null_mut(),
            dw_reserved: 0,
            flags_ex: 0,
        };

        // SAFETY: All pointers in OpenFileNameW remain valid for the duration of GetOpenFileNameW.
        let ok = unsafe { GetOpenFileNameW(&mut ofn) };
        if ok != 0 {
            Ok(wide_buf_to_path(&file_buf))
        } else {
            Ok(None)
        }
    }
}

#[cfg(target_os = "macos")]
mod platform {
    use std::path::{Path, PathBuf};
    use std::process::Command;

    use tauri::{AppHandle, Runtime};

    use crate::error::{Error, Result};

    pub fn save_file_dialog<R: Runtime>(
        _app: &AppHandle<R>,
        title: &str,
        suggested_filename: &str,
        _mime_type: &str,
        initial_dir: Option<&Path>,
    ) -> Result<Option<PathBuf>> {
        let mut cmd = Command::new("/usr/bin/osascript");
        if let Some(dir) = initial_dir {
            cmd.arg("-e")
                .arg("on run argv")
                .arg("-e")
                .arg("POSIX path of (choose file name with prompt (item 1 of argv) default name (item 2 of argv) default location (POSIX file (item 3 of argv)))")
                .arg("-e")
                .arg("end run")
                .arg("--")
                .arg(title)
                .arg(suggested_filename)
                .arg(dir.as_os_str());
        } else {
            cmd.arg("-e")
                .arg("on run argv")
                .arg("-e")
                .arg("POSIX path of (choose file name with prompt (item 1 of argv) default name (item 2 of argv))")
                .arg("-e")
                .arg("end run")
                .arg("--")
                .arg(title)
                .arg(suggested_filename);
        }

        let output = cmd
            .output()
            .map_err(|e| Error::Storage(format!("Failed to launch macOS Save dialog: {e}")))?;

        if !output.status.success() {
            return Ok(None);
        }
        let raw = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if raw.is_empty() {
            Ok(None)
        } else {
            Ok(Some(PathBuf::from(raw)))
        }
    }

    pub fn open_file_dialog<R: Runtime>(
        _app: &AppHandle<R>,
        title: &str,
        _mime_types: &[String],
        initial_dir: Option<&Path>,
    ) -> Result<Option<PathBuf>> {
        let mut cmd = Command::new("/usr/bin/osascript");
        if let Some(dir) = initial_dir {
            cmd.arg("-e")
                .arg("on run argv")
                .arg("-e")
                .arg("POSIX path of (choose file with prompt (item 1 of argv) default location (POSIX file (item 2 of argv)))")
                .arg("-e")
                .arg("end run")
                .arg("--")
                .arg(title)
                .arg(dir.as_os_str());
        } else {
            cmd.arg("-e")
                .arg("on run argv")
                .arg("-e")
                .arg("POSIX path of (choose file with prompt (item 1 of argv))")
                .arg("-e")
                .arg("end run")
                .arg("--")
                .arg(title);
        }

        let output = cmd
            .output()
            .map_err(|e| Error::Storage(format!("Failed to launch macOS Open dialog: {e}")))?;

        if !output.status.success() {
            return Ok(None);
        }
        let raw = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if raw.is_empty() {
            Ok(None)
        } else {
            Ok(Some(PathBuf::from(raw)))
        }
    }
}

#[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
mod platform {
    use std::path::{Path, PathBuf};
    use tauri::{AppHandle, Runtime};
    use crate::error::Result;

    pub fn save_file_dialog<R: Runtime>(
        _app: &AppHandle<R>,
        _title: &str,
        _suggested_filename: &str,
        _mime_type: &str,
        _initial_dir: Option<&Path>,
    ) -> Result<Option<PathBuf>> {
        Ok(None)
    }

    pub fn open_file_dialog<R: Runtime>(
        _app: &AppHandle<R>,
        _title: &str,
        _mime_types: &[String],
        _initial_dir: Option<&Path>,
    ) -> Result<Option<PathBuf>> {
        Ok(None)
    }
}
