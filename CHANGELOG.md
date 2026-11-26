# Changelog

All notable changes to **Xmemos** are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/spec/v2.0.0.html),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.9.1] - 2026-11-26

### Fixed
- **macOS DMG CI Packaging `hdiutil: couldn't eject "disk2" - Resource busy` (`.github/workflows/macos-release.yml`, `AGENTS.md`)**:
  - Switched `macos-release.yml` from Tauri's vendored `bundle_dmg.sh` (`--bundles dmg`, which mounts an interstitial RW image that Spotlight/Finder locks and fails with `hdiutil` exit code 2 on macOS 15 runners — `tauri-apps/tauri#14686`) to building signed `.app` bundles (`--bundles app`) and packaging directly into verified read-only `UDZO` `.dmg` installers via `ditto` + `/Applications` symlink + mount-free `hdiutil create -srcfolder ... -format UDZO` + `hdiutil verify`.
  - Locked this macOS packaging rule into `AGENTS.md` Invariant #5 so future AI updates never regress to `bundle_dmg.sh`.

## [2.9.0] - 2026-11-26

### Fixed
- **SvelteKit 3 `GotoOptions` Strict Typecheck Compliance (`src/lib/components/layout/BottomTabbar.svelte`, `src/routes/+layout.svelte`, `src/routes/+page.svelte`, `AGENTS.md`)**:
  - Replaced the removed SvelteKit 2 `goto(target.path, { keepFocus: true })` option in `BottomTabbar.svelte` with SvelteKit 3's canonical `goto(target.path, { reset: false })` (which combines `keepFocus: true` and `noScroll: true` per official SvelteKit v3 migration docs), resolving the `svelte-check` CI failure.
  - Migrated deprecated `{ replaceState: true }` calls in `src/routes/+layout.svelte` and `src/routes/+page.svelte` to SvelteKit 3's `{ replace: true }`, and documented SvelteKit 3 `GotoOptions` guardrails in `AGENTS.md` Invariant #3 to prevent future regressions.

## [2.8.1] - 2026-11-26

### Changed
- **Comprehensive GitHub Technology & Platform Badges (`README.md`)**:
  - Added complete Shields.io badges reflecting Xmemos's verified stack and distribution targets: License (`AGPL v3`), `GMS-Free / 100% FOSS`, `Offline-First`, `Reproducible Builds / SLSA Provenance`, Platforms (`Android API 24–36`, `GNU/Linux AppImage | Flatpak`, `Windows 10/11 x64`, `macOS 12+ arm64 | x64`), and Core Stack (`Tauri v2`, `Rust 1.100`, `Kotlin`, `SQLite WAL + FTS5`, `Svelte 5 Runes`, `SvelteKit 3`, `TypeScript 6`, `Tailwind CSS 4`, `Konsta UI v5 iOS Liquid Glass`, `Vite 8`, `Leaflet Offline Vector + OSM`, and `pnpm 12`).

## [2.8.0] - 2026-11-26

### Added & Improved
- **100% True Offline World Vector Map Engine (`src/lib/utils/offline-world-data.ts`, `src/lib/utils/offline-vector-layer.ts`, `src/lib/utils/osm-tiles.ts`, `src/lib/components/map/{OsmMapCanvas,LocationPreviewPopover}.svelte`)**:
  - Added a zero-dependency, 60fps HTML5 Canvas Leaflet `GridLayer` (`createOfflineVectorTileLayer`, `zIndex: 1`) rendering simplified Natural Earth-derived continents, major islands (including Sumatra, Java, Kalimantan, Sulawesi, Papua, Bali/Nusa Tenggara, Japan, UK, Australia, New Zealand), lat/lng graticules, and 50+ global & Indonesian cities completely offline with zero internet required and instant Light/Dark theme redraws.
  - Switched uncached/offline OSM tile fallbacks (`zIndex: 2`) to transparent 1×1 pixels so cached or online OpenStreetMap street tiles seamlessly overlay the crisp offline vector world map without ever showing blank/black squares.
- **Konsta UI v5 iOS 26 Liquid Glass Mobile Bottom Tabbar + Realtime Scrubbing (`src/lib/components/layout/BottomTabbar.svelte`, `src/app.css`)**:
  - Restored strict Konsta UI v5 iOS 26 Liquid Glass styling on `.xm-ios-tabbar-pill` (neutral frosted glass lens with hairline specular highlight, removing the blue active background fill).
  - Upgraded horizontal finger scrubbing to `requestAnimationFrame`-throttled **realtime page switching** with boundary hysteresis (`0.36`), `pendingNavIndex` lock (eliminating release snap-back/bounce), and critically-damped `cubic-bezier(0.25, 1, 0.5, 1)` transitions.
- **Rich Android Home-Screen Sticky Memo Widget (`plugins/tauri-plugin-xmemos-storage/android/src/main/res/layout/widget_memo_sticky.xml`, `XmemosMemoWidgetProvider.kt`, `XmemosStoragePlugin.kt`, `src/lib/api/shortcut.ts`, `MemoActionsSheet.svelte`)**:
  - Upgraded the `@RemoteView`-safe Android home-screen sticky widget to render rich structured memo content: pin badge (`📌`) + relative timestamp, formatted checklists (`☑`/`☐`) and bullet points (`•`), plus a compact metadata footer (`#tags · 📍 location · 📎 attachments`) while remaining zero-service and lightweight.

### Fixed
- **Android File Manager Downloads & Share Card Export (`plugins/tauri-plugin-xmemos-storage/android/.../XmemosStoragePlugin.kt`, `plugins/tauri-plugin-xmemos-storage/src/{models,mobile}.rs`, `src-tauri/src/commands/resource.rs`)**:
  - Replaced private sandbox `download_dir()` writes on Android with native Kotlin `saveToDownloads` backed by Android's official `MediaStore.Downloads.EXTERNAL_CONTENT_URI` (`RELATIVE_PATH = Download/Xmemos`, `IS_PENDING = 1 -> 0` on API 29+, and `Environment.DIRECTORY_DOWNLOADS/Xmemos` + `MediaScannerConnection` on API 24–28), so downloaded images, static files, and exported Share Card PNGs appear immediately in Android File Manager and Gallery.
- **Memo Comment Visibility Inheritance (`src-tauri/src/commands/memo.rs`, `src/lib/components/memo/MemoEditor.svelte`, `src/routes/memo/[uid]/+page.svelte`)**:
  - Removed the illogical `PRIVATE` / `PROTECTED` / `PUBLIC` visibility selector when composing or editing comments. Comments now strictly inherit the top-level root parent memo's visibility in Rust (`resolve_root_memo_visibility`) and automatically stay synced via `cascade_comment_visibility` whenever the parent memo's visibility changes.
- **Android Memo Composer Keyboard Stability (`src/app.css`, `src/lib/components/memo/MemoEditor.svelte`)**:
  - Locked `.xm-kb-editor-sheet` on mobile (`< 768px`) to the top of the visual viewport (`top: var(--xm-vvt); height: var(--xm-vvh);`) with a pinned header/toolbar, flexible scrollable textarea, and pinned footer so opening, closing, or resizing the Android soft keyboard never causes UI elements to jump up and down.
- **GitHub Release Notes Sourced Directly from `CHANGELOG.md` (`.github/workflows/release.yml`, `AGENTS.md`)**:
  - Replaced raw `git log` commit-subject release notes in `publish-release` with exact `## [${APP_VERSION}]` section extraction from `CHANGELOG.md` (falling back to `git log` only if no matching section exists) so published GitHub Releases display the full structured changelog.
- **Desktop "Add to Desktop" Shortcut Single-Window MS Sticky Notes Launch (`src-tauri/src/{state,lib}.rs`, `src-tauri/src/commands/system.rs`, `src/lib/api/system.ts`, `src/routes/+layout.svelte`)**:
  - Launching a Desktop Sticky Note shortcut (`--open-memo=<uid>`) now spawns **only** the floating `sticky-<uid>` window directly in Rust `setup()`, keeps `main` hidden + `set_skip_taskbar(true)` (`sticky_launch_only` guard), never double-opens the full Xmemos window, and cleanly exits when the last sticky window closes (while `↗ Open in Xmemos` still reveals the main window on demand).

## [2.7.35] - 2026-11-26

### Fixed
- **Desktop Touchpad Two-Finger Pinch Zooming the Entire App UI on Map (`src/lib/utils/osm-tiles.ts`, `src/lib/components/map/OsmMapCanvas.svelte`, `src/lib/components/map/LocationPreviewPopover.svelte`, `src/routes/+layout.svelte`, `src/app.html`, `src-tauri/tauri.conf.json`)**:
  - Added `attachMapPinchZoomGuard()` in `src/lib/utils/osm-tiles.ts` (wired into both `OsmMapCanvas` and `LocationPreviewPopover` with `zoomSnap: 0.25`) to intercept non-passive `wheel` (`ctrlKey: true`) and WebKit `gesturestart`/`gesturechange`/`gestureend` touchpad pinch events, routing pinch gestures exclusively to Leaflet cursor-anchored zoom without zooming the app WebView.
  - Added an app-wide non-passive `wheel` (`ctrlKey`) + `gesture*` pinch-zoom guard in `src/routes/+layout.svelte`, locked viewport scaling in `src/app.html`, and explicitly set `"zoomHotkeysEnabled": false` in `src-tauri/tauri.conf.json` so Desktop WebViews (GNU/Linux WebKitGTK, Windows WebView2, macOS WKWebView) never accidentally page-zoom the UI.

## [2.7.34] - 2026-11-26

### Changed & Improved
- **Zero-Config Automatic Persistent Android Release Signing in GitHub Actions (`.github/workflows/android-release.yml`, `.github/workflows/release.yml`, `AGENTS.md`, `docs/REPRODUCIBLE_BUILDS.md`)**:
  - Upgraded `android-release.yml` to automatically bootstrap a permanent 10,000-day RSA-4096 PKCS12 signing keystore on first run and persist it inside a private, maintainer-only GitHub Draft Release (`xmemos-android-keystore-internal`) whenever `ANDROID_KEYSTORE_BASE64` is not manually configured—requiring zero local terminal commands and zero manual GitHub Secrets while guaranteeing 100% identical signing certificates across every future Android update.
  - Preserved full priority override if custom `ANDROID_KEYSTORE_BASE64` secrets are ever added later, masked auto-generated credentials via `::add-mask::`, and added Immutable Invariant #10 to `AGENTS.md` so future AI updates never regress persistent Android signing.

## [2.7.33] - 2026-11-26

### Fixed
- **Android In-Place APK Update Conflict (`INSTALL_FAILED_UPDATE_INCOMPATIBLE` / "Package conflicts with an existing package") (`.github/workflows/android-release.yml`, `.github/workflows/release.yml`, `scripts/generate-android-keystore.sh`, `.gitignore`, `docs/REPRODUCIBLE_BUILDS.md`)**:
  - Eliminated the silent random ephemeral keystore fallback on published Android releases (`require_persistent_signing: true` when `publish_release` is enabled), which previously signed each GitHub Release with a different throwaway RSA key and prevented in-place Android updates without uninstalling.
  - Added upfront `keytool` keystore/alias/password verification with whitespace-safe Base64 decoding (`tr -d '\r\n '`), explicitly pinned monotonic `defaultConfig.versionCode` (`major*1_000_000 + minor*1_000 + patch`) and `versionName` inside `build.gradle.kts`, and added an `aapt2 dump badging` assertion verifying `org.xmemos.app`, `versionCode`, and `versionName` on every signed APK.
  - Added `scripts/generate-android-keystore.sh` to generate a permanent 10,000-day RSA-4096 PKCS12 release keystore and GitHub Secrets configuration in one command.

## [2.7.32] - 2026-11-26

### Fixed
- **Broken Static Media & Image Previews on Android WebView & Windows WebView2 (`src/lib/api/invoke.ts`, `src-tauri/src/lib.rs`, `src-tauri/src/service/media.rs`, `src-tauri/tauri.conf.json`, `src/lib/components/media/AttachmentGrid.svelte`, `src/routes/resources/+page.svelte`)**:
  - Fixed `getMediaUrl()` to use Tauri v2's native platform-aware `convertFileSrc(uid, 'xmemos-media')` so **Android WebView** and **Windows WebView2** resolve custom protocol URLs to `http(s)://xmemos-media.localhost/<uid>` instead of hardcoded `xmemos-media://localhost/<uid>` (which failed with `ERR_UNKNOWN_URL_SCHEME`), while preserving `xmemos-media://localhost/<uid>` on GNU/Linux and macOS.
  - Upgraded `handle_media_protocol()` in `src-tauri/src/lib.rs` with full HTTP `Range` (`206 Partial Content`, `Accept-Ranges: bytes`, `Content-Range`, `Content-Length`, `HEAD`/`OPTIONS`) support for reliable `<video>` and `<audio>` playback/seeking, `Access-Control-Allow-Origin: *` for cross-origin Canvas share-card rendering, and defensive `localhost/` path stripping.
  - Added `xmemos-media: http://xmemos-media.localhost https://xmemos-media.localhost` to `connect-src` in `src-tauri/tauri.conf.json`, guarded against 0-byte thumbnail reads in `src-tauri/src/service/media.rs`, and added one-shot full-image `onerror` fallbacks in `AttachmentGrid.svelte` and `src/routes/resources/+page.svelte`.

## [2.7.31] - 2026-11-26

### Changed & Fixed
- **MS Sticky Notes-Style Desktop Memo Shortcuts (`plugins/tauri-plugin-xmemos-storage/src/{sticky_icon.rs,desktop_shortcut.rs,models.rs,lib.rs}`, `src-tauri/src/lib.rs`, `src/routes/+layout.svelte`, `src/lib/api/shortcut.ts`, `src/lib/components/memo/MemoActionsSheet.svelte`, `src/routes/sticky/[uid]/+page.svelte`)**:
  - Replaced the generic Xmemos app icon on Desktop memo shortcuts with a zero-extra-crate native **MS Sticky Notes card icon** matching the memo's sticky color (defaulting to warm MS Sticky Notes yellow `#fff5a8`): scalable SVG with real memo title & wrapped preview text on **GNU/Linux** (`.desktop`), 128×128 RGBA PNG-compressed `.ico` on **Windows 10/11** (`.lnk`), and 128×128 RGBA `ic07` `.icns` on **macOS** (`.app`).
  - Launching a Desktop Sticky Note shortcut (`--open-memo=<uid>`) now opens **directly into the floating MS Sticky Note window** while keeping the main Xmemos window hidden (unless PIN lock is enabled), and exits cleanly with zero background ghost process when the last floating sticky note closes.
  - Added MS Sticky Notes quick-edit double-click on memo content and `Ctrl/Cmd+S` save shortcut inside floating sticky windows.
- **Cross-OS Desktop Map Hardening — GNU/Linux, Windows & macOS (`packaging/flatpak/org.xmemos.app.yaml`, `src/app.css`, `src/lib/utils/osm-tiles.ts`, `src/routes/map/+page.svelte`)**:
  - Added `--share=network` to the GNU/Linux Flatpak manifest so OpenStreetMap tiles and Nominatim/Photon reverse geocoding work inside Flatpak while `tauri.conf.json` CSP strictly restricts network egress to FOSS OpenStreetMap hosts only.
  - Imported `leaflet/dist/leaflet.css` statically in `src/app.css` to prevent first-frame tile geometry races on WebKitGTK (GNU/Linux), WKWebView (macOS), WebView2 (Windows), and Android WebView, set `referrerPolicy = 'no-referrer-when-downgrade'` on direct tile loads, and added an automatic `https://www.openstreetmap.org/` browser fallback when Desktop OSes lack a `geo:` URI handler.

## [2.7.30] - 2026-11-26

### Fixed
- **Android Map Page Blank/Black Canvas & Disappearing Map Container (`src/routes/map/+page.svelte`, `src/lib/utils/osm-tiles.ts`, `src/lib/components/map/OsmMapCanvas.svelte`, `src/lib/components/map/LocationPreviewPopover.svelte`, `src-tauri/tauri.conf.json`, `src/app.css`)**:
  - Fixed mobile flexbox height collapse on `/map` where `flex-1` (`flex-basis: 0%`) inside a mobile `flex-col` container overrode the explicit height (`h-[min(58dvh,460px)]`), causing `OsmMapCanvas` (`h-full`) to collapse to `0px` and disappear on Android WebView; scoped flex growth to `md:flex-1 md:shrink` with explicit mobile `shrink-0` height and positioned the Leaflet mount container with `absolute inset-0 h-full w-full`.
  - Updated `src-tauri/tauri.conf.json` CSP (`img-src` and `connect-src`) to allow FOSS OpenStreetMap tile (`https://tile.openstreetmap.org`, `https://*.tile.openstreetmap.org`) and reverse-geocoding (`https://nominatim.openstreetmap.org`, `https://photon.komoot.io`) endpoints, resolving the CSP block that forced all map tiles into a dark blank fallback.
  - Replaced CSS `filter` on `.xm-osm-dark .leaflet-tile-pane` with compositor-native `opacity: 0.84` over `#181a1f` to prevent Android Chromium WebView GPU compositing blackouts on 3D-transformed Leaflet tile panes, and added `touch-action: none` plus `max-width: none !important; max-height: none !important` on `.leaflet-tile` to protect tiles from Tailwind v4 preflight rules.
  - Extracted a shared, Android-resilient tile loader (`src/lib/utils/osm-tiles.ts`) with safe ESM/CJS Leaflet unwrapping, `Cache` API + CORS fetch with automatic direct `<img>` fallback (critical on Android WebView `http://tauri.localhost`), proper Blob URL revocation on both `onload` and `onerror`, legible light/dark offline grid tiles, unmount lifecycle guards, post-animation `invalidateSize()` passes, Svelte 5 `untrack()` guards, and native Tauri `plugin:opener|open_url` dispatch for `geo:` OsmAnd intents.

## [2.7.29] - 2026-11-26

### Fixed
- **Desktop Memo Shortcut `Could not locate Desktop directory: unknown path` (`plugins/tauri-plugin-xmemos-storage/src/desktop_shortcut.rs`, `packaging/flatpak/org.xmemos.app.yaml`, `src/lib/components/memo/MemoActionsSheet.svelte`)**:
  - Replaced the single `app.path().desktop_dir()` call with a resilient multi-tier `resolve_desktop_dir()` resolver across GNU/Linux (Tauri `desktop_dir()` → `$XDG_DESKTOP_DIR` → `~/.config/user-dirs.dirs` → `$HOME/Desktop`), Windows (`desktop_dir()` → `%OneDrive%\Desktop` → `%USERPROFILE%\Desktop`), and macOS (`desktop_dir()` → `$HOME/Desktop`), preventing failures on minimal Linux/Devuan/Wayland setups or Flatpak sandboxes where `user-dirs.dirs` is absent.
  - Added a narrow `--filesystem=xdg-desktop:create` permission in `packaging/flatpak/org.xmemos.app.yaml` (preserving strict offline sandboxing without `--filesystem=host`) plus an automatic `xdg-desktop-portal` native Save dialog fallback in `create_linux_desktop_entry()` whenever direct Desktop writes are restricted.

## [2.7.28] - 2026-11-26

### Fixed
- **Fatal Svelte 5 `effect_update_depth_exceeded` (Error 500) on Home Launch (`src/lib/state/memos.svelte.ts`, `src/routes/+page.svelte`, `src/routes/{explore,archived,memo/[uid],u/[username],settings/storage}/+page.svelte`, `src/lib/components/memo/MemoEditor.svelte`, `src/lib/components/media/ImageLightbox.svelte`)**:
  - Fixed an infinite reactive loop in `MemosState.hydrateSnapshot()` where checking `this.hydratedForUser === creatorId && this.items.length > 0` inside Home's `$effect` subscribed the effect to `memosState.items` whenever an account had 0 memos and then re-assigned a fresh `JSON.parse(...)` array to `this.items` on every flush, exceeding Svelte 5's 1000-iteration depth limit (`effect_update_depth_exceeded`).
  - Wrapped side-effecting data loaders and state resets inside `untrack()` across Home, Explore, Archived, Memo Detail, User Profile, Storage Settings, `MemoEditor`, and `ImageLightbox` so `$effect` blocks only track their explicit trigger dependencies and can never re-trigger themselves.

## [2.7.27] - 2026-11-26

### Fixed
- **Android Release R8 Minification (`:app:minifyUniversalReleaseWithR8`) & Clean Per-ABI Split Packaging (`.github/workflows/android-release.yml`, `plugins/tauri-plugin-xmemos-storage/android/consumer-rules.pro`)**:
  - Added `-dontwarn java.beans.**` (covering `java.beans.ConstructorProperties` and `java.beans.Transient` referenced optionally by Jackson's `Java7SupportImpl` inside Tauri Android) along with `-dontwarn javax.xml.**` and `-dontwarn org.w3c.dom.bootstrap.DOMImplementationRegistry` so AGP 9.x / R8 release minification completes cleanly.
  - Cleared `src-tauri/gen/android/app/src/main/jniLibs` alongside `outputs/apk` before each ABI pass in `build_abi()` so `arm64-v8a`, `armeabi-v7a`, and `x86_64` split APKs strictly bundle only their own native library.
- **Zero-Warning Svelte 5 Build (`src/lib/components/layout/BottomTabbar.svelte`)**:
  - Added `tabindex="-1"` to the `role="toolbar"` swipe track in `BottomTabbar.svelte` to resolve the `a11y_interactive_supports_focus` warning during `vite build`.

## [2.7.26] - 2026-11-26

### Fixed
- **Rust Integration Test Panic & Startup Directory Race (`src-tauri/src/state.rs`, `src-tauri/src/service/media.rs`, `src-tauri/src/service/backup.rs`, `src-tauri/tests/compatibility_tests.rs`)**:
  - Restored deterministic creation of `backups/`, `temp/`, and `thumbs/` inside `AppState::initialize()` (while keeping stale `temp/` cleanup and FTS5 parity verification on the non-blocking background thread), fixing the `test_path_traversal_and_zip_slip_rejected` (`NotFound`) panic in GitHub Actions `cargo test --locked`.
  - Added defensive `fs::create_dir_all()` guards in `media::generate_thumbnail()`, `backup::execute_atomic_import()`, and `compatibility_tests.rs` so thumbnail writes and atomic rollback snapshots never fail even if cache/temp directories are pruned at runtime.
- **Deterministic Multi-ABI Android APK Packaging (`.github/workflows/android-release.yml`, `scripts/verify-repro.sh`)**:
  - Cleared `src-tauri/gen/android/app/build/outputs/apk` at the start of `build_abi()` so `target_abis=all` strictly signs and packages each ABI's freshly built APK instead of re-selecting the alphabetically first ABI output (`aarch64`).
  - Hardened `scripts/verify-repro.sh` APK discovery to match release APKs deterministically (`*release-unsigned.apk` / `*release.apk`).
- **Locked Cargo Version SSOT & Documentation Parity (`scripts/sync-version.ts`, `docs/REPRODUCIBLE_BUILDS.md`, `docs/ai/TASK_PLAYBOOK.md`)**:
  - Extended `scripts/sync-version.ts` (`pnpm version:sync` / `pnpm version:check`) to validate and synchronize the root `xmemos` version in `src-tauri/Cargo.lock`, preventing `cargo clippy --locked` / `cargo test --locked` drift after version bumps.
  - Updated `docs/REPRODUCIBLE_BUILDS.md` (native `flatpak-builder` reference and `v2.7.26` verification examples) and `docs/ai/TASK_PLAYBOOK.md`.

## [2.7.25] - 2026-11-26

### Fixed
- **SvelteKit 3 Typecheck & Pre-Flight Build (`pnpm check`)**:
  - Removed the removed `@sveltejs/kit` v2 `HandleClientError` type import in `src/hooks.client.ts` and annotated `handleError` parameters explicitly for SvelteKit 3.4 compatibility.
  - Fixed `normalizedUsername` self-initialization typo in `src/routes/auth/+page.svelte` to normalize `username.trim().toLowerCase()`.
  - Corrected the desktop sticky-note menu i18n key in `src/lib/components/memo/MemoActionsSheet.svelte` to `'sticky.open'`.
  - Added `role="toolbar"` to the swipeable track in `src/lib/components/layout/BottomTabbar.svelte` to satisfy Svelte 5 a11y checks (`a11y_no_static_element_interactions`).

## [2.7.24] - 2026-11-25

### Fixed
- **Android Home-Screen Sticky Memo Widget (`"Masalah memuat widget"` / Inflation Crash)**:
  - Replaced the unsupported `<View>` element in `plugins/tauri-plugin-xmemos-storage/android/src/main/res/layout/widget_memo_sticky.xml` with `@RemoteView`-compliant `<FrameLayout>` so Android's `AppWidgetHostView` inflates the sticky widget cleanly without throwing `InflateException`.
  - Updated `XmemosMemoWidgetProvider` in `plugins/tauri-plugin-xmemos-storage/android/src/main/AndroidManifest.xml` to `android:exported="true"` as required on Android 12+ (`targetSdk >= 31`) for `APPWIDGET_UPDATE` broadcasts.
  - Added `plugins/tauri-plugin-xmemos-storage/android/src/main/res/raw/keep.xml` to protect widget layout, drawable, XML provider metadata, and colors from AGP resource shrinking in release APKs, and switched pending widget preference writes in `XmemosMemoWidgetProvider.kt` to synchronous `commit()` to eliminate pin-time race conditions.
- **Android Soft-Keyboard Layout Stability (Account Switcher, PIN Lock & Memo Editor)**:
  - Added W3C Visual Viewport tracker (`src/lib/utils/keyboard.svelte.ts`, wired in `src/routes/+layout.svelte`) to sync `--xm-vvh`, `--xm-vvt`, and `--xm-kb-inset` cleanly on Android WebView.
  - Refactored `src/lib/components/auth/AccountSwitcherSheet.svelte`, `src/lib/components/auth/PinLockOverlay.svelte`, and `src/lib/components/memo/MemoEditor.svelte` so password/PIN inputs and memo editing stay centered and stably positioned above the Android on-screen keyboard without jumping or clipping.

### Changed
- **Silky iOS-Style Mobile Bottom Navigation Bar (`src/lib/components/layout/BottomTabbar.svelte`)**:
  - Upgraded the mobile bottom bar to a 60/120fps GPU-composited floating pill dock (`transform: translate3d`) with real-time 1:1 horizontal finger scrubbing, spring easing (`cubic-bezier(0.22, 1, 0.36, 1)`), and automatic hide when the soft keyboard is open.
- **Lightweight Android Memo Options Popup (`src/lib/components/memo/MemoActionsSheet.svelte`)**:
  - Replaced the heavy multi-layer blurred mobile ActionSheet on `< 1024px` viewports with a GPU-composited, zero-scrim-blur iOS Action Sheet for instant, smooth opening/closing on Android WebView while keeping the desktop macOS floating menu intact.

## [2.7.23] - 2026-11-25

### Fixed & Improved
- **Case-Insensitive Lowercase Usernames (`src-tauri/src/service/auth.rs`, `src-tauri/src/store/user.rs`, `src-tauri/src/commands/{auth,user}.rs`, `src/routes/auth/+page.svelte`, `src/routes/settings/users/+page.svelte`)**:
  - Normalized all usernames to lowercase on sign-up, host setup, admin creation, and profile updates (`normalize_username`) and switched SQLite lookups to case-insensitive `LOWER(username) = LOWER(?1)` so sign-in works seamlessly regardless of keyboard capitalization (including existing databases), paired with `autocapitalize="none"` and live lowercase input normalization on mobile keyboards.
- **Android APK Launcher Icon Fix (`.github/workflows/android-release.yml`, `docs/FDROID.md`)**:
  - Added `pnpm tauri icon src-tauri/icons/icon.png` immediately after `pnpm tauri android init --ci` so every Android APK ships the canonical Xmemos Liquid Glass launcher & adaptive `mipmap-*` icons instead of Tauri's default template icon.
- **Android Release Stability, R8 Keep Rules & Error 500 Resilience (`plugins/tauri-plugin-xmemos-storage/android/{build.gradle.kts,consumer-rules.pro}`, `.github/workflows/android-release.yml`, `src-tauri/tauri.conf.json`, `src/hooks.client.ts`)**:
  - Added `consumer-rules.pro` + app-level ProGuard `-keep` rules for `app.tauri.**` and `org.xmemos.storage.**` plus `android.enableR8.fullMode=false` so R8 minification never strips `@TauriPlugin`/`@Command` JNI bindings or widget classes on release APKs.
  - Added `http://asset.localhost` to CSP `img-src`/`media-src` for Android `convertFileSrc`, added `src/hooks.client.ts` for clear client error reporting, and fixed `/auth` redirect history handling (`replaceState: true`).
- **Faster Instant Android Startup (`src-tauri/src/state.rs`, `src-tauri/src/lib.rs`, `src/routes/+layout.svelte`)**:
  - Deferred non-critical directory creation (`backups/`, `temp/`, `thumbs/`) to the background maintenance thread, gated desktop-only window reveal threads and `KeyboardShortcutsModal` behind desktop checks, and made shortcut listener wiring non-blocking so Android launches directly to the first frame with minimal boot latency.

## [2.7.22] - 2026-11-25

### Added
- **Adaptive Debloated/Air-Gapped Windows WebView2 Startup Resolver (`src-tauri/src/main.rs`, `.github/workflows/windows-release.yml`, `README.md`, `docs/REPRODUCIBLE_BUILDS.md`)**:
  - Added a zero-extra-crate Windows startup resolver (`windows_webview2::prepare_or_prompt`) that automatically detects a local portable **WebView2 Fixed Version Runtime** (`webview2\` next to `Xmemos.exe`) via `WEBVIEW2_BROWSER_EXECUTABLE_FOLDER`, checks official Evergreen WebView2 Runtime registry keys (`{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}`, completely independent of the Microsoft Edge browser), and displays a clear native Win32 guidance dialog instead of crashing if an aggressive Windows debloater removed WebView2 Runtime completely.

## [2.7.21] - 2026-11-25

### Changed
- **100% Zero-Network Offline & Lean Windows NSIS Installer (`src-tauri/tauri.conf.json`, `.github/workflows/release.yml`, `README.md`, `docs/REPRODUCIBLE_BUILDS.md`)**:
  - Set `bundle.windows.webviewInstallMode.type` to `"skip"` (0 MB overhead, zero network requests during installation or runtime) so the ~5–6 MB NSIS installer is strictly 100% offline while leveraging the OS-preinstalled Microsoft Edge WebView2 Runtime on Windows 10/11.

## [2.7.20] - 2026-11-25

### Optimized
- **Lean Windows Installer & GNU/Linux AppImage Bundle Sizing (`src-tauri/tauri.conf.json`, `.github/workflows/{linux,windows,release}.yml`, `README.md`, `docs/REPRODUCIBLE_BUILDS.md`)**:
  - Switched Windows NSIS `webviewInstallMode.type` from `"offlineInstaller"` (which embedded the ~280 MB standalone Chromium WebView2 installer, inflating the setup binary to 289 MB) to Tauri v2's official lean default `"downloadBootstrapper"` (0 MB overhead; uses the OS-preinstalled WebView2 Runtime offline on Windows 10/11 and trims `Xmemos-v2.7.20-windows-x64-setup.exe` to ~5–6 MB).
  - Disabled `bundle.linux.appimage.bundleMediaFramework` (`false`), removed `NO_STRIP: 'true'` inside the `ubuntu:22.04` AppImage container, and dropped unused GStreamer apt packages from `linux-release.yml` to eliminate ~50–65 MB of bundled GStreamer/FFmpeg bloat and strip bundled shared libraries cleanly.

## [2.7.19] - 2026-11-25

### Fixed
- **Idempotent & Retry-Hardened GitHub Release Publishing (`.github/workflows/release.yml`)**:
  - Replaced the single-shot `gh release create ... dist-release/*` call (which failed on transient GitHub API `HTTP 500` responses) with an atomic, idempotent 3-phase publisher featuring 5-attempt exponential-backoff retries (`retry_gh`): create/update `--draft` release with explicit `--target "${GITHUB_SHA}"`, upload every platform asset individually with `gh release upload --clobber`, then publish via `gh release edit --draft=false` (fully compatible with GitHub Immutable Releases and safe re-runs).

## [2.7.18] - 2026-11-25

### Fixed
- **macOS Runner Pool Capacity Resilience (`.github/workflows/macos-release.yml`)**:
  - Consolidated the dual parallel `macos-15` (ARM64) matrix jobs into a single standard `macos-15-intel` runner building both `aarch64-apple-darwin` (`arm64`) and `x86_64-apple-darwin` (`x64`) DMGs sequentially, bypassing GitHub Actions' macOS ARM64 runner pool capacity timeouts (`The job was not acquired by Runner of type hosted even after multiple attempts`).

## [2.7.17] - 2026-11-25

### Improved
- **Clean Kotlin 2.x Gradle DSL in Android Storage Plugin (`plugins/tauri-plugin-xmemos-storage/android/build.gradle.kts`)**:
  - Migrated deprecated `kotlinOptions { jvmTarget = "17" }` to the modern top-level `kotlin { compilerOptions { jvmTarget.set(JvmTarget.JVM_17) } }` DSL and removed the unused `consumerProguardFiles("consumer-rules.pro")` declaration to eliminate Gradle warnings during Android builds.

## [2.7.16] - 2026-11-25

### Fixed
- **macOS Unsigned/Ad-Hoc DMG Bundling CI Fix (`.github/workflows/macos-release.yml`)**:
  - Unset empty `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY`, `APPLE_ID`, `APPLE_PASSWORD`, and `APPLE_TEAM_ID` environment variables before invoking `pnpm tauri build` so GitHub Actions does not pass empty strings that trigger `security import -f pkcs12` (`SecKeychainItemImport: Unable to decode the provided data`) when Apple Developer signing secrets are not configured, while preserving automatic full signing/notarization once those secrets are added.

## [2.7.15] - 2026-11-25

### Fixed
- **Zero-Warning macOS Build (`plugins/tauri-plugin-xmemos-storage/src/desktop_shortcut.rs`)**:
  - Gated `preview` sanitization behind `#[cfg(not(target_os = "macos"))]` (resolving `unused_variables` warning on `aarch64`/`x86_64` macOS builds) and collapsed the nested `src_icns` check for strict Clippy compliance.

## [2.7.14] - 2026-11-25

### Fixed
- **Android Mobile Rust Compilation (`src-tauri/src/commands/system.rs`)**:
  - Gated desktop-only `window_pin` module import and `main_win.unminimize()` behind `#[cfg(desktop)]`, and fixed `E0505` borrow-then-move of `payload` in the `#[cfg(not(desktop))]` branch of `system_open_sticky_memo_window` so `aarch64`/`armv7`/`x86_64` Android builds compile cleanly with zero warnings.
- **GNU/Linux Flatpak CI SDK Discovery (`.github/workflows/linux-release.yml`)**:
  - Replaced sandboxed `flatpak run org.flatpak.Builder` (which failed with `Unable to find sdk org.gnome.Sdk version 51` on headless GitHub Actions runners due to `flatpak-spawn --host` D-Bus/user-installation isolation) with native `flatpak-builder` (`apt-get install flatpak-builder elfutils`) plus `--disable-rofiles-fuse` and `kernel.apparmor_restrict_unprivileged_userns=0` for Ubuntu 24.04 compatibility.
- **Zero-Warning Windows Build (`plugins/tauri-plugin-xmemos-storage/src/desktop_shortcut.rs`)**:
  - Restricted `use std::path::PathBuf;` to `#[cfg(target_os = "macos")]` so Windows compiles with zero warnings.

## [2.7.13] - 2026-11-25

### Fixed
- **GNU/Linux AppImage Container CI Fix (`.github/workflows/linux-release.yml`)**:
  - Fixed `Error: spawn npm ENOENT` during `pnpm/action-setup@v6` inside the bare `ubuntu:22.04` AppImage build container (`x86_64` and `aarch64`) by running `actions/setup-node@v7` before `pnpm/action-setup@v6` (with native `cache: true`), ensuring `npm` is on `PATH` for `pnpm/action-setup@v6`'s `npm ci` bootstrap.

## [2.7.12] - 2026-11-25

### Changed & Improved
- **Single-Source-of-Truth Changelog & Automated Flatpak AppStream Metadata (`packaging/flatpak/org.xmemos.app.metainfo.xml`, `scripts/sync-version.ts`, `docs/ai/TASK_PLAYBOOK.md`)**:
  - Removed duplicated manual changelog `<description>` blocks from `packaging/flatpak/org.xmemos.app.metainfo.xml` and replaced them with a clean, AppStream 1.2-compliant `<release version="2.7.12" date="2026-11-25">` entry referencing `CHANGELOG.md` via `<url type="details">`.
  - Upgraded `scripts/sync-version.ts` (`pnpm version:sync` & `pnpm version:check`) to automatically read the release date from `CHANGELOG.md` and keep `packaging/flatpak/org.xmemos.app.metainfo.xml` and the About page fallback version in sync with `package.json`, eliminating double editing.

## [2.7.11] - 2026-11-25

### Fixed
- **GitHub Actions `dtolnay/rust-toolchain` Required Input Fix (`.github/workflows/{release,android-release,linux-release,windows-release,macos-release}.yml`)**:
  - Passed explicit `toolchain: '1.100.0'` (matching `rust-toolchain.toml`, since `dtolnay/rust-toolchain@master` ignores `rust-toolchain.toml` and requires the `toolchain` input) along with `components: clippy, rustfmt` in `release.yml` and Android targets in `android-release.yml` across all 6 workflow jobs.

## [2.7.10] - 2026-11-25

### Fixed
- **Strict Rust 1.100 Clippy `-D warnings` Compliance (`src-tauri/src/commands/system.rs`, `src/lib/api/shortcut.ts`, `src-tauri/src/service/media.rs`, `src-tauri/src/service/window_pin.rs`)**:
  - Replaced the 9-parameter signature of `system_open_sticky_memo_window` with a typed `OpenStickyWindowPayload` DTO (resolving `clippy::too_many_arguments`) and updated `shortcutApi.openStickyMemoWindow()`.
  - Removed the redundant `.into_iter()` inside `.chain(...)` in `service/media.rs` (`clippy::useless_conversion`).
  - Removed the trailing `return` expression in `service/window_pin.rs` (`clippy::needless_return`).

## [2.7.9] - 2026-11-25

### Fixed & Improved
- **Complete Sticky Note Navigation Isolation, Inline Comments & Main-Window Routing (`src-tauri/src/commands/system.rs`, `src-tauri/src/lib.rs`, `src/lib/api/shortcut.ts`, `src/lib/components/map/LocationPreviewPopover.svelte`, `src/routes/+layout.svelte`, `src/routes/sticky/[uid]/+page.svelte`)**:
  - Added validated `system_focus_main_route` command emitting `xmemos:navigate-main` directly to the `"main"` window (`app.emit_to("main", ...)` + `getCurrentWindow().listen(...)` in `+layout.svelte`) so clicking **Open on full map (`↗`)** in `LocationPreviewPopover`, `#tag` pills, linked memos, or **Open in Xmemos** always opens cleanly in the main Xmemos window instead of hijacking the Sticky Note window.
  - Added an **inline expandable Comments panel** (`memoApi.listComments`) when clicking the `💬` comment count inside a Sticky Note, with an optional **Open in Xmemos →** link.
  - Blocked global app navigation shortcuts (`Alt+1–5`, `G` chords, `Ctrl+K`, `Ctrl+,`, `/`) inside `/sticky/*` windows in `+layout.svelte`, and clamped `LocationPreviewPopover` height safely for compact Sticky Note viewports.

## [2.7.8] - 2026-11-25

### Fixed
- **Cargo.toml Dependency Pin Fix (`src-tauri/Cargo.toml`)**: Restored `tauri-build = { version = "2.7.6", features = [] }` so `Cargo.toml` matches `Cargo.lock` 1:1 under `cargo --locked`.

## [2.7.7] - 2026-11-25

### Fixed
- **Zero-Warning Rust Build (`src-tauri/src/service/window_pin.rs`)**: Removed the unused `use std::process::Command;` import inside `try_kwin_dbus_keep_above()` so `cargo clippy --locked --all-targets -- -D warnings` passes cleanly.

## [2.7.6] - 2026-11-25

### Fixed
- **True Cross-OS & KDE Plasma Wayland Always on Top Engine (`src-tauri/src/service/window_pin.rs`, `src-tauri/src/commands/system.rs`, `src/lib/api/shortcut.ts`, `src/routes/sticky/[uid]/+page.svelte`, `src/lib/i18n/locales/{en,id,es}.ts`)**:
  - Solved the Wayland `xdg_toplevel` limitation where GTK3 `set_always_on_top(true)` is a silent no-op on native Wayland sessions: added modular `window_pin.rs` implementing **KDE Plasma 5 & 6 Wayland** `keepAbove` via KWin's official D-Bus Scripting interface (`org.kde.KWin /Scripting`, supporting both Plasma 6 `workspace.windowList()` + `/Scripting/Script<id>` and Plasma 5 `workspace.clientList()` + `/<id>`) matched by exact Sticky Note caption (`<Title> · Xmemos [<uid8>]`), plus **Hyprland Wayland** `hyprctl dispatch pin`.
  - Preserved native `set_always_on_top` on **Windows 10/11** (`HWND_TOPMOST`), **macOS 12+** (`NSFloatingWindowLevel`), and **GNU/Linux X11 / XWayland** (`_NET_WM_STATE_ABOVE`), and returned structured `AlwaysOnTopResult` so unsupported Wayland compositors (e.g. GNOME Mutter) display an honest localized hint instead of silently failing.

## [2.7.5] - 2026-11-25

### Fixed
- **Sticky Note Always on Top Toggle Fix (`src-tauri/src/commands/system.rs`, `src/routes/sticky/[uid]/+page.svelte`)**:
  - Fixed a bug where clicking **On Top** on GNU/Linux immediately reverted to disabled because `system_set_sticky_always_on_top` synchronously called `win.is_always_on_top()` before GTK/X11 processed the async `_NET_WM_STATE_ABOVE` hint, returning stale `false` which the frontend then re-applied as `setAlwaysOnTop(false)`.
  - `system_set_sticky_always_on_top` now applies `win.set_always_on_top(always_on_top)` (raising focus when enabled) and returns the applied state directly without stale synchronous readback, and the frontend removes the redundant second `setAlwaysOnTop()` call.

## [2.7.4] - 2026-11-25

### Fixed
- **Prevent Sticky Note Edge-Snap Maximize / Quick-Tiling on KDE Plasma, GNOME & Windows (`src-tauri/src/commands/system.rs`, `src-tauri/capabilities/desktop.json`, `src/lib/utils/platform.ts`, `src/routes/sticky/[uid]/+page.svelte`)**:
  - Configured `WebviewWindowBuilder` for floating Sticky Note windows with `.maximizable(false)` and `.max_inner_size(720.0, 860.0)` so KDE Plasma KWin (Wayland & X11), GNOME Mutter, and Windows Snap never snap-maximize or stretch Sticky Notes to fullscreen when dragged to top or side screen edges.
  - Added drag-only `handleStickyDragMouseDown()` (eliminating accidental double-click maximize on the Sticky Note header) plus an automatic `isMaximized()` → `unmaximize()` runtime safeguard in `/sticky/[uid]/+page.svelte` (with `core:window:allow-unmaximize` in `desktop.json`).

## [2.7.3] - 2026-11-25

### Added & Improved
- **Rich Media, Location, Relations & Reactions in Sticky Notes (`src/routes/sticky/[uid]/+page.svelte`)**:
  - Upgraded floating Desktop Sticky Notes to render full memo content beyond plain text: `AttachmentGrid` (images, videos, audio, downloadable files) with scoped `ImageLightbox`, `MemoLocationPreview` with scoped `LocationPreviewPopover` mini-map, linked memo relations (`Link2`), interactive emoji reaction chips (`memoApi.toggleReaction`), comment count indicator, visibility icon (`Globe`/`Users`/`Lock`), and amber pinned corner fold.
- **Cross-OS Native Always on Top Button (`src-tauri/src/commands/system.rs`, `src-tauri/src/lib.rs`, `src/lib/api/shortcut.ts`, `src/routes/sticky/[uid]/+page.svelte`, `src/lib/i18n/locales/{en,id,es}.ts`)**:
  - Added a dedicated, clearly labeled **On Top** (`Always on Top`) pill button in the Sticky Note header backed by native Rust `system_set_sticky_always_on_top` (`set_always_on_top` + verified `is_always_on_top` state readback + post-show re-sync for X11/Wayland/Windows/macOS compositors) with localized active/inactive toast feedback.

## [2.7.2] - 2026-11-25

### Fixed & Improved
- **Sticky Note Quick-Edit Save Bug Fix (`src/routes/sticky/[uid]/+page.svelte`)**:
  - Fixed `invalid args 'payload' for command 'memo_update': missing field 'id'` by passing `{ id: view.memo.id, content: trimmed }` matching `memoApi.update()` and Rust `UpdateMemoRequest`.
- **Zero-Flash Smooth Reveal & Konsta UI macOS/iOS Theme Alignment (`src-tauri/src/commands/system.rs`, `src/lib/api/shortcut.ts`, `src/routes/sticky/[uid]/+page.svelte`, `plugins/tauri-plugin-xmemos-storage/android/src/main/res/{values,values-night,drawable,layout}/*`)**:
  - Eliminated black startup blink and visual stutter when opening Desktop Sticky Note windows by initializing `WebviewWindowBuilder` with `.visible(false)` and theme/palette-matched `.background_color(Color(r, g, b, 255))`, revealing strictly after double-`requestAnimationFrame` first paint (backed by a 700ms Rust failsafe).
  - Redesigned Desktop Sticky Notes in **Konsta UI macOS 2026 Liquid Glass** with a crisp `3px` top accent bar, translucent specular header, `AppIcon` footer branding, WCAG AAA text contrast, and 6 curated high-contrast palettes (**Graphite Glass**, **Amber Glow**, **Oceanic Blue**, **Emerald Mint**, **Rose Quartz**, **Indigo Mist**).
  - Aligned the Android Home-Screen Sticky Note widget with **Konsta UI iOS Theme** via automatic `values/` (Light `#FFFFFF` + `#007AFF`) and `values-night/` (Dark `#1C1C1E` + `#0A84FF`) system theme adaptation.

## [2.7.1] - 2026-11-25

### Added & Fixed
- **MS Sticky Notes-Style Interactive Memo Shortcuts for Desktop & Android (`src/routes/sticky/[uid]/+page.svelte`, `src-tauri/src/commands/system.rs`, `src-tauri/capabilities/{default,desktop}.json`, `XmemosMemoWidgetProvider.kt`, `XmemosStoragePlugin.kt`, `MemoActionsSheet.svelte`, `+layout.svelte`, `src/lib/i18n/locales/{en,id,es}.ts`)**:
  - **Desktop Floating MS Sticky Notes (`/sticky/[uid]`)**: Added `async fn system_open_sticky_memo_window` (`WebviewWindowBuilder`, avoiding WebView2 sync deadlocks) with smart RAM efficiency: label deduplication (`sticky-<uid>`), a hard cap of 8 concurrent sticky windows, zero heavy layout chrome in `+layout.svelte` (`isStickyRoute` skips `BottomTabbar`, `MemoActionsSheet`, `ImageLightbox`, map/display popovers, and modals), free desktop drag & resize with debounced `localStorage` geometry/color/`alwaysOnTop` persistence, 5 MS Sticky Notes color themes, interactive task checkboxes, inline quick edit, and **Open in Xmemos** (`system_focus_main_memo`). Desktop shortcuts (`--open-memo=<uid>`) now launch straight into the floating Sticky Note window.
  - **Android Resizable MS Sticky Notes Home-Screen Widget (`XmemosMemoWidgetProvider.kt`, `widget_memo_sticky.xml`, `xmemos_memo_widget_info.xml`)**: Added a zero-background-service (`updatePeriodMillis="0"`), horizontally & vertically resizable `AppWidgetProvider` styled like MS Sticky Notes, pinned via official `AppWidgetManager.requestPinAppWidget()` with `EXTRA_APPWIDGET_PREVIEW` on Android 8.0+ (API 26–36) and automatic `ShortcutManagerCompat.requestPinShortcut()` fallback on API 24–25 or restricted launchers.
- **Fixed Rust Compiler Warning (`plugins/tauri-plugin-xmemos-storage/src/desktop_shortcut.rs`)**: Scoped `use std::path::PathBuf;` strictly under `#[cfg(any(target_os = "windows", target_os = "macos"))]` to eliminate the `unused_imports` warning on GNU/Linux builds under `-D warnings`.

## [2.7.0] - 2026-11-25

### Added
- **Cross-OS Native Memo Shortcuts for Android & Desktop (`XmemosStoragePlugin.kt`, `desktop_shortcut.rs`, `src/lib/api/shortcut.ts`, `MemoActionsSheet.svelte`, `src/routes/+layout.svelte`, `src/lib/i18n/locales/{en,id,es}.ts`)**:
  - **Android Home-Screen Pinned & Dynamic Memo Shortcuts**: Implemented via official AndroidX `ShortcutManagerCompat.isRequestPinShortcutSupported()`, `requestPinShortcut()`, and `pushDynamicShortcut()` (`androidx.core:core-ktx:1.15.0`) with a custom `192×192` widget-style Liquid Glass adaptive icon (`IconCompat.createWithAdaptiveBitmap` on API 26+, `createWithBitmap` on API 24–25) showing the memo's initial, mini preview lines, and optional amber pin corner fold.
  - **Android Cold & Warm Deep-Link Routing**: Implemented Tauri v2 `Plugin.load(webView)` and `Plugin.onNewIntent(intent)` hooks paired with `consumePendingMemoShortcut` and `memoShortcutOpened` plugin events so tapping a pinned memo shortcut opens `/memo/<uid>` cleanly whether Xmemos is closed or already running (while strictly preserving local 6-digit PIN / Biometric lock enforcement).
  - **Desktop Native Memo Shortcuts (Zero Extra Crates)**:
    - **GNU/Linux (X11 & Wayland)**: Writes executable (`0755`) FreeDesktop Desktop Entry Specification `.desktop` files to `~/Desktop` with automatic Flatpak (`flatpak run org.xmemos.app --open-memo=<uid>`), AppImage (`$APPIMAGE`), and native binary detection.
    - **Windows 10/11**: Creates native Shell `.lnk` shortcuts on `%USERPROFILE%\Desktop` via hidden `WScript.Shell.CreateShortcut` with strict environment-variable parameter passing (zero script/shell injection).
    - **macOS 12+**: Generates lightweight `LSUIElement` `.app` launcher bundles on `~/Desktop` with copied `AppIcon.icns` invoking `/usr/bin/open -a <Xmemos.app> --args --open-memo=<uid>` silently without opening Terminal.
  - **Unified Memo Options Action & Localization**: Added **Pin to Home Screen** (Android) / **Add to Desktop** (Desktop) in `MemoActionsSheet.svelte` with strict `^[a-zA-Z0-9_-]{1,64}$` UID validation, `buildMemoShortcutMeta` unit tests (`shortcut.test.ts`), and full `en`/`id`/`es` localization.

## [2.6.6] - 2026-11-25

### Added & Hardened
- **Sign Up & Host Setup Re-Type Password Confirmation (`src/routes/auth/+page.svelte`, `src/lib/i18n/locales/{en,id,es}.ts`)**:
  - Added a dedicated **Confirm Password** (`#auth-password-confirm`) re-type input during both **Sign Up** and initial **Host Setup** so users confirm their password before creating a local account.
  - Added live inline minimum-length (`>= 4` chars) and mismatch validation (`aria-invalid`, `role="alert"`), disabled submit guard on invalid/mismatched entries, clean `confirmPassword` reset when toggling between Sign In and Sign Up, and `novalidate` form handling with localized toast errors across `en`, `id`, and `es`.

## [2.6.5] - 2026-11-25

### Changed & Improved
- **1:1 MemoCard PNG Export Share Card Redesign (`src/lib/utils/share-card.ts`, `src/lib/utils/share-card-markdown.ts`, `src/lib/utils/share-card.test.ts`)**:
  - Rebuilt the HTML5 Canvas 2D share-card renderer to match the in-app `MemoCard.svelte` UI/UX **1:1** across both Light (`#f2f2f7`) and Dark (`#000000`) `--xm-bg-mesh` Liquid Glass themes: `.glass-surface` rounded card with specular top rim, amber top-right pinned corner-fold indicator, circular initial avatar badge, single-line `Author · Timestamp` header, and right-aligned Lucide-accurate visibility icon (`Globe` / `Users` / `Lock`).
  - Added modular `share-card-markdown.ts` block/inline parser (with Vitest coverage in `share-card.test.ts`) so exported cards faithfully render `.memo-prose` headings, bold/italic/strikethrough, inline code pills, `#tag` accent pills, underlined links, interactive-style task checkboxes, bullet/numbered lists, blockquotes with `3px` accent bar, and Official Memos `#efefe9`/`#222326` fenced code blocks with language headers.
  - Added support for image attachment grids (cover-cropped with `+N` overflow), non-image file pills, location & linked-memo metadata rows, and grouped reaction/comment chips in the footer, paired with a clean minimal `[AppIcon] Xmemos` mark — removing the old `"Offline Memos"` footer text.

## [2.6.4] - 2026-11-25

### Fixed
- **6-Digit PIN Validation & Android/IME Input Fix (`src/lib/utils/pin.ts`, `src/lib/utils/pin.test.ts`, `src/routes/settings/account/+page.svelte`, `src/lib/components/auth/PinLockOverlay.svelte`)**:
  - Fixed a Svelte 5 template interpolation bug where `pattern="[0-9]{6}"` compiled into `pattern="[0-9]6"` (matching only 2-character strings ending in `6`), which falsely blocked valid 6-digit PIN submissions with the browser's `"Match the requested format"` tooltip in both **Settings → Account** and `PinLockOverlay`.
  - Exported `PIN_PATTERN = '[0-9]{6}'` from `src/lib/utils/pin.ts`, bound `pattern={PIN_PATTERN}` on all 6-digit PIN inputs, and added `novalidate` to the Lock Settings and PIN Unlock forms so localized inline validation and toast feedback always govern form submission cleanly.
  - Updated `handlePinKeydown()` to allow Android/IME `Unidentified`, `Process`, and `isComposing` keydown events so mobile soft keyboards commit digits cleanly into `sanitizePinInput()`, backed by expanded Vitest coverage in `pin.test.ts`.

## [2.6.3] - 2026-11-25

### Changed & Improved
- **macOS 2026 Liquid Glass Application Icon Redesign & Cross-OS Asset Pipeline (`src-tauri/icons/icon.svg`, `static/icon.svg`, `src-tauri/icons/{32x32.png,128x128.png,128x128@2x.png,icon.png,icon.ico,icon.icns}`, `fastlane/metadata/android/en-US/images/icon.png`, `packaging/flatpak/org.xmemos.app.yaml`, `.github/workflows/linux-release.yml`, `src/app.html`)**:
  - Redesigned the canonical `1024×1024` vector application icon (`icon.svg`, lightweight ~10 KB pure standard SVG) strictly aligned with **Apple HIG macOS 2026 Liquid Glass** standards: mathematically smooth `G2`-continuous Apple superellipse (`864×864` optical grid), oceanic sapphire substrate with subsurface caustics and Fresnel luminous bevel rim, stacked frosted-glass memo card with signature folded corner, and interlocking Liquid Glass **X** monogram legible down to `16×16`.
  - Regenerated all native OS icon bundles from a 2048×2048 supersampled 32-bit RGBA master for crisp rendering across **Android** (512×512 launcher & Fastlane icon), **GNU/Linux X11/Wayland** (hicolor `32×32`, `128×128`, `256×256`, `512×512` PNGs + Flatpak `hicolor/scalable/apps/org.xmemos.app.svg`), **Windows 10/11** (7-layer PNG-compressed RGBA `icon.ico` from `16×16` to `256×256`), and **macOS** (11-chunk standard + `@2x` Retina `icon.icns` up to `1024×1024`), plus `<link rel="icon" type="image/svg+xml" href="/icon.svg" />` in `src/app.html`.
- **Unified App Icon Branding on SignIn/SignUp, Desktop Sidebar & About (`src/lib/components/ui/AppIcon.svelte`, `src/routes/auth/+page.svelte`, `src/lib/components/layout/AppShell.svelte`, `src/routes/settings/about/+page.svelte`)**:
  - Added modular Svelte 5 `AppIcon.svelte` with `$props.id()`-scoped SVG `<defs>` IDs for collision-free inline vector rendering.
  - Replaced the `lucide-svelte` `<Shield>` icon on `/auth` (SignIn / SignUp / Host Setup), the Desktop Sidebar brand badge in `AppShell.svelte`, and added the matching app emblem in **Settings → About** for complete visual consistency.

## [2.6.2] - 2026-11-25

### Changed & Hardened
- **Modular GitHub Actions Reusable Workflows Architecture (`.github/workflows/release.yml`, `android-release.yml`, `linux-release.yml`, `windows-release.yml`, `macos-release.yml`)**:
  - Refactored the monolithic multi-platform release workflow into a concise, readable `release.yml` orchestrator (`workflow_dispatch` only) that invokes four dedicated `workflow_call` reusable workflows:
    - `.github/workflows/android-release.yml` — Deterministic v2/v3 signed Android APKs + optional 2nd-pass SHA-256 verification.
    - `.github/workflows/linux-release.yml` — GNU/Linux cross-distro AppImage (`ubuntu:22.04` / `glibc 2.35`) & sandboxed Flatpak GNOME 51 single-file offline bundle (`x86_64` & `aarch64`).
    - `.github/workflows/windows-release.yml` — Windows 10/11 Full Offline NSIS Installer (embedded WebView2) & deterministic Portable `.zip`.
    - `.github/workflows/macos-release.yml` — macOS 12+ Apple Silicon (`arm64`) & Intel (`x64`) `.dmg` images.
  - Replaced broad secret inheritance with **explicit per-platform secret passing** (Android keystore secrets isolated strictly to `android-release.yml`, Apple signing secrets strictly to `macos-release.yml`, zero secrets passed to `linux-release.yml` and `windows-release.yml`) and strict `permissions: contents: read` boundaries across all called workflows.

## [2.6.1] - 2026-11-25

### Fixed & Hardened
- **Strict 6-Digit Numeric PIN Settings, Confirmation Input & Cold-Start Auto-Lock (`src/lib/utils/pin.ts`, `src/lib/utils/pin.test.ts`, `src/routes/settings/account/+page.svelte`, `src/lib/components/auth/PinLockOverlay.svelte`, `src/lib/state/auth.svelte.ts`, `src-tauri/src/service/auth.rs`, `src/lib/i18n/locales/{en,id,es}.ts`)**:
  - Added a dedicated **Confirm 6-Digit PIN** (re-type) field below the PIN input in **Settings → Account** matching the Account Password confirmation workflow, complete with live `0/6` digit counters, inline mismatch & 6-digit validation errors, an active **PIN Active** badge, and a **Lock Now** button.
  - Enforced strict numeric-only (`0–9`) and exact 6-digit input guards across both Settings and `PinLockOverlay` (`handlePinKeydown` blocking letters/symbols/spaces + `sanitizePinInput` stripping non-digits on paste/IME + `pattern="[0-9]{6}"`).
  - Fixed cold-start app lock enforcement in `authState.refresh()` so accounts with an active 6-digit PIN automatically present `PinLockOverlay` upon reopening the application, and localized all lock-overlay labels/errors across `en`, `id`, and `es`.
  - Hardened Rust backend `set_user_lock_config` and `verify_user_pin` to reject non-6-digit ASCII inputs early and hash 6-digit PINs with `bcrypt` cost `10`.

## [2.6.0] - 2026-11-25

### Added & Improved
- **Multi-Platform Deterministic Offline GitHub Release Workflow (`.github/workflows/release.yml`, `packaging/flatpak/*`, `scripts/package-windows-portable.py`, `src-tauri/tauri.conf.json`)**:
  - Replaced the Android-only workflow with a unified, strictly manual (`workflow_dispatch`) multi-platform release pipeline with 7 parallel jobs (`preflight`, `build-android`, `build-linux-appimage`, `build-linux-flatpak`, `build-windows`, `build-macos`, `publish-release`).
  - **GNU/Linux Cross-Distro AppImage (`x86_64` & `aarch64`)**: Built inside official `ubuntu:22.04` (`glibc 2.35`) with bundled GStreamer (`bundleMediaFramework: true`) for 100% offline execution across Debian 12+, Devuan, Ubuntu 22.04/24.04/26.04, Fedora, Arch, and Mint.
  - **GNU/Linux Flatpak Offline Bundle (`x86_64` & `aarch64`)**: Official Tauri v2 `org.gnome.Platform//51` manifest (`packaging/flatpak/org.xmemos.app.yaml`) with strict offline sandboxing (no `--share=network`, no `--filesystem=host`) exported via `flatpak build-bundle`.
  - **Windows 10/11 Full Offline Installer & Portable (`x64`)**: NSIS LZMA installer embedding Microsoft WebView2 Offline Installer (`webviewInstallMode.type = "offlineInstaller"`), plus a deterministic `SOURCE_DATE_EPOCH` Portable `.zip` with `portable.flag` enabling self-contained `./data/` storage.
  - **macOS 12+ (`arm64` Apple Silicon & `x64` Intel)**: Native per-architecture `.dmg` images with generated `icon.icns` and `icon.ico` assets.
  - **2026 Supply-Chain Hardening & Reproducible Builds**: Upgraded all actions to latest 2026 releases (`actions/checkout@v7`, `actions/setup-node@v7`, `actions/setup-java@v6`, `actions/upload-artifact@v7`, `actions/download-artifact@v8`, `pnpm/action-setup@v6`, `android-actions/setup-android@v4`, `gradle/actions/setup-gradle@v6` with open-source `basic` cache, `Swatinem/rust-cache@v2`), unified `SOURCE_DATE_EPOCH`, `SHA256SUMS.txt`, and `actions/attest-build-provenance@v4` SLSA attestations.
- **Native Desktop OS File Manager Save & Open Dialogs (`plugins/tauri-plugin-xmemos-storage/src/desktop_dialog.rs`, `desktop.rs`, `commands/resource.rs`, `src/lib/utils/download.ts`, `AttachmentGrid.svelte`, `ImageLightbox.svelte`, `MemoActionsSheet.svelte`, `BackupRestorePanel.svelte`, `resources/+page.svelte`)**:
  - Implemented zero-extra-crate native File Manager Save/Open dialogs on GNU/Linux (`GtkFileChooserNative` with automatic `xdg-desktop-portal` support inside Flatpak), Windows 10/11 (Win32 `comdlg32.dll` `GetSaveFileNameW`/`GetOpenFileNameW`), and macOS (Finder `osascript` argv-safe sheet).
  - Wired all static memo file downloads, image lightbox downloads, share-card PNG exports, database/bundle backup exports, and backup history re-downloads on Desktop to prompt the native File Manager to choose the save destination, with clean cancel handling.
- **Compile-Time & Runtime Adaptive Platform Separation (`vite.config.ts`, `platform.ts`, `settings/storage/+page.svelte`, `settings/account/+page.svelte`, `PinLockOverlay.svelte`, `AccountSwitcherSheet.svelte`, `settings/about/+page.svelte`)**:
  - Added `XMEMOS_TARGET_PLATFORM` (`android` | `desktop` | `auto`) compile-time injection so Android-only controls (`.nomedia` gallery toggle, `BiometricPrompt`) never appear in Desktop builds and Desktop-only window chrome/save dialogs are excluded from Android APKs.
- **Centralized, Conflict-Free Keyboard-Driven Shortcut Engine & `Ctrl+/` Cheatsheet (`shortcuts.svelte.ts`, `KeyboardShortcutsModal.svelte`, `+layout.svelte`, `MemoEditor.svelte`)**:
  - Built a single root-mounted keyboard dispatcher with strict IME (`isComposing`), input/textarea, and modal/overlay guards so shortcuts never conflict or double-fire across pages.
  - Added the responsive **Keyboard Shortcuts Cheatsheet Modal** (`KeyboardShortcutsModal.svelte`, triggered anytime with **`Ctrl+/`** / **`⌘/`** or **`?`**) with adaptive 2-column Desktop / 1-column Mobile layout and platform-aware tactile keycaps (`⌘` on macOS, `Ctrl` on GNU/Linux & Windows).
  - Implemented 1:1 Official Memos 2026 Markdown Editor shortcuts (`Ctrl/Cmd+Enter` Save, `Ctrl/Cmd+Shift+P` Write/Preview, `Ctrl/Cmd+B/I/E` Bold/Italic/Code, `Ctrl/Cmd+Shift+S` Strikethrough, `Ctrl/Cmd+Alt+1–3/0` Headings/Paragraph, `Ctrl/Cmd+Alt+C` Code Block, `Ctrl/Cmd+Shift+7/8/9` Numbered/Bulleted/Task Lists, `Tab`/`Shift+Tab` Indent/Outdent, `Esc` Blur) alongside fast global navigation (`N`/`C` New Memo, `Ctrl/Cmd+K` or `/` Search, `Ctrl/Cmd+,` Settings, `Ctrl/Cmd+\` Sidebar, `Ctrl/Cmd+Shift+\` Inspector, `G H/E/R/M/F/I/A/S` & `Alt+1–5` Go-To routes, and `Alt+←/→` History).
- **Desktop GPU Performance & Zero-Jank Sidebar Toggle (`AppShell.svelte`, `app.css`)**:
  - Eliminated synchronous per-frame layout reflow stutter when toggling the Desktop Sidebar by replacing CSS `width` transition with an instant layout switch paired with GPU-composited `transform` + `opacity` slide (`will-change-transform`) and `contain: layout style` on desktop chrome panels.
  - Removed `background-attachment: fixed` on `body` and eliminated redundant full-viewport desktop `backdrop-filter` stacking across scrolling cards to deliver smooth, consistent 60–120 FPS performance on WebKitGTK (GNU/Linux), WKWebView (macOS), and WebView2 (Windows).

## [2.5.0] - 2026-11-25

### Added & Changed
- **macOS 2026 Desktop Pull-Down Memo Options Menu (`MemoActionsSheet.svelte`, `memo-actions.svelte.ts`, `MemoCard.svelte`, `app.css`)**:
  - Replaced the bottom ActionSheet on **Desktop (`lg >= 1024px`)** with an authentic **Apple HIG macOS 2026 Liquid Glass Floating Pull-Down Menu** (`216px` width, `12px` radius, multi-layer ambient shadow, translucent backdrop blur, compact `30px` rows with left-aligned Lucide icons, and native macOS accent-blue hover/focus highlights) anchored directly to the memo card's `···` trigger button.
  - Organized actions into logical Apple HIG groups separated by crisp `1px` hairlines (**View & Edit** → **Copy & Export** → **Restore / Archive** → destructive red **Delete**).
  - Added smart viewport-aware auto-flipping (flips cleanly above the button when near the bottom edge), click-outside & scroll/resize dismissal without backdrop dimming, pull-down toggle on re-click, and full keyboard navigation (`Escape`, `ArrowUp`/`ArrowDown`, `Home`/`End`).
  - Kept **Mobile & Tablet/iPad (`< 1024px`)** 100% untouched with the native Konsta UI v5 iOS `<Actions>` bottom sheet.

## [2.4.0] - 2026-11-25

### Added & Changed
- **Frameless Integrated macOS 2026 Desktop Window Header (`tauri.conf.json`, `capabilities/desktop.json`, `lib.rs`, `DesktopWindowControls.svelte`, `platform.ts`, `AppShell.svelte`, `auth/+page.svelte`)**:
  - Replaced the generic OS window frame on **GNU/Linux (Wayland & X11), macOS, and Windows** with a seamless frameless window (`"decorations": false` + `#[cfg(desktop)]` safeguard) styled with **Konsta UI v5 iOS Theme** and **Apple HIG macOS 2026 Liquid Glass**.
  - Added least-privilege desktop-only Tauri v2 capability (`src-tauri/capabilities/desktop.json` scoped strictly to `["linux", "macOS", "windows"]`) granting `core:window:allow-start-dragging`, `allow-minimize`, `allow-toggle-maximize`, `allow-internal-toggle-maximize`, `allow-close`, and `allow-is-maximized` with zero impact on Android (`mobile.json`).
  - Built authentic macOS 2026 traffic-light controls (`DesktopWindowControls.svelte`: red Close, yellow Minimize, green Zoom/Maximize/Restore with hover glyphs and unfocused dimming) integrated directly into the `52px` Sidebar header, seamlessly migrating to the `52px` Unified Toolbar when the sidebar is collapsed (`Ctrl+\`), plus a compact `34px` Liquid Glass strip when a desktop window is resized narrow (`< 1024px`) and a floating drag bar on `/auth`.
  - Enabled reliable window dragging (`data-tauri-drag-region` + manual `mousedown` drag & double-click maximize) across GNU/Linux, macOS, and Windows while keeping Android/Mobile and plain web browsers 100% untouched.

## [2.3.11] - 2026-11-25

### Fixed
- **Isolate Map Stacking Context (`OsmMapCanvas.svelte`, `LocationPreviewPopover.svelte`)**: Added `isolate z-0` to the Leaflet map container so Leaflet's internal `z-[400]` panes and `+`/`-` zoom controls never bleed through or overlap `MemoEditor` sheets, mobile drawers, dialogs, or bottom navigation on `/map`.

## [2.3.10] - 2026-11-25

### Added & Improved
- **Mobile Map Navigation in Slide-Over Rail & Responsive Mobile Map UX (`AppShell.svelte`, `map/+page.svelte`)**:
  - Added **Map** (`/map`) cleanly to the Mobile slide-over menu drawer (`Panel`) alongside Inbox and Archived, keeping the bottom 5-tab Konsta iOS `Tabbar` uncluttered.
  - Optimized `/map` for mobile viewports (`< 768px`) with adaptive `h-[min(58dvh,460px)]` map-first stacking (`order-1`), smooth auto-scroll to the selected pin/memo detail panel (`order-2`) on tap, and compact Map-scoped `TagTree` (`order-3`) right below.

## [2.3.9] - 2026-11-25

### Fixed
- **Clean Mini-Map Popover Footer (`LocationPreviewPopover.svelte`)**: Removed the redundant blue arrow button from the mini-map preview footer so it cleanly displays only the nearest region name and exact degree coordinates matching Official Memos 2026.
- **Prevent External Link WebView Hijack & Add Back Navigation Shortcuts (`+layout.svelte`, `OsmMapCanvas.svelte`, `LocationPreviewPopover.svelte`)**: Disabled the `Leaflet` attribution prefix (keeping clean `© OpenStreetMap` attribution), intercepted all external `http(s)://` links to open safely in the OS default browser via `tauri-plugin-opener` (never trapping the Xmemos app window), and added standard `Alt+Left` / `⌘[` Back and `Alt+Right` / `⌘]` Forward shortcuts.

## [2.3.8] - 2026-11-25

### Changed
- **Full MemoCard Parity on Archived (`/archived`) (`archived/+page.svelte`, `MemoCard.svelte`, `MemoActionsSheet.svelte`)**: Upgraded Archived memos to render using the complete `MemoCard` component—preserving author header, visibility icon, `•••` options menu (with `Restore`, `Edit`, `Copy`, `Export card`, `Delete`), location mini-map preview popover, attachments, reactions, comments, and interactive task checkboxes—plus a 1-tap **Restore** pill in the card footer.

## [2.3.7] - 2026-11-25

### Changed
- **Context-Aware Desktop Tags Rail Placement (`AppShell.svelte`)**:
  - **Home (`/`) & Explore (`/explore`)**: Placed `TagTree` in the **Desktop Right Inspector Rail** beneath the pinned `ActivityHeatmap` with its own independent smooth-scrolling container (`min-h-0 flex-1 overflow-y-auto overscroll-contain`), keeping the Left Rail clean and spacious for Search, Filters, Display, and Navigation.
  - **Review (`/review`) & Map (`/map`)**: Placed `TagTree` (global on `/review`, map-scoped on `/map`) in the **Desktop Left Rail** inside its own independent smooth-scrolling pane below pinned `Workspace`/`Library` navigation.

## [2.3.6] - 2026-11-25

### Changed
- **macOS & Konsta UI iOS Primary Button Contrast (`app.css`, `AppShell.svelte`)**: Added dedicated `--xm-accent-btn` / `--xm-accent-btn-hover` tokens (`#0066cc` / `#0052a3` in Light, Apple HIG Dark Mode `systemBlue` `#0a84ff` / `#0071e3` in Dark) with subtle specular top highlight for the Desktop Toolbar **Create Memo** button and Sidebar brand badge so white labels stay crisp and never look washed out in Dark Mode.

## [2.3.5] - 2026-11-25

### Fixed
- **Independent Tags Scroll Pane in Desktop Left Rail (`AppShell.svelte`)**: Separated the Desktop Left Rail `Tags` region into its own `min-h-0 flex-1 overflow-y-auto overscroll-contain` flex pane beneath the pinned `Workspace` and `Library` navigation list, ensuring scrolling through long tag lists never scrolls away or hides navigation items on Home, Explore, or `/map`.

## [2.3.4] - 2026-11-25

### Fixed & Changed
- **Root-Mounted Display Popover (`TimelineDisplayPopover.svelte`, `feed-display.svelte.ts`)**: Lifted the Display menu popover to the root `+layout.svelte` layer (`z-[80]`) so clicking **Display** in the Desktop Left Rail escapes the sidebar's `backdrop-filter` / `overflow-hidden` containing block and opens reliably every time.
- **Desktop Active Search & Filter Pills Above Memos (`+page.svelte`, `explore/+page.svelte`)**: Moved active removable search chips (`🔍 <term> ×`), `#tag`, and date filter pills to the top of the center Memos feed on Desktop (matching Official Memos 2026), keeping the Desktop Left Rail clean with Search, Quick Filters, Display, Navigation, and Tags.

## [2.3.3] - 2026-11-25

### Fixed & Changed
- **Mobile FAB on Explore (`/explore`)**: Enabled the floating `+` Create Memo FAB on Explore for authenticated mobile users, immediately adding newly created public/protected memos to the Explore feed.
- **Desktop Left Rail Hierarchy (`AppShell.svelte`, `map/+page.svelte`)**:
  - Grouped **Search Bar**, **Filter Tabs**, active filter pills, and **Display** controls together at the **top** of the Desktop Left Rail right beneath the `Xmemos` header for instant access.
  - Moved **Tags (`TagTree`)** into the **Desktop Left Rail** on Home, Explore, and `/map` (with Map-scoped tags cleanly isolated to `/map`), matching Official Memos 2026 and leaving the Right Inspector Rail dedicated to the Monthly Activity Calendar.
- **Root-Mounted Interactive Mini-Map Popover (`LocationPreviewPopover.svelte`, `location-preview.svelte.ts`)**: Lifted the memo location mini-map preview popover to the root `+layout.svelte` layer (`z-[80]`) so it escapes `MemoCard` `backdrop-filter` / `overflow-hidden` containing blocks and opens reliably on every card across Mobile, Tablet, and Desktop.
- **Multi-Tier FOSS Nearest-Region Geocoding (`geocode.ts`, `MemoEditor.svelte`)**: Upgraded reverse geocoding to chain OpenStreetMap Nominatim (`zoom=10`) -> Komoot Photon (`photon.komoot.io/reverse`) -> Nominatim (`zoom=6`) with request deduplication, and made `MemoEditor` resolve coordinate-only labels before saving to SQLite so remote/rural pins always show real region names instead of raw coordinates.

## [2.3.2] - 2026-11-25

### Changed
- **macOS 2026 Desktop Left Rail Controls (`AppShell.svelte`, `TimelineDisplayMenu.svelte`, `+page.svelte`, `explore/+page.svelte`)**:
  - Relocated the Search bar, Quick Filter tabs (`All`, `Tasks`, `Media`, `Links`, `Code`), active removable filter pills (`🔍 term ×`, `#tag`, date), and **Display** menu cleanly into the Desktop (`lg >= 1024px`) macOS Left Sidebar Rail while keeping Mobile (`< 768px`) and Tablet/iPad (`768px–1023px`) inline controls 100% untouched.
  - Added `sidebarMode` to `TimelineDisplayMenu.svelte` with viewport-clamped `position: fixed` popover positioning so Display options open cleanly alongside the sidebar with zero scroll-container clipping.
  - Upgraded `Ctrl+K` / `⌘K` on Desktop to automatically expand the Left Sidebar if collapsed and focus the sidebar search field immediately.

## [2.3.1] - 2026-11-25

### Added
- **Nearest Region Auto-Naming & Interactive Mini-Map Location Preview Popover (`geocode.ts`, `MemoLocationPreview.svelte`, `MemoCard.svelte`, `MemoEditor.svelte`, `map/+page.svelte`)**:
  - Dropping a pin on `/map` or in `MemoEditor` (or using GPS) now automatically resolves the nearest human-readable region name (e.g. `Bandung, Jawa Barat, Indonesia`) via FOSS OpenStreetMap Nominatim reverse geocoding with persistent `localStorage` caching (`xmemos.geocode.cache.v1`) and graceful offline fallback, without overwriting custom user labels.
  - Clicking any memo's `📍 <Place Name>` row now opens an Official Memos 2026-style viewport-clamped floating **Mini-Map Preview Popover** with interactive offline-cached OpenStreetMap tiles, top-right **Open on Full Map (`↗`)**, **Zoom In (`+`)**, and **Zoom Out (`-`)** controls, exact degree coordinates (`37.995400°, -123.020400°`), and 1-tap **OsmAnd** launch.

## [2.3.0] - 2026-11-25

### Added
- **macOS 2026 Desktop Split-View & Liquid Glass Architecture (`AppShell.svelte`, `desktop-layout.svelte.ts`, `app.css`)**:
  - Upgraded Desktop viewports (`lg >= 1024px`) to follow Apple Human Interface Guidelines for macOS 2026 (Liquid Glass) harmonized with Konsta UI v5.5.0 (`theme="ios"`), while keeping Mobile (`< 768px`) and Tablet/iPad (`768px–1023px`) layouts 100% untouched.
  - **Full-Height Collapsible Leading Sidebar (`244px` / `256px`)**: Translucent Liquid Glass source list with grouped `Workspace` (Home, Explore, Review, Map, Media) and `Library` (Inbox, Archived, Settings) sections, compact `34px` macOS pill selection rows, hairline dividers, and persistent toggle (`Ctrl+\` / `⌘\`).
  - **Unified macOS Window Toolbar (`52px`)**: Leading Sidebar Toggle, contextual Back button, leading-aligned window title & subtitle, single non-redundant **New Memo** pill CTA, contextual Inspector Toggle (`Ctrl+Shift+\` / `⌘⇧\`), and Account Switcher Liquid Glass pill.
  - **Collapsible Trailing Inspector Rail (`288px` / `304px`)**: Houses the Monthly Activity Calendar and Tag Tree in an independently scrollable macOS inspector pane on timeline routes, giving instant extra horizontal room for multi-column masonry, `/review`, and `/map` when collapsed.
  - **Single-Mount Adaptive Canvas**: Keeps `{@render children()}` mounted strictly once across all breakpoints so switching or resizing viewports never double-mounts Leaflet maps, editors, or data fetches.

## [2.2.1] - 2026-11-25

### Fixed
- **Rust Backend Compile Fix for `clear_location` (`store/memo.rs`, `commands/memo.rs`)**:
  - Removed the misplaced `clear_location` field from persisted `MemoPayload` (preserving strict 1:1 Official Memos `memo.payload` JSON schema `{ tags, property, location? }`) and from `CreateMemoPayload` / `CreateMemoInput`.
  - Added `#[serde(default)] pub clear_location: bool` to `UpdateMemoInput` and `pub clear_location: Option<bool>` to `UpdateMemoPayload`, wiring it cleanly into `memo_update` so clearing a saved memo location works end-to-end and compiles cleanly under `cargo check --all-targets` and `cargo test`.

## [2.2.0] - 2026-11-25

### Added
- **Official Memos 2026 Map Page (`/map`, `OsmMapCanvas.svelte`) — 100% FOSS Offline-First OpenStreetMap + OsmAnd Integration**:
  - Built interactive map powered by **Leaflet `1.9.4`** (BSD-2-Clause FOSS) and **OpenStreetMap** tiles with automatic local `Cache` API offline tile persistence (`xmemos-osm-tiles-v1`) and offline vector grid fallback, plus 1-tap **Open in OsmAnd** (`geo:lat,lng?q=...`) launch for native turn-by-turn offline navigation.
  - Added smart proximity marker clustering (circular count badges for clusters, author-initial pins for single memos), floating bottom-left `+` / `-` / `Show all results` controls, tap-anywhere pin dropping for planning, **Write here** direct memo creation, and clickable `MapPin` location links on `MemoCard`.
  - **Map-Scoped Tags Sidebar**: Displays tags exclusively from memos with saved locations on `/map`, while keeping global mixed tags on Home and Explore.
- **Add / Edit Memo Location Picker (`MemoEditor.svelte`, `store/memo.rs`)**:
  - Added `MapPin` toolbar toggle with interactive mini-map pin picker, custom location label, manual latitude/longitude inputs, optional device GPS lookup, and full location clearing (`clear_location`) stored 1:1 in Official Memos `memo.payload.location`.
- **Official Memos Tags "Show as tree" Toggle (`TagTree.svelte`, `tagViewState`)**:
  - Added `•••` options popover on every Tags card with persistent **Show as tree** toggle switching cleanly between flat slash-separated tags (`# travel / hikes`) and a collapsible hierarchical tree across Home, Explore, and Map.

## [2.1.25] - 2026-11-25

### Added
- **Official Memos 2026 Search Filter Chips & Keyword Highlighting (`filters.svelte.ts`, `markdown.ts`, `MemoContent.svelte`, `search.rs`)**:
  - Pressing **Enter** (or tapping the `↵` submit button) in the Search bar on Home (`/`) or Explore (`/explore`) commits the query into a removable **`🔍 <term> ×`** filter chip with `"Remove filter"` tooltip, alongside `Ctrl+K` / `Cmd+K` quick focus and real-time debounced preview.
  - Added HTML-tag-aware `highlightHtmlMatches()` rendering warm amber `<mark class="xm-search-mark">` highlights on matching substrings inside memos without ever mutating HTML tags, `href` attributes, SVG icons, or KaTeX `<math>` formulas (covered by Vitest unit tests).
  - Upgraded Rust `search_memo_ids` to combine FTS5 `bm25` prefix ranking with parameterized case-insensitive substring matching (`INSTR(LOWER(content), LOWER(?)) > 0`) so partial and mid-word matches work identically to Official Memos.

## [2.1.24] - 2026-11-25

### Fixed
- **Native Image Download in Fullscreen Preview (`resource_export_to_downloads`, `ImageLightbox.svelte`)**: Replaced broken `<a href="xmemos-media://..." download>` navigation with a native Rust Tauri command (`resource_export_to_downloads`) that enforces SQLite visibility permissions, sanitizes and deduplicates filenames, writes full original bytes straight into the OS Downloads folder, and shows localized confirmation toasts (with safe Blob-fetch fallback for unsaved local previews).

## [2.1.23] - 2026-11-25

### Added
- **Full Search Bar & Quick Filter Tabs on Explore (`/explore`)**: Added 1:1 Home-parity FTS5 search bar (120ms debounce + clear button), quick filter chips (`All`, `Tasks`, `Media`, `Links`, `Code`), and direct in-page `#tag` and calendar date filtering on Explore, strictly scoped in SQLite to `PUBLIC` (Guest) or `PUBLIC` + `PROTECTED` (authenticated) with race-free `requestSeq` guards.

## [2.1.22] - 2026-11-25

### Fixed
- **Container-Query Aware MemoCard Header (`MemoCard.svelte`)**: Enabled CSS `@container` on memo cards so author names and long timestamps (`Nov 28, 2026, 11:10 AM`) automatically stack cleanly on two lines in narrow multi-column cards (`< 340px`) and stay inline on wider cards, permanently eliminating text/icon overlap with the visibility icon and `•••` menu.
- **ResizeObserver Smart Column Width (`MemoMasonryGrid.svelte`)**: Measures actual container width so multi-column modes (`2`, `3`, `Auto`) maintain comfortable card widths (`>= 285px`) across every window size.
- **Dynamic Scroll-Aware `Back to Top ↑` (`InfiniteScrollSentinel.svelte`)**: Evaluates scroll container overflow (`scrollHeight > clientHeight + 120`) and active scroll offset (`scrollTop > 120`) so `Back to Top ↑` only appears when the feed actually overflows the screen and the user has scrolled down.

## [2.1.21] - 2026-11-25

### Added
- **Official Memos 2026 Fullscreen Image Lightbox (`ImageLightbox.svelte`, `lightboxState`)**:
  - Upgraded image preview with complete interactive controls matching the Official Memos demo: top-bar filename, multi-image counter, **Info (`ⓘ`)** metadata popover (filename, natural dimensions `W × H px`, file size, zoom level), **Download (`↓`)**, and **Close (`×`)**; plus a frosted floating bottom pill with **Zoom Out (`25%`)**, clickable **Zoom `%` Reset**, **Zoom In (`400%`)**, **Rotate 90° (`↻`)**, **Reset (`↺`)**, mouse-wheel zoom, double-click zoom, drag-to-pan, and `←`/`→` gallery navigation.

### Fixed
- **Independent 3-Column Scroll Panes (`AppShell.svelte`, `InfiniteScrollSentinel.svelte`)**: Locked the desktop/tablet shell to `h-dvh` with three independent vertical scroll containers (Left Rail, Center Feed `#main-content`, and Right Rail) so scrolling memos never affects the Left or Right rails and never bleeds behind the top navbar.
- **Flexbox Masonry Grid (`MemoMasonryGrid.svelte`) & Permanent Right Rail**: Replaced CSS `columns` (which caused WebKit card fragmentation, staggered holes, and clipped `Back to Top ↑`) with a true round-robin Flexbox column distributor (`1` col on mobile `< 768px`, `2`/`3`/`Auto` on tablet/desktop), kept the Right Rail permanently visible on Home and Explore, and bounded the Home search bar (`sm:max-w-[540px]`) so it never stretches excessively on wide screens.

## [2.1.20] - 2026-11-25

### Added
- **Official Memos 2026 Timeline Display & SQLite Sorting (`TimelineDisplayMenu.svelte`, `feedDisplayState`)**:
  - Added Konsta iOS **Display** popover on Home (`/`) and Explore (`/explore`) matching the Official Memos demo:
    - **Desktop/Tablet Column Layouts (`md+` only)**: `List` (1-col), `2` columns, `3` columns, and `Auto` (2/3/4 adaptive masonry columns via CSS `columns` + `break-inside-avoid`), plus **Full width** toggle (`Use entire width of the memo timeline`). Mobile screens (`< 768px`) automatically stay on a clean, fast 1-column list and hide desktop-only column/full-width controls.
    - **Real SQLite `Order by` & `Direction` (All Viewports)**: Whitelisted backend SQLite `ORDER BY` supporting `Created at` (`created_ts`) or `Last updated at` (`updated_ts`) in `Newest first` (`DESC`) or `Oldest first` (`ASC`) order across Home and Explore.

### Fixed
- **Viewport-Locked Big Calendar on Tablet & Desktop (`FullMonthCalendar.svelte`, `/review`)**: Locked the calendar workspace to `md:h-[calc(100dvh-5.25rem)]` with dynamic `grid-template-rows: repeat(var(--xm-cal-weeks), minmax(0, 1fr))` and aligned the Day Detail Panel cleanly beside it (with internal-only scrolling) so tablet and desktop viewports never trigger an unwanted outer vertical scrollbar.

## [2.1.19] - 2026-11-25

### Fixed
- **Calendar Day Panel Deduplication & Visual Polish**: Removed the duplicate `+ Create Memo` button from the Day Detail Panel header next to `×` on `/review`, keeping a single clean **New memo on this day** action (centered in the empty-day state, or docked cleanly in the panel footer when memos exist).
- **Konsta UI iOS Grid Alignment & Rhythm**: Aligned the sticky desktop Day Detail Panel top edge with the monthly calendar grid, enforced uniform desktop week row heights (`md:auto-rows-fr`), softened the selected-day iOS accent tint, and refined mobile spacing and full-width segmented switcher ergonomics.

## [2.1.18] - 2026-11-25

### Added
- **Official Memos 2026 Full-Page Monthly Calendar (`FullMonthCalendar.svelte`)**:
  - Upgraded `/review` with a full-width Official Memos 26.10/26.11 style monthly calendar grid styled with Konsta UI iOS Theme (`glass-surface`, iOS segmented mode switcher between **Calendar** and **Random**, crisp 1px grid borders, and high-contrast Today/Selected pills).
  - **Interactive Month & Year Picker (`Month Year ↕`)**: Lists active years containing memos, marks months that hold memos with live counts, and supports keyboard navigation (`ArrowLeft`/`ArrowRight`, `Escape`).
  - **Rich Day Cells with Responsive Adaptation**: Desktop and tablet cells display up to 3 plain-text memo excerpts with author avatar initials, `+N more` overflow counter, and lazy-loaded local image attachment thumbnails (`getMediaUrl(uid, true)`); mobile cells adapt cleanly into tap-friendly count pills, accent dots, and photo indicators without crowding.
  - **Responsive Day Detail Panel**: Docks as a sticky right-hand side panel on desktop (`lg+`) and smoothly below the grid on mobile/tablet (`< lg`), displaying the localized weekday/date heading, full interactive `<MemoCard>` actions, `"No memos on this day"` empty state, and 1-tap **New memo on this day** creation.

## [2.1.17] - 2026-11-25

### Added
- **Official Memos Custom Timestamp & "New Memo on This Day" Parity**:
  - Upgraded `MemoEditor.svelte` with an explicit **Now (Auto)** vs. **Set Date & Time** custom timestamp flow (supporting arbitrary past and future dates with live `Past` / `Future` badges and 1-tap `Reset to Now` / `Now` controls, eliminating confusing empty WebKit `datetime-local` placeholders).
  - Added **New memo on this day** action on `/review` when any past or future calendar date is selected, and automatic date pre-fill on Home (`/`) when composing under an active calendar date filter (combining the selected `YYYY-MM-DD` date with the current local clock time, matching Official Memos 26.10/26.11).
  - Added Vitest unit coverage in `src/lib/utils/utils.test.ts` for arbitrary past/future local timestamp round-trips and calendar-date clock synthesis.

### Fixed
- **All-Time Calendar Heatmap (`memo_get_heatmap`)**: Removed the 365-day lookback cutoff when `days = 0` (default) so memos backdated to previous years or scheduled into future dates always populate the monthly calendar heatmap.
- **Chronological Sorting on Create & Edit**: Updated `memosState.createMemo` and `memosState.replaceItem` to sort by `pinned DESC, createdTs DESC, id DESC` (1:1 parity with SQLite) and honor active date filters, and aligned Rust `create_memo` to initialize `updated_ts = created_ts` when a custom creation timestamp is provided.

## [2.1.16] - 2026-11-25

### Fixed
- **Full Memo Options on Explore, Review & Profile Pages**: Wired `Edit`, `Pin/Unpin`, `Archive`, and `Delete` handlers plus lazy-mounted `<MemoEditor>` across Explore (`/explore`), Review (`/review`), and User Profile (`/u/[username]`) so memos created by the currently active account always expose the complete Memo Options menu consistently with Home.
- **Strict Active-Creator Ownership Gate**: Tightened `MemoCard` management visibility (`canManage`) strictly to `authState.currentUser !== null && authState.currentUser.id === view.memo.creatorId` so Guest Mode and memos created by other accounts only see read-only actions (`Open Details & Comments`, `Copy Markdown`, `Copy Plain Text`, `Export Share Card`).
- **Reactive Account Switching on Review**: Replaced one-shot `onMount` fetch on `/review` with a race-guarded `$effect` bound to `authState.currentUser?.id` so switching accounts or entering Guest Mode immediately refreshes the Review heatmap and memo list.

## [2.1.15] - 2026-11-25

### Added & Improved
- **Smart Multi-Language Code Block Engine (`src/lib/utils/code-highlight.ts`, `src/lib/utils/markdown.ts`, `src/lib/components/memo/MemoContent.svelte`, `src/app.css`)**:
  - Bundled 28 offline PrismJS grammars (`bash`, `python`, `typescript`, `jsx`, `tsx`, `rust`, `go`, `c`, `cpp`, `csharp`, `java`, `kotlin`, `swift`, `dart`, `sql`, `json`, `yaml`, `toml`, `docker`, `diff`, `php`, `ruby`, `scss`, `powershell`, `shell-session`, `ini`, `markdown`, `markup-templating`) with modern CLI tool highlighting (`git`, `pnpm`, `cargo`, `docker`, `kubectl`, etc.).
  - Added fast heuristic auto-language detection (`detectCodeLanguage`) for unlabeled fenced code blocks plus universal comment highlighting (`#`, `//`, `/* */`, `--`, `<!-- -->`) for plain/unknown languages.
  - Isolated `#tag` and KaTeX `$math$` parsing strictly outside `<pre>` and `<code>` blocks so `#comments` and `$variables` inside code blocks are never corrupted.
  - Matched Official Memos Demo Light (`#efefe9`) and Dark (`#222326`) code block surfaces, inline code pills, top-left language label, one-tap `Copy` button, and WCAG AA+ GitHub-style syntax palettes, verified by automated Vitest unit tests.

## [2.1.14] - 2026-11-25

### Security & UX
- **Dynamic Guest Mode Settings Lockdown (`src-tauri/src/commands/system.rs`, `src/routes/settings/+page.svelte`, `src/routes/settings/account/+page.svelte`, `src/routes/settings/preferences/+page.svelte`, `src/routes/settings/storage/+page.svelte`)**:
  - **My Account**: Dynamically hides the Profile Details form (nickname, email, bio, password) and Local App Lock controls (PIN, biometric, switch lock, sign-out password toggle) when browsing as Guest, displaying a clean read-only Guest Mode card with a `Sign In` button.
  - **Preferences**: Locks and disables the `Confirm Before Deleting` toggle in Guest mode while keeping safe display preferences (Language, Appearance, Liquid Glass, Density) accessible.
  - **Storage & Backup**: Hides the `Storage, Backup & 1:1 Import` entry in the Settings hub and locks direct navigation to `/settings/storage` behind a `Sign In Required` guard when unauthenticated.
  - **Rust Backend Enforcement**: Guarded `system_set_local_pref` with `require_authenticated_user(&state)?` for `confirm_before_delete` and `require_password_after_sign_out` so Guest sessions cannot mutate security/safety preferences via IPC.

## [2.1.13] - 2026-11-25

### Changed & Improved
- **Official Memos 26.11.2 Pin Indicator & Card Header (`src/lib/components/memo/MemoCard.svelte`)**:
  - Replaced the inline "Pinned" badge with the authentic Official Memos 26.11 top-right golden-amber corner-fold triangle (`clip-path: polygon(100% 0, 0 0, 100% 100%)`).
  - Refined the card header to match Official Memos (`usememos.com` / `demo.usememos.com/explore`): compact single-line `Avatar + Author · Relative Time` on the left and subtle Visibility icon (`Globe` / `Users` / `Lock`) beside `•••` on the right, plus clean `MapPin` location and `Link2` related-memo metadata rows when present.
- **Interactive `Back to Top ↑` Button (`src/lib/components/ui/InfiniteScrollSentinel.svelte`, `src/lib/i18n/locales/{en,id,es}.ts`)**:
  - Replaced static end-of-feed text with an interactive `Back to Top ↑` button (localized in `en`, `id`, `es`) that smoothly scrolls both `window` and Konsta `.k-page` back to the top once all memos/media are loaded.
- **Calendar `Today` Quick-Jump Button (`src/lib/components/calendar/ActivityHeatmap.svelte`)**:
  - Added a localized `Today` quick-jump button beside the `<` `>` month arrows whenever browsing another month.

## [2.1.12] - 2026-11-25

### Improved
- **Vertically Resizable Memo Textarea with Pinned Toolbar & Footer (`src/lib/components/memo/MemoEditor.svelte`)**: Enabled smooth vertical drag resizing (`resize-y`, bounded between `150px` and `55dvh` inside the scrollable flex body) so users can comfortably expand the memo input while keeping the header, `Write / Preview` tabs, Markdown formatting toolbar, and `Save Memo` footer permanently pinned and visible.

## [2.1.11] - 2026-11-25

### Fixed
- **Memo Editor Long-Text Overflow & Clipped Formatting Toolbar (`src/lib/components/memo/MemoEditor.svelte`)**:
  - Wrapped the sheet interior in an explicit `flex max-h-[90dvh] flex-col overflow-hidden` container so header, scrollable body, and footer stay strictly bounded across all viewports.
  - Pinned the 10-action Markdown formatting toolbar (`Bold` .. `Attach`) inside the fixed header when in `Write` mode so it never clips or scrolls under the `Write / Preview` bar.
  - Constrained `<textarea>` height (`min-h-[150px] max-h-[40dvh] resize-none overflow-y-auto`) so long pasted memos scroll cleanly inside the input without pushing the character counter, tag suggestions, or attachment preview strip behind the footer, and localized remaining visibility/button labels.

## [2.1.10] - 2026-11-25

### Fixed
- **Root-Mounted Memo Actions Sheet (`src/lib/state/memo-actions.svelte.ts`, `src/lib/components/memo/MemoActionsSheet.svelte`, `src/lib/components/memo/MemoCard.svelte`, `src/routes/+layout.svelte`, `src/app.css`)**:
  - Removed `content-visibility: auto` containment from `.xm-memo-card` (which created a CSS layout/paint containing block trapping `position: fixed` descendants inside individual cards).
  - Extracted Konsta `<Actions>` out of `MemoCard.svelte` into a single root-mounted `<MemoActionsSheet />` inside `src/routes/+layout.svelte` (`z-[60]`), guaranteeing the iOS Action Sheet always floats cleanly at the bottom of the viewport above the Navbar & BottomTabbar with zero clipping, duplication, or stacking-context conflicts.

## [2.1.9] - 2026-11-25

### Added & Improved
- **Smart Infinite Scroll Pagination (`src/lib/components/ui/InfiniteScrollSentinel.svelte`, `src/lib/state/memos.svelte.ts`)**:
  - Replaced manual "Load More" pagination with `IntersectionObserver`-driven infinite scroll (`rootMargin: '320px 0px'` prefetching, concurrency lock, ID deduplication, and race-free request sequencing) across **Home**, **Explore**, **Archived** (`PAGE_SIZE = 20`), and **Resources** (24-item progressive media windowing).
- **Konsta UI iOS 2026 Adaptive Shimmer Skeletons (`src/lib/components/ui/KonstaSkeleton.svelte`, `src/app.css`)**:
  - Added GPU-composited `@keyframes xm-shimmer` skeleton placeholders (`memo` and `media` variants matching exact card geometry for zero CLS) that appear strictly during real in-flight async loads with zero artificial delay and automatically respect `prefers-reduced-motion`.
- **Deep Lazy Loading & Offscreen Rendering (`src/routes/+page.svelte`, `src/lib/components/media/AttachmentGrid.svelte`, `src/app.css`)**:
  - Lazy-mounted `MemoEditor` conditionally (`{#if editorOpened}`), set `preload="none"` on `<video>`/`<audio>` and `loading="lazy" decoding="async" fetchpriority="low"` on image thumbnails, and applied CSS `content-visibility: auto; contain-intrinsic-size: auto 180px` on `.xm-memo-card` so long infinite-scroll timelines stay fluid at 60/120fps.

## [2.1.8] - 2026-11-25

### Changed & Improved
- **Official Memos 2026 Monthly Activity Calendar (`src/lib/components/calendar/ActivityHeatmap.svelte`, `src/lib/i18n/locales/{en,id,es}.ts`)**:
  - Replaced the unnumbered dot matrix with the Official Memos 2026 (`usememos.com` v26.11.2) Monthly Calendar Grid showing localized month/year header (click to jump back to current month), `<` / `>` month navigation arrows, localized 7-day weekday header (`S M T W T F S`), and crisp numbered day tiles (`1..31` plus muted adjacent-month overflow days).
  - Applied stepped 4-level iOS blue activity intensity fills, distinct Today highlight, active selected-date ring, and full click-to-filter interactivity on every day (including 0-memo days, aligned with `usememos/memos#5415`) across Desktop Right Rail and Daily Review.

## [2.1.7] - 2026-11-25

### Fixed & Improved
- **Zero-Flash Startup (Eliminated Gray → White → Black Flash) (`src-tauri/tauri.conf.json`, `src-tauri/src/lib.rs`, `src-tauri/src/commands/system.rs`, `src/app.html`, `src/routes/+layout.svelte`, `src/lib/state/theme.svelte.ts`)**:
  - Configured the main Tauri v2 window with `"visible": false` and `"backgroundColor": "#000000"` per official Tauri v2 WindowConfig documentation (`v2.tauri.app/reference/config/`), keeping the native OS window hidden during WebView initialization and eliminating both the unpainted gray frame and WebView white flash.
  - Synchronized the native window/WebView background color (`#000000` dark / `#f2f2f7` light) in Rust `setup` from `xmemos_local.db` (`local_pref.theme_mode`) and live on theme toggles via `system_set_window_theme_bg`.
  - Revealed and focused the main window via `app_show_main_window` strictly after double-`requestAnimationFrame` first paint in `+layout.svelte` (with a 1200ms Rust failsafe reveal so the window can never remain hidden).
  - Executed theme detection before inline critical CSS covering `html`, `body`, and Konsta `.k-ios-page` in `src/app.html`, and removed full-black opacity-0 cold-start fade on `.xm-page-enter`.

## [2.1.6] - 2026-11-25

### Performance & Architecture
- **Instant Cold/Warm Startup & Fast Shutdown (`src-tauri/src/state.rs`, `src-tauri/src/lib.rs`, `src-tauri/src/service/search.rs`, `src-tauri/src/store/db/mod.rs`, `src-tauri/Cargo.toml`)**:
  - Removed synchronous full FTS5 rebuild and `temp/` directory sweep from `AppState::initialize()` so Tauri launches in O(1) time; offloaded both to a background startup thread running an O(1) row-count parity check (`ensure_fts_index_ready`) that rebuilds FTS5 only if out of sync.
  - Enabled 64 MiB zero-copy SQLite memory-mapped I/O (`PRAGMA mmap_size = 67108864`) and bounded WAL checkpoints (`PRAGMA wal_autocheckpoint = 1000`) per official SQLite and Android performance guidelines.
  - Added non-blocking `RunEvent::Exit` fast shutdown (`prepare_fast_shutdown` via `try_lock` + `PRAGMA wal_checkpoint(PASSIVE); PRAGMA optimize;`) so closing the app is instantaneous with zero WAL lock hangs.
  - Tuned `[profile.release]` to `opt-level = 3`, `lto = true`, `codegen-units = 1`, `panic = "abort"`, and `strip = true` per official Tauri v2 performance guidance.
- **Zero-Flash Frame-1 Hydration & Parallel Feed Loading (`src/app.html`, `src/routes/+layout.svelte`, `src/routes/+page.svelte`, `src/lib/state/auth.svelte.ts`, `src/lib/state/memos.svelte.ts`, `src/lib/utils/markdown.ts`)**:
  - Inlined critical Light/Dark iOS canvas styles in `src/app.html` to eliminate white launch flash on frame 0.
  - Added synchronous frame-0 auth & per-user timeline snapshot hydration (strictly wiped on sign-out or account switch) so the timeline renders immediately on launch while SQLite revalidates in parallel (`Promise.all([loadTimeline, loadSidebarMetadata])`), backed by iOS shimmer skeletons on fresh installs.
  - Deferred non-critical `local_pref` DB reconciliation (`theme`, `i18n`, `confirmDelete`) via `requestIdleCallback` so the Tauri IPC bridge prioritizes auth and timeline queries on startup.
  - Added fast-path KaTeX math skip and a bounded 250-entry LRU cache to `renderMemoMarkdown` for instant 60/120fps scrolling.

## [2.1.5] - 2026-11-25

### Added & Changed
- **Confirm New Password Field in Profile Details (`src/routes/settings/account/+page.svelte`)**: Added a dedicated `Confirm New Password` (`Re-type new password`) input beneath `New Password` with live inline validation (minimum 4 characters and exact password match verification before saving, clearing both inputs after update).
- **Require Password After Sign Out Toggle (`src-tauri/src/service/auth.rs`, `src-tauri/src/commands/auth.rs`, `src/routes/settings/account/+page.svelte`, `src/lib/components/auth/AccountSwitcherSheet.svelte`)**:
  - Added a device-local `Require Password After Sign Out` toggle persisted in `xmemos_local.db` (`local_pref` key `require_password_after_sign_out`, keeping `memos_prod.db` 1:1 untouched).
  - Enforced strict bcrypt password verification in Rust (`auth_switch_account`) whenever the active session is signed out (`current_user_id == None`) and the toggle is enabled — blocking passwordless or PIN-only bypasses.
  - Updated `AccountSwitcherSheet` to immediately prompt for the target account's password when signing back in from Guest mode while the toggle is active.
- **i18n Parity (`src/lib/i18n/locales/{en,id,es}.ts`)**: Added full English, Bahasa Indonesia, and Español translations for all new password confirmation and sign-out security controls.

## [2.1.4] - 2026-11-25

### Fixed & Changed
- **Permanent Single-Checkbox Guarantee (`src/lib/utils/markdown.ts`, `src/app.css`, `src/lib/utils/utils.test.ts`)**: Used `this.parser.parse(...)` with `checkbox` token filtering (preserving nested task checkboxes) plus a CSS safety rule hiding any non-`.xm-task-checkbox` inputs inside `.memo-prose`, verified by automated Vitest unit tests for both flat and nested task lists.
- **100% Invariant Desktop & Tablet Left Navigation Rail (`src/lib/components/layout/AppShell.svelte`)**:
  - Locked the Left Rail width (`228px` md / `240px` lg / `248px` xl) and top vertical position across EVERY route so navigating between `Home`, `Explore`, `Review`, `Media`, `Settings`, `Inbox`, and `Archived` causes zero horizontal or vertical layout shift.
  - Placed the persistent `Create Memo` CTA cleanly below the Navigation Card on every authenticated page (with built-in fallback `MemoEditor` so creating a memo works from any screen on Tablet/Desktop).
  - Removed `Tags` from the Left Rail on non-feed Desktop pages (`Settings`, `Resources`, `Daily Review`), hid the redundant Desktop top-left Back button on top-level sidebar routes (`Inbox`, `Archived`), and refined `Daily Review` and `Resources` maintenance proportions on wide viewports.

## [2.1.3] - 2026-11-25

### Fixed & Changed
- **Memo Checklist Double-Checkbox Bug (`src/lib/utils/markdown.ts`, `src/app.css`, `src/lib/utils/utils.test.ts`)**: Overrode `renderer.checkbox = () => ''` in `marked` v18 so GFM task lists emit exactly one interactive checkbox per item (verified by automated regression test), styled as a clean native iOS 2026 rounded checkbox with SVG checkmark and completed-item strikethrough.
- **Official Memos 2026 Desktop & Tablet Layout Alignment (`src/lib/components/layout/AppShell.svelte`, `src/lib/components/calendar/ActivityHeatmap.svelte`)**:
  - Aligned the 3-column Desktop layout (`lg+`) with Official Memos 2026 structure: clean sticky Left Navigation Rail (`Create Memo` CTA + unified navigation list), crisp `max-w-[680px]` centered Timeline Feed, and sticky Right Insights Rail combining `ActivityHeatmap` (with today's cell outlined) + `TagTree`.
  - Kept Tablet/iPad (`md`) cleanly in 2-column split (navigation + `TagTree` on the left rail) and Phone (`< md`) in native Konsta UI iOS single-column + bottom `Tabbar` + FAB.

## [2.1.2] - 2026-11-25

### Changed
- **Smart Multi-Viewport Responsive Layout for Tablet/iPad & Desktop (`src/lib/components/layout/AppShell.svelte`, `src/app.css`)**:
  - Upgraded left navigation sidebar and right Activity Heatmap rail to sticky independent-scroll panels (`md:sticky md:top-16`) with subtle `.xm-sidebar-scroll` styling so navigation and insights stay accessible on long timelines.
  - Added context-aware right-rail toggling (automatically hidden on `Settings`, `Daily Review`, `Resources`, and `Profile` routes to eliminate duplicate heatmaps and wasted columns) with clean reading/working max-widths (`max-w-3xl` for feeds/forms, `max-w-5xl` for media grids).
  - Placed a native iPadOS/Desktop `Create Memo` CTA button at the top of the sticky left sidebar (`md+`) while keeping the floating FAB exclusively on mobile (`md:hidden`).
- **Adaptive Media Density (`src/routes/resources/+page.svelte`, `src/lib/components/media/AttachmentGrid.svelte`)**:
  - Expanded Resources grid cleanly across breakpoints (`2 → 3 → 4 → 5` columns from phone to desktop) and capped single-image attachment height (`max-h-[380px] lg:max-h-[420px]`) for balanced reading on large displays.

## [2.1.1] - 2026-11-25

### Fixed
- **Rust Build Fix (`E0061` in `src-tauri/src/service/backup.rs`)**: Passed `&BackupScope::AllUsers` to `create_clean_db_snapshot(state, &rollback_path, &BackupScope::AllUsers)?` inside `execute_atomic_import` so automatic pre-import rollback snapshots always capture the full workspace database cleanly.

## [2.1.0] - 2026-11-25

### Added
- **Modular Strict i18n Multi-Language Architecture (`en`, `id`, `es`)**:
  - Created isolated, zero-hardcode i18n module under `src/lib/i18n/` (`types.ts`, `index.svelte.ts`, `locales/en.ts`, `locales/id.ts`, `locales/es.ts`, and `i18n.test.ts`).
  - Full compile-time TypeScript key enforcement (`Record<TranslationKey, string>`) plus automated Vitest checks guaranteeing 100% key parity, non-empty translations, and identical `{param}` interpolation placeholders across **English**, **Bahasa Indonesia**, and **Español**.
  - Zero-lag reactive Svelte 5 Runes switching via `Settings → Preferences → Language`, pre-paint `<html lang>` synchronization in `src/app.html`, `Intl` locale-aware relative/full timestamps in `src/lib/utils/date.ts`, and dual persistence (`localStorage` + `xmemos_local.db` `local_pref`).
  - Wired all 29 routes and components across navigation, timeline, memo cards/editor, review/heatmap, resources/orphan cleanup, inbox, archive, auth/lock, account switcher, profile, settings, and scoped backup/restore.

## [2.0.9] - 2026-11-25

### Changed & Fixed
- **Compact Konsta UI iOS 2026 Top-Right User Control (`src/lib/components/layout/AppShell.svelte`)**: Replaced the oversized custom `<button>` in the Navbar right slot with native Konsta `<Link navbar>` pairing a crisp 24px circular initial badge with a compact `@username` label so it matches the left menu button proportions seamlessly.
- **Official Memos RBAC Scoped Backup (`src-tauri/src/service/backup.rs`, `src-tauri/src/commands/backup.rs`, `src/lib/components/backup/BackupRestorePanel.svelte`)**:
  - Added `BackupScope` (`ALL_USERS` vs `ACTIVE_USER`) for both Mode 1 (`.db`) and Mode 2 (`.xmemos`) exports.
  - Strictly restricted `ALL_USERS` full workspace export and Safe Atomic Workspace Restore to `HOST` and `ADMIN` accounts (enforced both in Rust IPC commands and UI), while regular `USER` accounts export a standalone 1:1 Official Memos SQLite database and matching `assets/` media scoped strictly to their own active account.
  - Added clear, user-friendly Backup Scope selector and explanations in `Settings → Storage & Backup`.

## [2.0.8] - 2026-11-25

### Fixed & Changed
- **Root-Level Fullscreen Image Lightbox (`src/lib/state/lightbox.svelte.ts`, `src/lib/components/media/ImageLightbox.svelte`)**: Moved the image preview viewer out of individual cards/sheets (where ancestor `backdrop-filter` trapped `position: fixed` and clipped/misaligned previews on mobile and desktop) into a single root-mounted viewport viewer (`100vw × 100dvh`, `z-[90]`) with safe-area header bar, responsive centered `object-contain` stage, body scroll lock, and Escape/backdrop dismiss. Migrated `AttachmentGrid`, `AttachmentPreviewStrip`, and `Resources` to use this unified viewer.
- **Native Konsta UI iOS Resources Maintenance (`src/routes/resources/+page.svelte`, `src/routes/settings/storage/+page.svelte`)**: Removed the top orphan banner card and sparkle button; moved `Clean Orphan Files` to a deliberate bottom Konsta UI inset `List` section (`Storage Maintenance`) and into `Settings → Storage & Backup`.
- **Mandatory Orphan Cleanup Confirmation (`src/lib/state/confirm-delete.svelte.ts`)**: Added `forceConfirm: true` so `Clean Orphan Files` always opens the dynamic breakdown confirmation dialog regardless of the Settings `Confirm Before Deleting` ON/OFF toggle.

## [2.0.7] - 2026-11-25

### Changed & Fixed
- **Resources Media Library UI/UX (`src/routes/resources/+page.svelte`)**: Upgraded to a clean iOS 2026 layout featuring a compact storage summary bar (total files, total size, live orphan indicator, and tonal `Clean Orphans` action), Title-Case filter pills (`All`, `Photos`, `Videos`, `Audio`, `Files`) with counts, and `Unlinked` badges on attachments no longer tied to any memo.
- **Full Logical Orphan Scan & Cleanup (`src-tauri/src/service/media.rs`, `src-tauri/src/commands/resource.rs`)**: Added `resource_scan_orphans` and upgraded `resource_clean_orphans` to detect and remove unlinked database attachments (`memo_id IS NULL`), unreferenced disk files in `assets/` (with canonicalized paths), and orphaned thumbnails in `thumbs/`.
- **Dynamic Orphan Confirmation Dialog (`src/lib/state/confirm-delete.svelte.ts`, `src/routes/+layout.svelte`)**: `Clean Orphans` now scans first, notifies cleanly when storage is already clean, and respects the Settings `Confirm Before Deleting` ON/OFF toggle—displaying a structured dynamic breakdown (unlinked attachments, unreferenced disk files, reclaimable space, and sample filenames) before deletion when enabled.
- **Toast Positioning (`src/routes/+layout.svelte`)**: Repositioned `<Toast>` cleanly above the floating mobile bottom `Tabbar` (`!bottom-[calc(env(safe-area-inset-bottom,0px)+5.5rem)] md:!bottom-6 z-50`) so notifications never overlap bottom navigation controls.

## [2.0.6] - 2026-11-25

### Fixed
- **Critical Overlay Containing-Block Bug (`Memo Options` Visible on Load)**: Removed `transform` and `animation-fill-mode: both` from `.xm-page-enter` in `src/app.css` (switching to lightweight opacity-only fade-in) so `<main>` never creates a containing block that traps `position: fixed` overlays (`<Actions>`, `<Sheet>`, `<Dialog>`).
- **Lazy-Mounted Memo Actions**: `MemoCard.svelte` now lazy-mounts Konsta `<Actions>` only on demand and unmounts after close, eliminating hidden action-sheet DOM nodes on initial page render and reducing memory footprint on long timelines.
- **Switch Local Account Sheet (`AccountSwitcherSheet.svelte`)**: Removed Konsta `outline` blue-border conflict on the destructive `Sign Out` button, added native iOS grabber handle and avatar initials without misleading navigation chevrons (`chevron={false}`), and balanced `Add Account` / `Sign Out` in a clean 2-column iOS 2026 tonal button row.
- **Whole-App Consistency & Full Interactivity**:
  - Moved `<MemoEditor>` sheets outside `<AppShell>` in `src/routes/memo/[uid]/+page.svelte` and wired full pin, archive, edit, delete, task, reaction, and tag handlers on main memos and comments.
  - Wired reaction, checklist task, and `#tag` filter handlers across `src/routes/review/+page.svelte`, `src/routes/explore/+page.svelte`, and `src/routes/u/[username]/+page.svelte`, and hid reaction picker when no handler is bound so zero dead/gimmick controls exist.
  - Aligned Lock Settings buttons (`Save Lock Settings` / tonal destructive `Clear PIN`) in `src/routes/settings/account/+page.svelte` and iOS 2026 Restore Dialog buttons in `src/lib/components/backup/BackupRestorePanel.svelte`.

## [2.0.5] - 2026-11-25

### Fixed
- **100% Accurate Device Local Time & Timezone Synchronization:** Replaced UTC `toISOString()` / `T00:00:00Z` / `DateTime<Utc>` bucketing with device-local timezone helpers (`toLocalDateKey`, `formatLocalDateTimeInput`, `parseLocalDateTimeInput`, `getLocalDayBounds` in `src/lib/utils/date.ts`) across `MemoEditor.svelte`, `ActivityHeatmap.svelte`, `review/+page.svelte`, `filters.svelte.ts`, and Rust `memo_get_heatmap` (`chrono::Local`), plus unit test coverage in `utils.test.ts`.
- **Smooth Persistent Native iOS Bottom Tabbar & Page Transitions:** Hoisted the mobile `<Tabbar>` + `<ToolbarPane>` into a root-mounted `BottomTabbar.svelte` inside `+layout.svelte` so switching tabs never unmounts or resets the sliding glass thumb back to Home, added tap-active-tab smooth scroll-to-top, and added a lightweight GPU-composited (`opacity` + `translate3d`, 160ms) `.xm-page-enter` transition on `<main>`.

## [2.0.4] - 2026-11-25

### Fixed
- **Authentic iOS 26 Delete Confirmation Dialog (`+layout.svelte`):** Rendered dialog description in Konsta UI v5 `<Dialog>`'s default `children` slot (fixing missing body text), replaced blue-with-red-text destructive button with a genuine solid iOS red pill (`#d70015` / `#ff453a` with crisp white text) and neutral iOS Cancel pill, and standardized concise single-line `"Delete"` CTA labels across all delete actions.
- **Zero Duplicate Clear Icon in Search Bar (`+page.svelte`):** Switched search input from `type="search"` to `type="text" inputmode="search" enterkeyhint="search" role="searchbox"` so WebKitGTK/Tauri never renders a second native `[x]` icon beside Xmemos's circular iOS clear button.
- **Crisp Minimalist iOS 2026 Navbar Titles (`app.css`):** Disabled WebKitGTK's `backdrop-blur-[2px]` halo/bloom on `.k-navbar` so all top-center page titles (`Xmemos`, `Daily Review`, `Settings`, `Menu`, etc.) render razor-sharp with zero glow or blur.
- **Zero-Scrollbar Fluid Activity Heatmap (`ActivityHeatmap.svelte`):** Replaced fixed-width horizontal scrolling with a responsive `repeat(weeks, minmax(0, 1fr))` CSS grid and `aspect-square` cells so the heatmap fits 100% of card width cleanly with zero horizontal scrollbar and proper singular/plural count grammar.
- **Archived Memo Attachments (`archived/+page.svelte`):** Added `<AttachmentGrid resources={view.resources} />` to archived memo cards so images (with Lightbox), videos, audio, and static files remain fully visible in the Archive view.

## [2.0.3] - 2026-11-25

### Added
- **Configurable Delete Confirmation Popup (`confirm-delete.svelte.ts`):** Added a native Konsta UI iOS `<Dialog>` confirmation popup before permanently deleting memos, archived memos, media attachments, inbox notifications, or local user accounts, controlled by a persistent **Confirm Before Deleting** `<Toggle>` in **Settings → Preferences** (synced to both `localStorage` and `xmemos_local.db`).

### Fixed
- **Clipped Top-Left Title & Blurry Menu Title:** Added `!translate-none` alongside `!inset-0 !transform-none` on both the main `Navbar` and mobile `Panel` `Menu` `Navbar` (resetting Tailwind v4's standalone CSS `translate: -50% -50%` property), making all centered titles dead-center and razor-sharp without WebKitGTK GPU blur or top-left clipping.
- **Search Bar Clear Button Behavior:** Restricted the search pill `X` button to appear strictly when search query text is non-empty (`filterState.query.length > 0`) rather than when switching quick filter chips (`Tasks`, `Media`, `Links`, `Code`).
- **MemoEditor Toolbar Layout:** Replaced the scrolling toolbar with a responsive 10-column icon grid (`grid-cols-10`) and an icon-only `Paperclip` attach button (with optional numeric badge when files are attached), eliminating horizontal scrollbars on narrow mobile viewports.
- **FAB Precision & Bottom Clearance:** Refined the floating `+` button size (`52×52px`), elevation, and scroll container bottom clearance so bottom memo cards never collide with the FAB or floating iOS `Tabbar`.

## [2.0.2] - 2026-11-25

### Fixed
- **100% Official SvelteKit 3 `#lib/*` Subpath Imports:** Configured standard Node subpath imports (`"imports": { "#lib/*": "./src/lib/*" }`) in `package.json`, removed the `compilerOptions.paths` override from `tsconfig.json` per official SvelteKit v3 `$app/tsconfig` guidance, and added explicit `.ts` / `.svelte.ts` / `.svelte` extensions across all `#lib/*` imports so `pnpm dev`, `pnpm tauri dev --release`, `pnpm check`, `pnpm test`, and `pnpm build` run with zero `module_removed_lib` errors and zero `config_option_deprecated_alias` warnings.
- **Repository Documentation Sync:** Updated `README.md`, `CHANGELOG.md`, `docs/ARCHITECTURE.md`, and `docs/ai/CONVENTIONS.md` to reflect SvelteKit 3 `#lib/*` conventions, Konsta UI v5 iOS layout rules, WCAG 2.2 standards, and `MemoEditor` Write/Preview + pre-save attachment previews.

## [2.0.1] - 2026-11-24

### Fixed
- **Fatal `500 Internal Error` in MemoEditor:** Fixed a Svelte 5 `effect_update_depth_exceeded` infinite reactive loop in `MemoEditor.svelte` caused by reading and reassigning `localPreviewUrls` inside `$effect` (now safely isolated with `untrack()`), and added a Konsta UI iOS error boundary (`src/routes/+error.svelte`).
- **Deprecated SvelteKit Alias Warning:** Removed deprecated `config.alias` from `vite.config.ts` in favor of Node subpath imports.
- **Settings `BlockTitle` Overlap & Alignment:** Fixed Konsta UI v5 `BlockTitle` negative bottom margin (`-mb-6`) overlapping inset `List` cards across Settings, Storage, System, Users, and Backup panels, and aligned inset lists flush (`!mx-0`) with cards.
- **Home Search Bar UI/UX:** Suppressed duplicate native WebKit/Chromium search clear button, moved focus styling onto the outer rounded iOS search pill (`focus-within`), removed the harsh square inner focus box, and added a dedicated "No matching memos" + "Clear Filters" empty state.
- **Navbar Title Crispness & Single Glass Capsule:** Removed `transform-gpu -translate-x-1/2 -translate-y-1/2` subpixel text blur on centered Navbar titles in WebKitGTK, and removed the double border around the `@username` button inside Konsta UI's native `<Glass>` right slot.
- **Activity Heatmap & Account Switcher:** Auto-scrolled `ActivityHeatmap` to the rightmost (today) column so recent activity is immediately visible, and aligned `AccountSwitcherSheet` list/tonal buttons flush.

## [2.0.0] - 2026-11-24

### Added
- **Memo Editor Write / Preview Toggle Tab:** Native Konsta UI iOS `Segmented` tab switcher (`Write` / `Preview`, `role="tablist"`/`role="tab"`, `Ctrl/Cmd + Shift + P`) inside `MemoEditor.svelte` rendering live Markdown, KaTeX math, PrismJS syntax highlighting, `#tag` pills, and attached media previews before saving.
- **Pre-Save Static File & Image Upload Preview (`AttachmentPreviewStrip.svelte`):** Rich visual preview strip inside the memo composer showing image thumbnails with full-screen Lightbox inspection, inline video/audio players, typed static file cards with extension badges and byte sizes, upload progress indicators, drag-and-drop, clipboard image paste, and memory-safe ObjectURL cleanup.
- **Native iOS Subpage Back Navigation:** Added `NavbarBackLink` support in `AppShell.svelte` across all settings subpages, memo details, user profiles, inbox, and archived views.

### Changed
- **Konsta UI v5 iOS Theme Alignment:** Removed conflicting double-glass/positioning wrappers around Konsta UI v5 `Navbar` and floating `Tabbar`/`ToolbarPane`, added native `Navbar` header to mobile slide-over `Panel`, and constrained `Sheet` modals to overflow-safe `max-h-[90dvh]` sticky header/footer layouts.
- **Light & Dark Mode Stability:** Enabled permanent `dark={true}` variant emission on root `<App theme="ios" dark={true} safeAreas>`, added synchronous pre-paint theme initialization in `src/app.html` to eliminate FOUC, synchronized explicit `color-scheme: light | dark`, and introduced semantic high-contrast CSS tokens (`--xm-text-primary`, `--xm-text-secondary`, `--xm-text-muted`, `--xm-accent`, `--xm-input-bg`, `--xm-divider`) with WCAG-compliant PrismJS code highlighting.
- **Strict WCAG 2.2 Accessibility:** Removed `user-scalable=no` / `maximum-scale=1` viewport zoom restrictions, added skip-to-content link, global `:focus-visible` outlines, semantic ARIA landmarks (`aria-current`, `aria-pressed`, `aria-expanded`, `role="status"`, `aria-live="polite"`), accessible form `<label>` associations, Escape-closable Lightboxes, and consistent touch targets across all 16 routes and shared components.

## [1.0.0] - 2026-11-24

### Added
- **1:1 Official Memos SQLite Engine:** Dual-DB isolation (`memos_prod.db` + device-only `xmemos_local.db` with FTS5) running in SQLite WAL mode.
- **Multi-User Offline Authentication:** Local `HOST`, `ADMIN`, and `USER` roles with `bcrypt` hashing, strict `PRIVATE`/`PROTECTED`/`PUBLIC` visibility matrix, quick account switcher, and per-user 6-digit PIN + Android `BiometricPrompt`.
- **Android Storage Plugin (`tauri-plugin-xmemos-storage`):** Native `/Android/media/org.xmemos.app/` resolution, Android SAF document picker/saver, `.nomedia` toggle, and WebP thumbnail pipeline.
- **Dual-Mode Backup & Atomic Import:** Mode 1 (`.db`) and Mode 2 (`.xmemos` ZIP with SHA-256 `manifest.json`), pre-import inspection, rollback snapshot, and ZIP-slip protection.
- **Konsta UI iOS + Liquid Glass Frontend:** SvelteKit + Svelte 5 Runes SPA with Markdown, KaTeX, PrismJS, DOMPurify, interactive task checkboxes, nested tag tree, 365-day activity heatmap, comments, reactions, inbox, and Liquid Glass PNG share-card export.
- **Deterministic CI/CD & Governance:** Manual `workflow_dispatch` Android release pipeline, `google/addlicense` AGPLv3 automation, and AI agent governance docs (`AGENTS.md`, `docs/ai/*`).
