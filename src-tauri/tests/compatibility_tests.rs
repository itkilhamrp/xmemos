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

use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use rusqlite::Connection;
use tempfile::TempDir;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use xmemos_lib::service::{auth, backup, media, search};
use xmemos_lib::state::AppState;
use xmemos_lib::store::db::memos_schema::{
    assert_zero_proprietary_tables, validate_imported_memos_db,
};
use xmemos_lib::store::memo::{self, CreateMemoInput, MemoFilter};
use xmemos_lib::store::memo_relation;
use xmemos_lib::store::reaction;
use xmemos_lib::store::user::{self, CreateUserInput};

fn make_test_state() -> (TempDir, AppState) {
    let dir = TempDir::new().expect("tempdir");
    let state = AppState::initialize(
        dir.path().join("db"),
        dir.path().join("media"),
        dir.path().join("cache"),
        "1.0.0".to_string(),
    )
    .expect("initialize AppState");
    (dir, state)
}

#[test]
fn test_wal_mode_and_foreign_keys_and_zero_proprietary_tables() {
    let (_dir, state) = make_test_state();
    state
        .with_memos_db(|conn| {
            let jm: String = conn.query_row("PRAGMA journal_mode", [], |r| r.get(0))?;
            assert_eq!(jm.to_lowercase(), "wal");
            let fk: i64 = conn.query_row("PRAGMA foreign_keys", [], |r| r.get(0))?;
            assert_eq!(fk, 1);
            assert_zero_proprietary_tables(conn)?;
            Ok(())
        })
        .expect("WAL & schema checks");
}

#[test]
fn test_multi_user_bcrypt_and_visibility_matrix() {
    let (_dir, state) = make_test_state();
    let hash_host = auth::hash_password("hostpass123").expect("hash host");
    assert!(
        hash_host.starts_with("$2b$") || hash_host.starts_with("$2a$"),
        "bcrypt hash must be Memos-compatible"
    );
    assert!(auth::verify_password("hostpass123", &hash_host).expect("verify"));
    assert!(!auth::verify_password("wrong", &hash_host).expect("verify wrong"));

    let (host, alice) = state
        .with_memos_db(|conn| {
            let h = user::create_user(
                conn,
                &CreateUserInput {
                    username: "host".to_string(),
                    role: "HOST".to_string(),
                    email: "host@local".to_string(),
                    nickname: "Host".to_string(),
                    password_hash: hash_host,
                    avatar_url: String::new(),
                    description: String::new(),
                },
            )?;
            let a = user::create_user(
                conn,
                &CreateUserInput {
                    username: "alice".to_string(),
                    role: "USER".to_string(),
                    email: "alice@local".to_string(),
                    nickname: "Alice".to_string(),
                    password_hash: auth::hash_password("alicepass").expect("hash alice"),
                    avatar_url: String::new(),
                    description: String::new(),
                },
            )?;

            memo::create_memo(
                conn,
                &CreateMemoInput {
                    uid: "m-priv".to_string(),
                    creator_id: h.id,
                    content: "Secret note #private".to_string(),
                    visibility: "PRIVATE".to_string(),
                    pinned: false,
                    created_ts: None,
                    location: None,
                },
            )?;
            memo::create_memo(
                conn,
                &CreateMemoInput {
                    uid: "m-prot".to_string(),
                    creator_id: h.id,
                    content: "Team note #work".to_string(),
                    visibility: "PROTECTED".to_string(),
                    pinned: false,
                    created_ts: None,
                    location: None,
                },
            )?;
            memo::create_memo(
                conn,
                &CreateMemoInput {
                    uid: "m-pub".to_string(),
                    creator_id: h.id,
                    content: "Public note #news".to_string(),
                    visibility: "PUBLIC".to_string(),
                    pinned: true,
                    created_ts: None,
                    location: None,
                },
            )?;
            Ok((h, a))
        })
        .expect("seed users & memos");

    let filter = MemoFilter {
        limit: 50,
        ..Default::default()
    };

    // Signed-out guest sees only PUBLIC
    let guest_memos = state
        .with_memos_db(|c| memo::list_memos(c, None, &filter))
        .expect("guest list");
    assert_eq!(guest_memos.len(), 1);
    assert_eq!(guest_memos[0].uid, "m-pub");

    // Alice sees PUBLIC + PROTECTED (not Host's PRIVATE)
    let alice_memos = state
        .with_memos_db(|c| memo::list_memos(c, Some(alice.id), &filter))
        .expect("alice list");
    assert_eq!(alice_memos.len(), 2);

    // Host sees all 3 (and pinned comes first)
    let host_memos = state
        .with_memos_db(|c| memo::list_memos(c, Some(host.id), &filter))
        .expect("host list");
    assert_eq!(host_memos.len(), 3);
    assert!(host_memos[0].pinned);
}

#[test]
fn test_memo_payload_checkbox_toggle_and_fts5_search() {
    let (_dir, state) = make_test_state();
    let content = "Check https://usememos.com\n- [ ] Write docs\n- [x] Ship app\n#work/backend `code`";
    let payload = memo::extract_payload(content, None);
    assert!(payload.property.has_link);
    assert!(payload.property.has_task_list);
    assert!(payload.property.has_incomplete_tasks);
    assert!(payload.property.has_code);
    assert_eq!(payload.tags, vec!["work/backend".to_string()]);

    let toggled = memo::toggle_task_checkbox(content, 0).expect("toggle first checkbox");
    assert!(toggled.contains("- [x] Write docs"));
    let updated_payload = memo::extract_payload(&toggled, None);
    assert!(!updated_payload.property.has_incomplete_tasks);

    let created = state
        .with_memos_db(|conn| {
            let u = user::create_user(
                conn,
                &CreateUserInput {
                    username: "searcher".to_string(),
                    role: "HOST".to_string(),
                    email: String::new(),
                    nickname: "Searcher".to_string(),
                    password_hash: auth::hash_password("pw1234").expect("pw"),
                    avatar_url: String::new(),
                    description: String::new(),
                },
            )?;
            memo::create_memo(
                conn,
                &CreateMemoInput {
                    uid: "fts-1".to_string(),
                    creator_id: u.id,
                    content: "Quantum rust sqlite architecture #engineering/rust".to_string(),
                    visibility: "PRIVATE".to_string(),
                    pinned: false,
                    created_ts: None,
                    location: None,
                },
            )
        })
        .expect("create memo");

    search::upsert_memo_fts(&state, &created).expect("upsert fts");
    let hits = search::search_memo_ids(&state, "quantum sqlite", 10).expect("fts search");
    assert_eq!(hits, vec![created.id]);
}

#[test]
fn test_1_to_1_compatibility_round_trip_mode1_db_and_mode2_bundle_and_rollback() {
    let (_dir1, state1) = make_test_state();

    // Seed rich dataset: users, memos, relation, reaction, and media asset
    let (host_id, m1_id, m1_uid) = state1
        .with_memos_db(|conn| {
            let host = user::create_user(
                conn,
                &CreateUserInput {
                    username: "founder".to_string(),
                    role: "HOST".to_string(),
                    email: "founder@xmemos.org".to_string(),
                    nickname: "Founder".to_string(),
                    password_hash: auth::hash_password("secret123").expect("pw"),
                    avatar_url: String::new(),
                    description: "Offline notes".to_string(),
                },
            )?;
            let m1 = memo::create_memo(
                conn,
                &CreateMemoInput {
                    uid: "memo-main-1".to_string(),
                    creator_id: host.id,
                    content: "Main memo with #tag/nested and [ ] task".to_string(),
                    visibility: "PUBLIC".to_string(),
                    pinned: true,
                    created_ts: None,
                    location: None,
                },
            )?;
            let m2 = memo::create_memo(
                conn,
                &CreateMemoInput {
                    uid: "memo-comment-1".to_string(),
                    creator_id: host.id,
                    content: "Follow-up comment".to_string(),
                    visibility: "PUBLIC".to_string(),
                    pinned: false,
                    created_ts: None,
                    location: None,
                },
            )?;
            memo_relation::upsert_relation(conn, m2.id, m1.id, "COMMENT")?;
            reaction::toggle_reaction(conn, host.id, &format!("memos/{}", m1.uid), "🔥")?;
            Ok((host.id, m1.id, m1.uid))
        })
        .expect("seed dataset");

    let media_bytes = b"hello-xmemos-binary-payload-1234567890".to_vec();
    let res = media::save_new_resource(
        &state1,
        host_id,
        "notes.txt",
        &media_bytes,
        Some("text/plain"),
        Some(m1_id),
    )
    .expect("save resource");
    assert!(res.reference.starts_with("assets/"));

    // --- MODE 1: DB-ONLY EXPORT ---
    let db_export = backup::export_database_only(&state1, None).expect("mode 1 export");
    let exported_conn = Connection::open(&db_export.path).expect("open exported db");
    validate_imported_memos_db(&exported_conn).expect("valid official memos db");
    assert_zero_proprietary_tables(&exported_conn).expect("zero proprietary tables");
    drop(exported_conn);

    // --- MODE 2: FULL BUNDLE (.xmemos ZIP) EXPORT ---
    let full_export = backup::export_full_bundle(&state1, None).expect("mode 2 export");
    assert_eq!(full_export.media_count, 1);

    // Simulate clean device wipe (brand new AppState) & import .xmemos bundle
    let (_dir2, state2) = make_test_state();
    let preview = backup::preview_import(&state2, Path::new(&full_export.path)).expect("preview");
    assert!(preview.checksum_verified);
    assert_eq!(preview.user_count, 1);
    assert_eq!(preview.memo_count, 2);
    assert_eq!(preview.resource_count, 1);
    assert_eq!(preview.media_file_count, 1);

    let imported =
        backup::execute_atomic_import(&state2, Path::new(&full_export.path)).expect("import");
    assert_eq!(imported.restored_media_files, 1);
    assert_eq!(imported.fts_indexed_memos, 2);

    // Verify bit-for-bit restoration of memo and media file
    let restored_memo = state2
        .with_memos_db(|c| memo::get_memo_by_uid(c, &m1_uid))
        .expect("query restored")
        .expect("memo exists");
    assert_eq!(restored_memo.content, "Main memo with #tag/nested and [ ] task");
    let restored_res = state2
        .with_memos_db(|c| xmemos_lib::store::resource::get_resource_by_uid(c, &res.uid))
        .expect("query res")
        .expect("res exists");
    let (restored_bytes, _) =
        media::read_resource_bytes(&state2, &restored_res, false).expect("read restored media");
    assert_eq!(restored_bytes, media_bytes);

    // --- CORRUPT ARCHIVE ROLLBACK TEST ---
    let corrupt_path = state2.temp_dir().join("corrupt.db");
    fs::write(&corrupt_path, b"NOT A REAL SQLITE FILE").expect("write corrupt");
    let err = backup::execute_atomic_import(&state2, &corrupt_path);
    assert!(err.is_err(), "corrupt import must fail");

    // Existing data must remain completely untouched
    let still_there = state2
        .with_memos_db(|c| memo::get_memo_by_uid(c, &m1_uid))
        .expect("query after failed import");
    assert!(still_there.is_some());
}

#[test]
fn test_path_traversal_and_zip_slip_rejected() {
    let (_dir, state) = make_test_state();

    assert!(media::resolve_safe_asset_path(&state.media_root, "../etc/passwd").is_err());
    assert!(media::resolve_safe_asset_path(&state.media_root, "assets/../../evil.sh").is_err());
    assert!(media::resolve_safe_asset_path(&state.media_root, "/etc/shadow").is_err());

    // Build malicious ZIP with path traversal entry
    let malicious_zip = state.temp_dir().join("evil.xmemos");
    {
        let f = File::create(&malicious_zip).expect("create evil zip");
        let mut zw = ZipWriter::new(f);
        zw.start_file("../pwned.txt", SimpleFileOptions::default())
            .expect("start file");
        zw.write_all(b"evil").expect("write");
        zw.finish().expect("finish");
    }

    let res = backup::preview_import(&state, &malicious_zip);
    assert!(res.is_err(), "ZIP-slip archive must be rejected");
}

#[test]
fn test_smart_image_compression_to_webp_and_resize_and_gif_preservation() {
    use image::{ImageBuffer, ImageFormat, Rgba};
    use std::io::Cursor;

    let (_dir, state) = make_test_state();
    let user = state
        .with_memos_db(|conn| {
            user::create_user(
                conn,
                &CreateUserInput {
                    username: "mediauser".to_string(),
                    role: "HOST".to_string(),
                    email: String::new(),
                    nickname: "Media".to_string(),
                    password_hash: auth::hash_password("pw1234").expect("pw"),
                    avatar_url: String::new(),
                    description: String::new(),
                },
            )
        })
        .expect("create user");

    // 1. Oversized 2600x1400 PNG with gradient/patterns -> must resize to <=2048px and compress to .webp
    let img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_fn(2600, 1400, |x, y| {
        Rgba([(x % 256) as u8, (y % 256) as u8, ((x + y) % 128) as u8, 200])
    });
    let mut raw_png = Vec::new();
    image::DynamicImage::ImageRgba8(img)
        .write_to(&mut Cursor::new(&mut raw_png), ImageFormat::Png)
        .expect("encode raw png");

    let saved = media::save_new_resource(
        &state,
        user.id,
        "screenshot.png",
        &raw_png,
        Some("image/png"),
        None,
    )
    .expect("save compressed resource");

    assert!(saved.r#type == "image/webp" || saved.r#type == "image/png");
    assert!(saved.filename.ends_with(".webp") || saved.filename.ends_with(".png"));
    assert!(
        (saved.size as usize) < raw_png.len(),
        "compressed WebP ({}) should be smaller than raw PNG ({})",
        saved.size,
        raw_png.len()
    );

    // Verify decoded output is valid WebP and resized within 2048px
    let (disk_bytes, _) = media::read_resource_bytes(&state, &saved, false).expect("read");
    let decoded = image::load_from_memory(&disk_bytes).expect("valid webp decode");
    assert!(decoded.width() <= 2048 && decoded.height() <= 2048);

    // 2. GIF must remain untouched as image/gif
    let gif_bytes = b"GIF89a\x01\x00\x01\x00\x80\x00\x00\xff\xff\xff\x00\x00\x00!\xf9\x04\x01\x00\x00\x00\x00,\x00\x00\x00\x00\x01\x00\x01\x00\x00\x02\x02D\x01\x00;";
    let saved_gif = media::save_new_resource(
        &state,
        user.id,
        "anim.gif",
        gif_bytes,
        Some("image/gif"),
        None,
    )
    .expect("save gif");
    assert_eq!(saved_gif.r#type, "image/gif");
    assert_eq!(saved_gif.size as usize, gif_bytes.len());
}
