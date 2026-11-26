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

use std::fs;
use std::path::Path;

use tauri::AppHandle;
#[cfg(not(desktop))]
use tauri_plugin_xmemos_storage::{CompressVideoRequest, XmemosStorageExt};

const MIN_VIDEO_COMPRESS_BYTES: usize = 1_500_000; // Skip tiny clips (<1.5 MB)
const DEFAULT_MAX_SHORT_SIDE: u32 = 720;
#[cfg(not(desktop))]
const DEFAULT_TARGET_BITRATE_BPS: u32 = 2_200_000;

fn replace_extension(filename: &str, new_ext: &str) -> String {
    let p = Path::new(filename);
    let stem = p
        .file_stem()
        .and_then(|s| s.to_str())
        .filter(|s| !s.is_empty())
        .unwrap_or("video");
    format!("{stem}.{new_ext}")
}

/// Fail-safe, lightweight cross-platform video optimizer:
/// - Android: delegates to Jetpack Media3 Transformer (`MediaCodec` hardware H.264/AAC, 720p max short side,
///   90s watchdog timeout, anti-bloat guard requiring >=8% reduction).
/// - Desktop (GNU/Linux, Windows, macOS): opportunistically uses system-installed `ffmpeg` (zero bundled
///   binary) with H.264 CRF 26 + `-movflags +faststart` + 90s watchdog timeout + anti-bloat guard; if `ffmpeg`
///   is unavailable or fails, cleanly preserves the original video with zero error.
pub fn optimize_video_upload(
    app: &AppHandle,
    temp_dir: &Path,
    filename: &str,
    bytes: &[u8],
    mime_type: &str,
) -> (String, Vec<u8>, String) {
    let fallback = || (filename.to_string(), bytes.to_vec(), mime_type.to_string());

    if !mime_type.starts_with("video/") || bytes.len() < MIN_VIDEO_COMPRESS_BYTES {
        return fallback();
    }

    let stage_dir = temp_dir.join("video_opt");
    if fs::create_dir_all(&stage_dir).is_err() {
        return fallback();
    }

    let id = uuid::Uuid::new_v4().simple().to_string();
    let ext = Path::new(filename)
        .extension()
        .and_then(|e| e.to_str())
        .filter(|e| !e.is_empty() && e.len() <= 8 && e.chars().all(|c| c.is_ascii_alphanumeric()))
        .unwrap_or("mp4");
    let staged_input = stage_dir.join(format!("in_{id}.{ext}"));
    if fs::write(&staged_input, bytes).is_err() {
        return fallback();
    }

    let outcome = compress_staged_video(app, &stage_dir, &staged_input, &id, bytes.len());
    let _ = fs::remove_file(&staged_input);

    match outcome {
        Some((compressed_bytes, out_ext, out_mime)) => (
            replace_extension(filename, out_ext),
            compressed_bytes,
            out_mime.to_string(),
        ),
        None => fallback(),
    }
}

#[cfg(not(desktop))]
fn compress_staged_video(
    app: &AppHandle,
    _stage_dir: &Path,
    staged_input: &Path,
    _id: &str,
    orig_len: usize,
) -> Option<(Vec<u8>, &'static str, &'static str)> {
    let res = app
        .xmemos_storage()
        .compress_video(CompressVideoRequest {
            source_path: staged_input.to_string_lossy().to_string(),
            max_short_side: Some(DEFAULT_MAX_SHORT_SIDE),
            target_bitrate: Some(DEFAULT_TARGET_BITRATE_BPS),
        })
        .ok()?;

    if !res.compressed {
        return None;
    }

    let out_path = Path::new(&res.output_path);
    let data = fs::read(out_path).ok();
    if out_path != staged_input {
        let _ = fs::remove_file(out_path);
    }

    let bytes = data?;
    if !bytes.is_empty() && bytes.len() < orig_len.saturating_mul(92) / 100 {
        Some((bytes, "mp4", "video/mp4"))
    } else {
        None
    }
}

#[cfg(desktop)]
fn compress_staged_video(
    _app: &AppHandle,
    stage_dir: &Path,
    staged_input: &Path,
    id: &str,
    orig_len: usize,
) -> Option<(Vec<u8>, &'static str, &'static str)> {
    use std::process::{Command, Stdio};
    use std::time::{Duration, Instant};

    #[cfg(target_os = "windows")]
    use std::os::windows::process::CommandExt;

    let staged_output = stage_dir.join(format!("out_{id}.mp4"));
    let scale_filter = format!(
        "scale='if(gt(min(iw,ih),{s}),if(gt(iw,ih),-2,{s}),iw)':'if(gt(min(iw,ih),{s}),if(gt(iw,ih),{s},-2),ih)'",
        s = DEFAULT_MAX_SHORT_SIDE
    );

    let mut cmd = Command::new("ffmpeg");
    cmd.args(["-hide_banner", "-loglevel", "error", "-y", "-i"])
        .arg(staged_input)
        .args([
            "-vf",
            &scale_filter,
            "-c:v",
            "libx264",
            "-crf",
            "26",
            "-preset",
            "fast",
            "-pix_fmt",
            "yuv420p",
            "-c:a",
            "aac",
            "-b:a",
            "96k",
            "-movflags",
            "+faststart",
        ])
        .arg(&staged_output)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    #[cfg(target_os = "windows")]
    {
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }

    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(_) => return None, // System ffmpeg not installed -> clean passthrough
    };

    let deadline = Instant::now() + Duration::from_secs(90);
    let succeeded = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status.success(),
            Ok(None) => {
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    break false;
                }
                std::thread::sleep(Duration::from_millis(120));
            }
            Err(_) => {
                let _ = child.kill();
                break false;
            }
        }
    };

    if !succeeded {
        let _ = fs::remove_file(&staged_output);
        return None;
    }

    let data = fs::read(&staged_output).ok();
    let _ = fs::remove_file(&staged_output);

    let bytes = data?;
    if !bytes.is_empty() && bytes.len() < orig_len.saturating_mul(92) / 100 {
        Some((bytes, "mp4", "video/mp4"))
    } else {
        None
    }
}
