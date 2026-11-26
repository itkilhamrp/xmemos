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

//! Zero-extra-crate MS Sticky Notes-style icon generator for Desktop memo shortcuts:
//! - GNU/Linux: Scalable SVG sticky note card with memo title & preview text
//! - Windows 10/11: Native 128x128 RGBA PNG-compressed `.ico` sticky note card
//! - macOS: Native 128x128 RGBA `ic07` PNG-compressed `.icns` sticky note card

#[derive(Debug, Clone, Copy)]
pub struct StickyPalette {
    pub body: [u8; 3],
    pub header: [u8; 3],
    pub border: [u8; 3],
    pub fold: [u8; 3],
    pub text: [u8; 3],
    pub body_hex: &'static str,
    pub header_hex: &'static str,
    pub border_hex: &'static str,
    pub fold_hex: &'static str,
    pub text_hex: &'static str,
}

pub fn resolve_sticky_palette(color: Option<&str>) -> StickyPalette {
    match color.map(|c| c.trim().to_ascii_lowercase()).as_deref() {
        Some("blue") => StickyPalette {
            body: [219, 239, 255],
            header: [125, 200, 248],
            border: [56, 152, 224],
            fold: [167, 216, 250],
            text: [23, 56, 84],
            body_hex: "#dbefff",
            header_hex: "#7dc8f8",
            border_hex: "#3898e0",
            fold_hex: "#a7d8fa",
            text_hex: "#173854",
        },
        Some("green") => StickyPalette {
            body: [220, 252, 231],
            header: [110, 224, 162],
            border: [34, 168, 102],
            fold: [166, 239, 195],
            text: [20, 68, 43],
            body_hex: "#dcfce7",
            header_hex: "#6ee0a2",
            border_hex: "#22a866",
            fold_hex: "#a6efc3",
            text_hex: "#14442b",
        },
        Some("pink") => StickyPalette {
            body: [252, 231, 243],
            header: [244, 152, 200],
            border: [214, 78, 148],
            fold: [249, 192, 222],
            text: [86, 25, 58],
            body_hex: "#fce7f3",
            header_hex: "#f498c8",
            border_hex: "#d64e94",
            fold_hex: "#f9c0de",
            text_hex: "#56193a",
        },
        Some("purple") => StickyPalette {
            body: [237, 233, 254],
            header: [184, 162, 250],
            border: [126, 92, 232],
            fold: [212, 198, 253],
            text: [48, 30, 96],
            body_hex: "#ede9fe",
            header_hex: "#b8a2fa",
            border_hex: "#7e5ce8",
            fold_hex: "#d4c6fd",
            text_hex: "#301e60",
        },
        Some("slate") => StickyPalette {
            body: [241, 245, 249],
            header: [148, 163, 184],
            border: [100, 116, 139],
            fold: [203, 213, 225],
            text: [30, 41, 59],
            body_hex: "#f1f5f9",
            header_hex: "#94a3b8",
            border_hex: "#64748b",
            fold_hex: "#cbd5e1",
            text_hex: "#1e293b",
        },
        // Default: iconic MS Sticky Notes warm yellow
        _ => StickyPalette {
            body: [255, 245, 168],
            header: [245, 208, 66],
            border: [212, 164, 24],
            fold: [240, 212, 92],
            text: [66, 48, 12],
            body_hex: "#fff5a8",
            header_hex: "#f5d042",
            border_hex: "#d4a418",
            fold_hex: "#f0d45c",
            text_hex: "#42300c",
        },
    }
}

fn xml_escape(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            c if c.is_control() => out.push(' '),
            c => out.push(c),
        }
    }
    out
}

fn wrap_preview_lines(text: &str, max_chars: usize, max_lines: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        if lines.len() >= max_lines {
            break;
        }
        let word_len = word.chars().count();
        let cur_len = current.chars().count();
        if current.is_empty() {
            if word_len > max_chars {
                let truncated: String = word.chars().take(max_chars.saturating_sub(1)).collect();
                lines.push(format!("{truncated}…"));
            } else {
                current.push_str(word);
            }
        } else if cur_len + 1 + word_len <= max_chars {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(std::mem::take(&mut current));
            if lines.len() >= max_lines {
                break;
            }
            if word_len > max_chars {
                let truncated: String = word.chars().take(max_chars.saturating_sub(1)).collect();
                lines.push(format!("{truncated}…"));
            } else {
                current.push_str(word);
            }
        }
    }
    if !current.is_empty() && lines.len() < max_lines {
        lines.push(current);
    }
    lines
}

/// Renders a crisp 256x256 MS Sticky Notes-style SVG card with actual memo title & preview lines.
/// Used natively by GNU/Linux Desktop environments (GNOME, KDE Plasma, XFCE, Cinnamon, etc.).
#[cfg_attr(not(target_os = "linux"), allow(dead_code))]
pub fn render_sticky_note_svg(title: &str, preview: &str, color: Option<&str>) -> String {
    let p = resolve_sticky_palette(color);
    let safe_title = xml_escape(&title.chars().take(18).collect::<String>());
    let preview_lines = wrap_preview_lines(preview, 20, 4);

    let mut text_svg = String::new();
    for (idx, line) in preview_lines.iter().enumerate() {
        let y = 104 + (idx as u32) * 26;
        text_svg.push_str(&format!(
            "  <text x=\"32\" y=\"{y}\" font-family=\"sans-serif\" font-size=\"17\" font-weight=\"500\" fill=\"{}\" opacity=\"0.88\">{}</text>\n",
            p.text_hex,
            xml_escape(line)
        ));
    }

    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"256\" height=\"256\" viewBox=\"0 0 256 256\">\n\
  <defs>\n\
    <filter id=\"sh\" x=\"-8%\" y=\"-6%\" width=\"118%\" height=\"120%\">\n\
      <feDropShadow dx=\"0\" dy=\"6\" stdDeviation=\"7\" flood-color=\"#000000\" flood-opacity=\"0.24\"/>\n\
    </filter>\n\
    <clipPath id=\"noteClip\">\n\
      <path d=\"M28 18 H228 Q238 18 238 28 V186 L186 238 H28 Q18 238 18 228 V28 Q18 18 28 18 Z\"/>\n\
    </clipPath>\n\
  </defs>\n\
  <g filter=\"url(#sh)\">\n\
    <g clip-path=\"url(#noteClip)\">\n\
      <rect x=\"18\" y=\"18\" width=\"220\" height=\"220\" fill=\"{body}\"/>\n\
      <rect x=\"18\" y=\"18\" width=\"220\" height=\"48\" fill=\"{header}\"/>\n\
      <line x1=\"18\" y1=\"66\" x2=\"238\" y2=\"66\" stroke=\"{border}\" stroke-width=\"2\" opacity=\"0.45\"/>\n\
      <line x1=\"30\" y1=\"112\" x2=\"226\" y2=\"112\" stroke=\"{border}\" stroke-width=\"1.2\" opacity=\"0.25\"/>\n\
      <line x1=\"30\" y1=\"138\" x2=\"226\" y2=\"138\" stroke=\"{border}\" stroke-width=\"1.2\" opacity=\"0.25\"/>\n\
      <line x1=\"30\" y1=\"164\" x2=\"226\" y2=\"164\" stroke=\"{border}\" stroke-width=\"1.2\" opacity=\"0.25\"/>\n\
      <line x1=\"30\" y1=\"190\" x2=\"176\" y2=\"190\" stroke=\"{border}\" stroke-width=\"1.2\" opacity=\"0.25\"/>\n\
      <text x=\"32\" y=\"50\" font-family=\"sans-serif\" font-size=\"20\" font-weight=\"700\" fill=\"{text}\">{safe_title}</text>\n\
{text_svg}\
    </g>\n\
    <path d=\"M28 18 H228 Q238 18 238 28 V186 L186 238 H28 Q18 238 18 228 V28 Q18 18 28 18 Z\" fill=\"none\" stroke=\"{border}\" stroke-width=\"3\" opacity=\"0.7\"/>\n\
    <path d=\"M186 238 V196 Q186 186 196 186 H238 Z\" fill=\"{fold}\" stroke=\"{border}\" stroke-width=\"2\" stroke-linejoin=\"round\"/>\n\
  </g>\n\
</svg>\n",
        body = p.body_hex,
        header = p.header_hex,
        border = p.border_hex,
        fold = p.fold_hex,
        text = p.text_hex,
        safe_title = safe_title,
        text_svg = text_svg,
    )
}

fn crc32(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xffff_ffff;
    for &byte in data {
        crc ^= u32::from(byte);
        for _ in 0..8 {
            if (crc & 1) != 0 {
                crc = (crc >> 1) ^ 0xedb8_8320;
            } else {
                crc >>= 1;
            }
        }
    }
    !crc
}

fn adler32(data: &[u8]) -> u32 {
    const MOD_ADLER: u32 = 65521;
    let mut a: u32 = 1;
    let mut b: u32 = 0;
    for &byte in data {
        a = (a + u32::from(byte)) % MOD_ADLER;
        b = (b + a) % MOD_ADLER;
    }
    (b << 16) | a
}

fn write_png_chunk(out: &mut Vec<u8>, chunk_type: &[u8; 4], payload: &[u8]) {
    out.extend_from_slice(&(payload.len() as u32).to_be_bytes());
    out.extend_from_slice(chunk_type);
    out.extend_from_slice(payload);
    let mut crc_buf = Vec::with_capacity(4 + payload.len());
    crc_buf.extend_from_slice(chunk_type);
    crc_buf.extend_from_slice(payload);
    out.extend_from_slice(&crc32(&crc_buf).to_be_bytes());
}

/// Encodes an RGBA pixel buffer into a 100% standards-compliant PNG using stored zlib blocks
/// (zero external dependencies, deterministic output).
fn encode_rgba_png(width: u32, height: u32, rgba: &[u8]) -> Vec<u8> {
    let row_bytes = (width as usize) * 4;
    let mut raw = Vec::with_capacity((row_bytes + 1) * (height as usize));
    for y in 0..(height as usize) {
        raw.push(0u8); // filter type 0 (None)
        let start = y * row_bytes;
        raw.extend_from_slice(&rgba[start..start + row_bytes]);
    }

    // Wrap raw scanlines in a valid zlib stream (CMF=0x78, FLG=0x01, uncompressed deflate blocks)
    let mut zlib = Vec::with_capacity(raw.len() + 64);
    zlib.push(0x78);
    zlib.push(0x01);
    let max_block = 65535usize;
    let mut offset = 0usize;
    while offset < raw.len() {
        let end = (offset + max_block).min(raw.len());
        let is_final = end == raw.len();
        let len = (end - offset) as u16;
        zlib.push(if is_final { 0x01 } else { 0x00 });
        zlib.extend_from_slice(&len.to_le_bytes());
        zlib.extend_from_slice(&(!len).to_le_bytes());
        zlib.extend_from_slice(&raw[offset..end]);
        offset = end;
    }
    zlib.extend_from_slice(&adler32(&raw).to_be_bytes());

    let mut png = Vec::with_capacity(zlib.len() + 128);
    png.extend_from_slice(&[137, 80, 78, 71, 13, 10, 26, 10]);

    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&width.to_be_bytes());
    ihdr.extend_from_slice(&height.to_be_bytes());
    ihdr.push(8); // bit depth
    ihdr.push(6); // color type RGBA
    ihdr.push(0); // compression
    ihdr.push(0); // filter
    ihdr.push(0); // interlace
    write_png_chunk(&mut png, b"IHDR", &ihdr);
    write_png_chunk(&mut png, b"IDAT", &zlib);
    write_png_chunk(&mut png, b"IEND", &[]);
    png
}

/// Renders a 128x128 RGBA MS Sticky Notes card bitmap (header strip, ruled note bars, folded corner, drop shadow).
pub fn render_sticky_note_png_128(color: Option<&str>) -> Vec<u8> {
    const SIZE: usize = 128;
    let p = resolve_sticky_palette(color);
    let mut rgba = vec![0u8; SIZE * SIZE * 4];

    let left = 10i32;
    let top = 8i32;
    let right = 117i32;
    let bottom = 115i32;
    let header_bottom = 32i32;
    let fold_size = 24i32;

    let inside_note = |x: i32, y: i32| -> bool {
        if x < left || x > right || y < top || y > bottom {
            return false;
        }
        // Cut bottom-right triangle for folded corner
        let dx = right - x;
        let dy = bottom - y;
        if dx + dy < fold_size {
            return false;
        }
        // Soft 6px rounded corners on top-left, top-right, bottom-left
        let r = 6i32;
        if x < left + r && y < top + r {
            let cx = left + r - x;
            let cy = top + r - y;
            return cx * cx + cy * cy <= r * r;
        }
        if x > right - r && y < top + r {
            let cx = x - (right - r);
            let cy = top + r - y;
            return cx * cx + cy * cy <= r * r;
        }
        if x < left + r && y > bottom - r {
            let cx = left + r - x;
            let cy = y - (bottom - r);
            return cx * cx + cy * cy <= r * r;
        }
        true
    };

    for y in 0..SIZE as i32 {
        for x in 0..SIZE as i32 {
            let idx = ((y as usize) * SIZE + (x as usize)) * 4;

            if inside_note(x, y) {
                let is_border = !inside_note(x - 1, y)
                    || !inside_note(x + 1, y)
                    || !inside_note(x, y - 1)
                    || !inside_note(x, y + 1);
                let in_fold_box = x >= right - fold_size && y >= bottom - fold_size;

                let rgb = if is_border {
                    p.border
                } else if y <= header_bottom {
                    if y == header_bottom {
                        p.border
                    } else {
                        p.header
                    }
                } else if in_fold_box {
                    if x == right - fold_size || y == bottom - fold_size {
                        p.border
                    } else {
                        p.fold
                    }
                } else {
                    // Stylised memo text lines inside the sticky note body
                    let is_line = (y >= 46 && y <= 50 && x >= 22 && x <= 96)
                        || (y >= 62 && y <= 66 && x >= 22 && x <= 104)
                        || (y >= 78 && y <= 82 && x >= 22 && x <= 90)
                        || (y >= 94 && y <= 98 && x >= 22 && x <= 74);
                    if is_line {
                        // Soften text line ink over body
                        [
                            ((u16::from(p.text[0]) * 3 + u16::from(p.body[0]) * 2) / 5) as u8,
                            ((u16::from(p.text[1]) * 3 + u16::from(p.body[1]) * 2) / 5) as u8,
                            ((u16::from(p.text[2]) * 3 + u16::from(p.body[2]) * 2) / 5) as u8,
                        ]
                    } else {
                        p.body
                    }
                };

                rgba[idx] = rgb[0];
                rgba[idx + 1] = rgb[1];
                rgba[idx + 2] = rgb[2];
                rgba[idx + 3] = 255;
            } else if inside_note(x - 2, y - 4) {
                // Subtle drop shadow beneath the sticky note
                rgba[idx] = 0;
                rgba[idx + 1] = 0;
                rgba[idx + 2] = 0;
                rgba[idx + 3] = 52;
            }
        }
    }

    encode_rgba_png(SIZE as u32, SIZE as u32, &rgba)
}

/// Wraps the 128x128 Sticky Note PNG into a standards-compliant Windows `.ico` file (Vista/7/10/11).
#[cfg_attr(not(target_os = "windows"), allow(dead_code))]
pub fn build_sticky_note_ico(color: Option<&str>) -> Vec<u8> {
    let png = render_sticky_note_png_128(color);
    let mut ico = Vec::with_capacity(22 + png.len());
    // ICONDIR (6 bytes)
    ico.extend_from_slice(&0u16.to_le_bytes()); // reserved
    ico.extend_from_slice(&1u16.to_le_bytes()); // type = ICO
    ico.extend_from_slice(&1u16.to_le_bytes()); // count = 1
    // ICONDIRENTRY (16 bytes)
    ico.push(128); // width
    ico.push(128); // height
    ico.push(0); // color count
    ico.push(0); // reserved
    ico.extend_from_slice(&1u16.to_le_bytes()); // planes
    ico.extend_from_slice(&32u16.to_le_bytes()); // bpp
    ico.extend_from_slice(&(png.len() as u32).to_le_bytes()); // bytes in res
    ico.extend_from_slice(&22u32.to_le_bytes()); // offset
    ico.extend_from_slice(&png);
    ico
}

/// Wraps the 128x128 Sticky Note PNG into a standards-compliant macOS `.icns` file (`ic07` chunk).
#[cfg_attr(not(target_os = "macos"), allow(dead_code))]
pub fn build_sticky_note_icns(color: Option<&str>) -> Vec<u8> {
    let png = render_sticky_note_png_128(color);
    let chunk_len = (8 + png.len()) as u32;
    let total_len = 8 + chunk_len;
    let mut icns = Vec::with_capacity(total_len as usize);
    icns.extend_from_slice(b"icns");
    icns.extend_from_slice(&total_len.to_be_bytes());
    icns.extend_from_slice(b"ic07"); // 128x128 PNG chunk
    icns.extend_from_slice(&chunk_len.to_be_bytes());
    icns.extend_from_slice(&png);
    icns
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_valid_sticky_svg_ico_and_icns() {
        let svg = render_sticky_note_svg("Shopping List", "Milk, eggs, coffee beans", Some("yellow"));
        assert!(svg.contains("<svg"));
        assert!(svg.contains("Shopping List"));
        assert!(svg.contains("#fff5a8"));

        let ico = build_sticky_note_ico(Some("blue"));
        assert_eq!(&ico[0..6], &[0, 0, 1, 0, 1, 0]);
        assert_eq!(&ico[22..30], &[137, 80, 78, 71, 13, 10, 26, 10]);

        let icns = build_sticky_note_icns(Some("green"));
        assert_eq!(&icns[0..4], b"icns");
        assert_eq!(&icns[8..12], b"ic07");
        assert_eq!(&icns[16..24], &[137, 80, 78, 71, 13, 10, 26, 10]);
    }
}
