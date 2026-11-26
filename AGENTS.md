# AGENTS.md — Universal AI Coding Agent Guardrails for Xmemos

All AI coding agents (Gemini, Claude, GPT, DeepSeek, Copilot, Cursor, Windsurf, Antigravity IDE) MUST read and obey this file and `docs/ai/*` before modifying any file in this repository.

## Immutable Invariants (NEVER Violate)

1. **1:1 Official Memos SQLite Integrity:** NEVER add custom columns or proprietary tables to `memos_prod.db`. Any Xmemos local-only data (FTS5 index, sessions, PIN locks, UI prefs, drafts, backup history) MUST live exclusively in `xmemos_local.db`. See [docs/ai/SCHEMA_CONTRACT.md](docs/ai/SCHEMA_CONTRACT.md).
2. **Konsta UI iOS Theme Only:** NEVER switch away from `theme="ios"` or introduce Material Design components. Use `konsta/svelte` + Liquid Glass utilities in `src/app.css`.
3. **Strict TypeScript + Svelte 5 Runes Only:** ZERO `.js` files inside `src/`. NEVER use legacy Svelte 4 syntax (`export let`, `$:`, `createEventDispatcher`, `writable()`). Use `$state`, `$derived`, `$effect`, `$props` exclusively. See [docs/ai/CONVENTIONS.md](docs/ai/CONVENTIONS.md).
4. **Single Source of Truth for App Version:** `package.json` is the sole version SSOT (`src-tauri/tauri.conf.json` references `"../package.json"`). Never hardcode version numbers elsewhere; run `pnpm version:sync`.
5. **Manual-Only Modular CI/CD:** NEVER add `push`, `pull_request`, or `schedule` triggers to `.github/workflows/release.yml` (`workflow_dispatch` only) or its reusable OS workflows (`android-release.yml`, `linux-release.yml`, `windows-release.yml`, `macos-release.yml`, `workflow_call` only). Always pass secrets explicitly per platform (never `secrets: inherit`), and ALWAYS source GitHub Release notes from the matching `## [${APP_VERSION}]` section of `CHANGELOG.md` (never revert to raw `git log` commit subjects).
6. **Zero Placeholders & No Unwrap in Commands:** Never leave `TODO`/`FIXME` stubs or `.unwrap()`/`.expect()` inside `src-tauri/src/commands/`. Always return `AppResult<T>`.
7. **100% GMS-Free & Full FOSS:** NEVER add `com.google.android.gms`, `com.google.firebase`, Play Services, or proprietary telemetry SDKs. Verify with `pnpm foss:check`.
8. **AGPL-3.0-or-later Headers:** Every new source file must include the SPDX AGPLv3 header (`pnpm license:apply`).
9. **Adaptive Platform Separation:** Always gate Android-only UI/APIs (`.nomedia`, `BiometricPrompt`) behind `supportsNomedia()` / `supportsAndroidBiometric()` and Desktop-only UI/APIs (window controls, native Save dialogs) behind `isDesktopTauri()` / `#[cfg(desktop)]`.
10. **Automatic Persistent Android Release Signing & Monotonic `versionCode`:** NEVER replace `.github/workflows/android-release.yml`'s persistent signing flow (priority: optional `ANDROID_KEYSTORE_BASE64` secret → automatic permanent RSA-4096 PKCS12 keystore stored in the private GitHub Draft Release `xmemos-android-keystore-internal`) with an ephemeral per-run keystore, and NEVER remove the explicit `defaultConfig.versionCode` pinning or `aapt2 dump badging` check. Changing signing keys breaks Android in-place updates (`INSTALL_FAILED_UPDATE_INCOMPATIBLE`).
11. **Konsta UI v5 iOS 26 Liquid Glass Tabbar & Realtime Scrub:** NEVER style `.xm-ios-tabbar-pill` with a solid blue background; keep the strict Konsta UI iOS 26 neutral frosted glass lens, critically-damped transition (`cubic-bezier(0.25, 1, 0.5, 1)`), and realtime route scrubbing with `pendingNavIndex` lock.
12. **Android Public Downloads via `MediaStore.Downloads`:** NEVER revert Android downloads (`save_bytes_for_platform`) to private sandbox `download_dir()` writes. Always route through `XmemosStoragePlugin.saveToDownloads` (`MediaStore.Downloads.EXTERNAL_CONTENT_URI`, `Download/Xmemos`, `IS_PENDING = 1 -> 0`) so images, static files, and Share Cards appear immediately in Android File Manager.
13. **Comment Visibility Inheritance & Top-Anchored Mobile Editor:** NEVER add `PRIVATE`/`PROTECTED`/`PUBLIC` selectors when composing or editing memo comments (comments strictly inherit the top-level root parent memo's visibility in both UI and `commands/memo.rs` with cascade on parent update). Keep mobile `.xm-kb-editor-sheet` top-anchored to `var(--xm-vvt)`/`var(--xm-vvh)` so soft keyboards never cause vertical UI jumping.
14. **Desktop Single-Window Sticky Launch & 100% Offline Vector Map:** Launching a Desktop sticky shortcut (`--open-memo=<uid>`) MUST keep `main` hidden + `skip_taskbar(true)` and open ONLY `sticky-<uid>` (via `sticky_launch_only`). Keep `createOfflineVectorTileLayer` (`zIndex: 1`) mounted beneath `createOfflineOsmTileLayer` (`zIndex: 2`, transparent fallback) so `/map` and mini-maps render complete coastlines/cities at 60fps with zero internet.

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
