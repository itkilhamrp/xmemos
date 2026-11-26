# AGENTS.md — Universal AI Coding Agent Guardrails for Xmemos

All AI coding agents (Gemini, Claude, GPT, DeepSeek, Copilot, Cursor, Windsurf, Antigravity IDE) MUST read and obey this file and `docs/ai/*` before modifying any file in this repository.

## Immutable Invariants (NEVER Violate)

1. **1:1 Official Memos SQLite Integrity:** NEVER add custom columns or proprietary tables to `memos_prod.db`. Any Xmemos local-only data (FTS5 index, sessions, PIN locks, UI prefs, drafts, backup history) MUST live exclusively in `xmemos_local.db`. See [docs/ai/SCHEMA_CONTRACT.md](docs/ai/SCHEMA_CONTRACT.md).
2. **Konsta UI iOS Theme Only:** NEVER switch away from `theme="ios"` or introduce Material Design components. Use `konsta/svelte` + Liquid Glass utilities in `src/app.css`.
3. **Strict TypeScript + Svelte 5 Runes + SvelteKit 3 API Only:** ZERO `.js` files inside `src/`. NEVER use legacy Svelte 4 syntax (`export let`, `$:`, `createEventDispatcher`, `writable()`) or removed/deprecated SvelteKit 2 `goto()` options (`keepFocus`, `noScroll`, `replaceState`, `invalidateAll` — in SvelteKit 3 use `reset: false`, `replace: true`, and `refreshAll: true` exclusively). Use `$state`, `$derived`, `$effect`, `$props` exclusively. See [docs/ai/CONVENTIONS.md](docs/ai/CONVENTIONS.md).
4. **Single Source of Truth for App Version:** `package.json` is the sole version SSOT (`src-tauri/tauri.conf.json` references `"../package.json"`). Never hardcode version numbers elsewhere; run `pnpm version:sync`.
5. **Manual-Only Modular CI/CD:** NEVER add `push`, `pull_request`, or `schedule` triggers to `.github/workflows/release.yml` (`workflow_dispatch` only) or its reusable OS workflows (`android-release.yml`, `linux-release.yml`, `windows-release.yml`, `macos-release.yml`, `workflow_call` only). Always pass secrets explicitly per platform (never `secrets: inherit`), and ALWAYS source GitHub Release notes from the matching `## [${APP_VERSION}]` section of `CHANGELOG.md` (never revert to raw `git log` commit subjects). In `.github/workflows/macos-release.yml`, ALWAYS build `--bundles app` and package DMGs via mount-free `ditto` + `hdiutil create -srcfolder ... -format UDZO` + `hdiutil verify` (NEVER revert to `--bundles dmg` / vendored `bundle_dmg.sh`, which fails with `hdiutil: couldn't eject - Resource busy` on macOS 15 runners).
6. **Zero Placeholders & No Unwrap in Commands:** Never leave `TODO`/`FIXME` stubs or `.unwrap()`/`.expect()` inside `src-tauri/src/commands/`. Always return `AppResult<T>`.
7. **100% GMS-Free & Full FOSS:** NEVER add `com.google.android.gms`, `com.google.firebase`, Play Services, or proprietary telemetry SDKs. Verify with `pnpm foss:check`.
8. **AGPL-3.0-or-later Headers:** Every new source file must include the SPDX AGPLv3 header (`pnpm license:apply`).
9. **Adaptive Platform Separation:** Always gate Android-only UI/APIs (`.nomedia`, `BiometricPrompt`) behind `supportsNomedia()` / `supportsAndroidBiometric()` and Desktop-only UI/APIs (window controls, native Save dialogs) behind `isDesktopTauri()` / `#[cfg(desktop)]`.
10. **Automatic Persistent Android Release Signing & Monotonic `versionCode`:** NEVER replace `.github/workflows/android-release.yml`'s persistent signing flow (priority: optional `ANDROID_KEYSTORE_BASE64` secret → automatic permanent RSA-4096 PKCS12 keystore stored in the private GitHub Draft Release `xmemos-android-keystore-internal`) with an ephemeral per-run keystore, and NEVER remove the explicit `defaultConfig.versionCode` pinning or `aapt2 dump badging` check. Changing signing keys breaks Android in-place updates (`INSTALL_FAILED_UPDATE_INCOMPATIBLE`).
11. **Konsta UI v5 iOS 26 Liquid Glass Tabbar, Zero-Jitter Scrub & PowerPoint Slide Page Transitions:** NEVER style `.xm-ios-tabbar-pill` with a solid blue background, and NEVER bind `transform: translate3d(...)` inside the Svelte `style={...}` attribute of `pillEl` (which overwrites imperative `requestAnimationFrame` transforms mid-drag and causes jitter!). Keep `touch-action: none` on `.xm-ios-tabbar-track`/`.xm-ios-tabbar-item`, `contain: strict` on `.xm-ios-tabbar-pill`, `165ms` dwell debounce, `view-transition-name: xm-bottom-tabbar` on `.xm-ios-tabbar-shell`, and the `onNavigate` + `document.startViewTransition` directional slide transition (`src/lib/utils/route-transition.ts` + `src/routes/+layout.svelte`, cancelling any in-flight transition via `skipTransition()` for zero queue lag).
12. **Android Public Downloads via `MediaStore.Downloads`:** NEVER revert Android downloads (`save_bytes_for_platform`) to private sandbox `download_dir()` writes. Always route through `XmemosStoragePlugin.saveToDownloads` (`MediaStore.Downloads.EXTERNAL_CONTENT_URI`, `Download/Xmemos`, `IS_PENDING = 1 -> 0`) so images, static files, and Share Cards appear immediately in Android File Manager.
13. **Comment Visibility Inheritance & Top-Anchored Mobile Editor:** NEVER add `PRIVATE`/`PROTECTED`/`PUBLIC` selectors when composing or editing memo comments (comments strictly inherit the top-level root parent memo's visibility in both UI and `commands/memo.rs` with cascade on parent update). Keep `MemoEditor.svelte` rendered strictly inside `{#if opened}` as a dedicated `role="dialog"` overlay (NEVER wrap `MemoEditor` in Konsta `<Sheet>` with a CSS `top: 0 !important` override, which inverts Konsta Sheet's `top: 100%` / `translateY(-100%)` visibility and leaks closed editors across routes!) with `pt-[max(0.875rem,env(safe-area-inset-top))]` and mobile `top: var(--xm-vvt); height: var(--xm-vvh)` lock.
14. **Desktop Single-Window Sticky Launch & 100% Offline Vector Map:** Launching a Desktop sticky shortcut (`--open-memo=<uid>`) MUST keep `main` hidden + `skip_taskbar(true)` and open ONLY `sticky-<uid>` (via `sticky_launch_only`). Keep `createOfflineVectorTileLayer` (`zIndex: 1`) mounted beneath `createOfflineOsmTileLayer` (`zIndex: 2`, transparent fallback) so `/map` and mini-maps render complete coastlines/cities at 60fps with zero internet.
15. **Temp-Staged Backup Export, Hybrid Rust/WebView Media Compression & HOST/ADMIN Upload Size Limit:**
    - `backup_export_db` and `backup_export_full` MUST stage exports strictly inside `temp/backup_exports/` (NEVER `backups/`) and ALWAYS clean up via `backup_finalize_export` in `try/finally` after the OS/SAF File Manager dialog finishes or is cancelled; record `backup_history` ONLY when actually saved.
    - Keep `media-compress.ts` (24 KB threshold, `1920px`, native WebView `libwebp` VP8 Q0.80) paired with Rust `optimize_static_upload` (`1920px` Lanczos3, lossless WebP fast-path & 8% preference, JPEG Q78/Q74, Best-Adaptive PNG) without adding unlocked crates to `Cargo.toml`. Editing a memo MUST only bind existing `resourceIds` (never re-upload existing attachments), and `AttachmentPreviewStrip.svelte` must render live per-file upload progress bars before Save is enabled.
    - Keep `max-upload-size-mb` in `memos_prod.db`'s existing `system_setting` table (default `32` MB, `0` = unlimited, `0..=2048`), restricted strictly to `HOST`/`ADMIN` via `require_admin_or_host` in `system_upsert_setting` and enforced in BOTH `MemoEditor.svelte` and `commands/resource.rs` (`resource_upload_bytes` & `resource_import_file`).
16. **Smart Crash-Proof Video Compression (Android Media3 Hardware + Desktop Optional System FFmpeg):**
    - Android video compression MUST stay on 100% FOSS Jetpack Media3 (`androidx.media3:media3-transformer:1.11.1`, `media3-effect`, `media3-common` — Apache 2.0, zero GMS) inside `XmemosVideoCompressor.kt` with `MediaMetadataRetriever` pre-probe, `Presentation.createForShortSide(720)`, adaptive bitrate (`0.55x`, `0.9–2.2 Mbps`), `DefaultEncoderFactory.Builder().setEnableFallback(true)`, 90-second watchdog timeout, and `>= 8%` anti-bloat guard falling back cleanly to the original file on any error.
    - Desktop video compression (`src-tauri/src/service/video.rs`) MUST remain zero-bundled-binary (opportunistic system `ffmpeg` with `libx264` CRF 26 + `-movflags +faststart` + `CREATE_NO_WINDOW` on Windows + 90s watchdog + `>= 8%` anti-bloat guard, silent passthrough if `ffmpeg` is absent). NEVER bundle FFmpeg binaries into the repo or APK, and ALWAYS run `resource_upload_bytes` / `resource_import_file` compression inside `tauri::async_runtime::spawn_blocking`.
17. **Instant Cross-OS Native Startup & Bounded Zero-Bloat Cache:**
    - **Android Native Kotlin (`XmemosStartupOptimizer.kt`)**: Keep `XmemosStartupOptimizer.applyOnLoad(activity, webView)` called inside `XmemosStoragePlugin.load()` to align native Activity/WebView background before first paint, enable `WebSettings.LOAD_DEFAULT` + `offscreenPreRaster` + hardware layer, and prune stale `video_opt`/`export_tmp` (>6h) + `thumbs/` (>48 MiB LRU) on a low-priority background executor.
    - **Desktop Native per OS/DE (`src-tauri/src/main.rs`, `state.rs`, `lib.rs`)**: NEVER revert Windows WebView2 detection to spawning `reg.exe` subprocesses (keep direct Win32 `advapi32.dll` `RegOpenKeyExW`/`RegQueryValueExW` + `--disable-background-networking --no-first-run`), keep Linux zero-subprocess NVIDIA+Wayland `WEBKIT_DISABLE_DMABUF_RENDERER=1` detection (respecting user overrides), and keep the 48 MiB LRU `thumbs/` cap + `PRAGMA optimize` in `run_background_startup_maintenance()`.
    - **Frame-0 Frontend Hydration (`+layout.svelte`, `memos.svelte.ts`)**: Keep synchronous Frame-0 `memosState.hydrateSnapshot()` + single-rAF instant Desktop `revealMainWindow()` (NEVER block window reveal on `await authState.refresh()`), and keep the strict `128 KB` / `7-day TTL` / `max 3 users` caps on `localStorage` memo snapshots.

## Command Reference

```bash
pnpm dev                 # Vite SPA dev server (port 1420)
pnpm check               # svelte-kit sync + svelte-check (strict TS)
pnpm test                # Vitest unit tests
pnpm build               # Static SPA production build -> build/
pnpm version:check       # Validate package.json <-> tauri.conf.json / Cargo.toml
pnpm license:apply       # Insert AGPLv3 headers via google/addlicense
pnpm license:check       # Verify all source files carry AGPLv3 headers
pnpm tauri android dev   # Run on connected Android device/emulator
cd src-tauri && cargo clippy --locked --all-targets -- -D warnings && cargo test --locked
```

## Mandatory Pre-Commit Verification Checklist

- [ ] `pnpm version:check` passes
- [ ] `pnpm license:check` passes
- [ ] `pnpm check` reports 0 errors and 0 warnings
- [ ] `pnpm test` passes
- [ ] `pnpm build` succeeds cleanly
- [ ] `cd src-tauri && cargo clippy --locked --all-targets -- -D warnings && cargo test --locked` passes

## Further AI Reference Guides

- [docs/ai/CONVENTIONS.md](docs/ai/CONVENTIONS.md)
- [docs/ai/SCHEMA_CONTRACT.md](docs/ai/SCHEMA_CONTRACT.md)
- [docs/ai/TASK_PLAYBOOK.md](docs/ai/TASK_PLAYBOOK.md)
