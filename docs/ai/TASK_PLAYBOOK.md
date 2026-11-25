# Task Playbook (SOP for Safe Feature Extensions)

## 1. Adding a New Tauri IPC Command
1. Add repository query in `src-tauri/src/store/<domain>.rs` (parameterized SQL only).
2. Add business logic / RBAC checks in `src-tauri/src/service/<domain>.rs`.
3. Add `#[tauri::command]` handler in `src-tauri/src/commands/<domain>.rs` returning `AppResult<T>`.
4. Register the command in `tauri::generate_handler![...]` inside `src-tauri/src/lib.rs`.
5. Mirror the DTO in `src/lib/types/<domain>.ts` and expose a typed method in `src/lib/api/<domain>.ts`.
6. Consume from a Svelte 5 Rune state class in `src/lib/state/*.svelte.ts`.
7. Add Rust integration test in `src-tauri/tests/compatibility_tests.rs`.

## 2. Adding a New UI Route or Sheet
1. Create `src/routes/<path>/+page.svelte` wrapped in `<AppShell>`.
2. Use only `konsta/svelte` iOS components and Liquid Glass utilities from `src/app.css`.
3. Gate any platform-specific controls with `isDesktopTauri()`, `supportsNomedia()`, or `supportsAndroidBiometric()`.
4. Verify with `pnpm check && pnpm build`.

## 3. Cutting a Deterministic Multi-Platform Release
1. Update `"version"` in `package.json` and document release notes once in `CHANGELOG.md` (single source of truth).
2. Run `pnpm version:sync && pnpm version:check && pnpm license:apply && pnpm license:check && pnpm foss:check` — this automatically syncs `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, Android `tauri.properties`, `packaging/flatpak/org.xmemos.app.metainfo.xml` (version + date from `CHANGELOG.md`, linking to `CHANGELOG.md` without duplicating changelog prose), and the About page fallback version.
3. Commit and push to GitHub.
4. Trigger manually via `workflow_dispatch`:
   ```bash
   gh workflow run release.yml \
     -f release_channel=stable \
     -f build_android=true \
     -f target_abis=all \
     -f build_linux_appimage=true \
     -f build_linux_flatpak=true \
     -f build_windows=true \
     -f build_macos=true \
     -f publish_release=true
   ```
