# Xmemos System Architecture

## High-Level Topology

```text
+-------------------------------------------------------------------+
|  SvelteKit 3 + Svelte 5 SPA (Konsta UI v5 iOS + Liquid Glass)     |
|  +layout.svelte (Persistent BottomTabbar + Global Confirm Dialog) |
|  src/routes/*  -->  #lib/state/*.svelte.ts  -->  #lib/api/*.ts    |
+-----------------------------------+-------------------------------+
                                    | Tauri v2 Brownfield IPC
                                    | (@tauri-apps/api/core invoke +
                                    |  xmemos-media://localhost/<uid>)
+-----------------------------------v-------------------------------+
|  Rust Core (src-tauri/src/)                                       |
|  commands/*  -->  service/* (auth, search, media, backup)         |
|              -->  store/*   (1:1 Memos SQLite repositories)       |
+-----------------+----------------------------------+--------------+
                  |                                  |
     +------------v------------+       +-------------v--------------+
     |  Dual SQLite WAL Engine |       | tauri-plugin-xmemos-storage|
     |  1. memos_prod.db (1:1) |       | Kotlin Android Bridge:     |
     |  2. xmemos_local.db     |       |  - externalMediaDirs       |
     |     (FTS5, locks, prefs)|       |  - SAF Open/Create Doc     |
     +-------------------------+       |  - .nomedia toggle         |
                                       |  - BiometricPrompt         |
                                       +----------------------------+
```

## Frontend Module Resolution, Timezone & UI Layer

- **SvelteKit 3 Subpath Imports:** Internal library imports use `#lib/*` declared in `package.json` `"imports"` with explicit `.ts` / `.svelte.ts` / `.svelte` extensions (zero deprecated `config.alias` or legacy `$lib` usage).
- **Persistent Native iOS Navigation:** `BottomTabbar.svelte` is mounted once at root in `src/routes/+layout.svelte` so Konsta UI v5's sliding glass pill thumb transitions smoothly between routes without resetting to Home, paired with lightweight GPU-composited `.xm-page-enter` transitions.
- **Local Device Timezone Accuracy & Monthly Activity Calendar:** `ActivityHeatmap.svelte` renders an Official Memos 2026 numbered monthly calendar matrix (localized via `Intl.DateTimeFormat`, with `<`/`>` month navigation and stepped activity intensity tiles), and all date filters and `datetime-local` inputs operate in the device's local timezone (`toLocalDateKey`, `formatLocalDateTimeInput`, `parseLocalDateTimeInput`, `getLocalDayBounds`, and Rust `chrono::Local`).
- **Official Memos 26.11.2 Visual & Interaction Parity:** `MemoCard.svelte` uses the top-right amber corner-fold pin indicator, single-line author/time header with right-aligned visibility icon, and location/relation metadata rows; `InfiniteScrollSentinel.svelte` renders an interactive `Back to Top ↑` control at feed end; `ActivityHeatmap.svelte` includes a `Today` quick-jump alongside `<` `>` month navigation.
- **Konsta UI v5 iOS + WCAG 2.2:** Root `<App theme="ios" dark={true} safeAreas>` drives native iOS `Navbar`, `NavbarBackLink`, `Tabbar`/`ToolbarPane`, `Sheet`, `Panel`, `List`, `Dialog`, and `Segmented` controls, backed by WCAG AA/AAA semantic Light/Dark tokens in `src/app.css`.
- **Smart Offline Code Highlighting (`code-highlight.ts`):** Bundles 28 PrismJS grammars locally with automatic language inference for unlabeled blocks, universal comment highlighting, code-safe `#tag`/`$math$` isolation, one-tap copy, and dual Light (`#efefe9`) / Dark (`#222326`) Official Memos Demo palettes.
- **Memo Composer & Pre-Save Previews:** `MemoEditor.svelte` provides a `Write` / `Preview` segmented toggle rendering live Markdown/KaTeX/PrismJS output alongside `AttachmentPreviewStrip.svelte` for pre-save image, video, audio, and static file inspection.

## Dual-Database Isolation

- **`memos_prod.db`**: Strictly 1:1 Official FOSS Memos SQLite schema. Contains zero proprietary tables or columns. Directly portable to/from any self-hosted Memos server.
- **`xmemos_local.db`**: Device-only SQLite WAL database holding local multi-user session state (`local_session`), per-user PIN/biometric metadata (`user_lock`), composer draft autosaves (`memo_draft`), Liquid Glass, delete-confirmation, and sign-out password enforcement (`require_password_after_sign_out`) preferences (`local_pref`), backup history (`backup_history`), and the synchronized FTS5 search index (`memo_fts`). Rebuilt automatically after any database import.

## Android Media Storage & Custom Protocol

Static media attachments are written to `/storage/emulated/0/Android/media/org.xmemos.app/assets/YYYY/MM/<uid>_<sanitized_filename>` via `externalMediaDirs` (no `MANAGE_EXTERNAL_STORAGE` required). Cached 360px WebP thumbnails are generated in `cacheDir/thumbs/<uid>.webp`. The WebView resolves media exclusively through Tauri v2's `convertFileSrc(uid, 'xmemos-media')` (`http(s)://xmemos-media.localhost/<uid>` on Android/Windows and `xmemos-media://localhost/<uid>` on GNU/Linux/macOS, with optional `?thumb=1`), where the Rust `handle_media_protocol` handler validates SQLite memo visibility and serves bytes with HTTP `Range` (`206 Partial Content`) and CORS headers (v2.7.32).

## Startup, Runtime & Shutdown Performance Architecture

- **O(1) Rust Cold Launch**: `AppState::initialize()` opens both SQLite WAL databases with zero-copy `PRAGMA mmap_size = 67108864`, `PRAGMA synchronous = NORMAL`, and `PRAGMA wal_autocheckpoint = 1000` without blocking on `temp/` cleanup or full FTS5 re-indexing. Background startup housekeeping runs on a dedicated thread via `ensure_fts_index_ready` (O(1) count check).
- **Instant Non-Blocking Exit**: `RunEvent::Exit` triggers `AppState::prepare_fast_shutdown()` using `try_lock()` + `PRAGMA wal_checkpoint(PASSIVE); PRAGMA optimize;` so closing the app never hangs.
- **Zero-Flash Native Window Reveal**: `tauri.conf.json` launches `main` with `visible: false` and `backgroundColor: "#000000"` (dynamically synced to `#f2f2f7` in Rust `setup` when `theme_mode = light`). `+layout.svelte` invokes `app_show_main_window` after double-`requestAnimationFrame` first paint (backed by a 1200ms Rust failsafe), eliminating OS gray frames and WebView white flashes.
- **Smart Infinite Scroll, Lazy Loading & Adaptive iOS Skeletons**: `InfiniteScrollSentinel.svelte` uses `IntersectionObserver` (`320px` prefetch margin + concurrency/deduplication guards) across Home, Explore, Archived (`PAGE_SIZE = 20`), and Resources (24-item progressive media windowing), paired with zero-delay GPU-shimmer `KonstaSkeleton.svelte`, conditional `MemoEditor` lazy mounting, `preload="none"` media, and a single root-mounted `MemoActionsSheet.svelte` portal (preventing any CSS containment or stacking-context clipping of `position: fixed` overlays).
- **Frame-1 Instant UI Hydration**: `src/app.html` paints the exact Light/Dark iOS canvas before stylesheet load; `+layout.svelte` and `+page.svelte` synchronously hydrate cached auth and first-page timeline snapshots (strictly wiped on sign-out/switch), revalidate timeline + sidebar metadata in parallel (`Promise.all`), defer non-critical `local_pref` sync to `requestIdleCallback`, and cache parsed Markdown via a 250-entry LRU cache.

## Internationalization (`src/lib/i18n/`)

- **Zero Hardcoded UI Strings**: All user-facing strings live in isolated TypeScript locale catalogs (`locales/en.ts`, `locales/id.ts`, `locales/es.ts`) typed against `TranslationKey = keyof typeof en`.
- **Compile-Time + Test-Time Guarantees**: Missing or extra keys fail `svelte-check`; `src/lib/i18n/i18n.test.ts` verifies 100% key parity and identical `{param}` placeholders across all locales.
- **Zero-Lag Reactivity & Persistence**: `i18n` (`index.svelte.ts`) uses Svelte 5 `$state` for instant O(1) lookups without page reloads, syncs `document.documentElement.lang` (pre-paint via `src/app.html`), formats dates via `i18n.bcp47`, and persists user preference to `localStorage` + `xmemos_local.db` (`local_pref`).

## Custom Creation Timestamps & Calendar Entry Parity (v2.1.17)

Aligned with the official Memos documentation (`usememos.com/docs/usage/memos`) and 26.10/26.11 calendar workflow:
- **Unrestricted Past & Future Timestamps**: Both `memo_create` and `memo_update` accept any valid Unix timestamp (`created_ts`), allowing backdated journal entries and future-dated memos without date caps. Initial creation initializes `updated_ts = created_ts`.
- **Explicit Auto-Now vs. Custom Picker (`MemoEditor.svelte`)**: Defaults to automatic save-time recording (`Now (Auto)`) with a 1-tap **Set Date & Time** toggle, `Past`/`Future` visual indicator, and 1-tap **Reset to Now** control. Untouched edits preserve the exact original second.
- **New Memo on This Day**: Selecting any date in `/review` (or filtering by date on `/`) allows creating a memo pre-filled with that calendar date (`YYYY-MM-DD`) and current local clock time (`HH:mm`).
- **All-Time Heatmap & Chronological Ordering**: `memo_get_heatmap` with `days = 0` aggregates all `NORMAL` memos across all years (past and future), and client state (`memosState`) maintains `pinned DESC, createdTs DESC, id DESC` ordering immediately on create or timestamp edit.

## Full-Page Monthly Calendar & Responsive Day Panel (v2.1.18)

Modeled after Official Memos 26.10/26.11 (`usememos.com/docs/usage/memos` & `demo.usememos.com/calendar`):
- **Single-Pass O(N) Indexing (`FullMonthCalendar.svelte`)**: Groups memos once into `byDay` (`YYYY-MM-DD`), `byMonth` (`YYYY-MM`), and active `years` inside a single `$derived.by` memoized computation so navigating across months and years runs at 60fps with zero extra IPC calls.
- **Full-Canvas Responsive Layout (`AppShell.svelte` + `/review`)**: Allocates full available canvas width to `/review`, pairing the 7-column calendar grid with a sticky right-hand Day Detail Panel on desktop (`lg+`) and a smooth below-grid panel on mobile/tablet (`< lg`), keeping every cell legible from 360px phones to ultrawide monitors.

## Timeline Display Modes & SQLite Sorting (v2.1.20)

- **Viewport-Aware Multi-Column Feed (`feed-display.svelte.ts`, `TimelineDisplayMenu.svelte`)**: Persists `layout` (`list | 2 | 3 | auto`), `fullWidth`, `orderBy` (`created_ts | updated_ts`), and `orderDirection` (`desc | asc`) locally per device. Multi-column CSS masonry and full-width expansion activate strictly at `md+` (`>= 768px`), keeping Android phones on a lightweight 1-column list.
- **Whitelisted SQLite Sorting (`store/memo.rs`)**: `list_memos` strictly whitelists `m.created_ts`/`m.updated_ts` and `DESC`/`ASC` while preserving `m.pinned DESC` priority.
- **Zero-Scrollbar Calendar Fit (`FullMonthCalendar.svelte`)**: Uses `--xm-cal-weeks` CSS grid row division inside a viewport-locked `md:h-[calc(100dvh-5.25rem)]` container on tablet/desktop while allowing natural vertical touch flow on mobile.

## Independent 3-Column Scrolling, Flexbox Masonry & Fullscreen Lightbox (v2.1.21)

- **Independent Desktop Scroll Panes (`AppShell.svelte`)**: At `md+`, `<Page>` locks to `h-dvh flex-col overflow-hidden` with `shrink-0` `<Navbar>`, giving Left Rail, Center `#main-content`, and Right Rail independent `overflow-y-auto overscroll-contain` scroll containers. On mobile (`< md`), native single-document scrolling is preserved.
- **Flexbox Masonry (`MemoMasonryGrid.svelte`)**: Distributes memos round-robin across normal Flexbox columns (`1` on mobile, `2`/`3`/`4` on tablet/desktop) so `backdrop-filter` glass cards, code blocks, and `Back to Top ↑` never suffer from CSS `columns` fragmentation.
- **Fullscreen Image Lightbox (`ImageLightbox.svelte`)**: Supports 25%–400% zoom, pointer drag-to-pan, 90° rotation, natural dimension inspection (`ⓘ`), direct download (`↓`), and multi-image gallery navigation (`←`/`→`).

## Hybrid Search & Safe Keyword Highlighting (v2.1.25)

- **Hybrid Rust Search (`src-tauri/src/service/search.rs`)**: Combines `xmemos_local.db` FTS5 `bm25` prefix ranking with parameterized `INSTR(LOWER(content), LOWER(?)) > 0` substring matching on `memos_prod.db` so mid-word and punctuation queries match reliably while preserving strict SQLite visibility enforcement.
- **Enter-to-Commit Search Chips & Safe Highlighting (`markdown.ts`, `MemoContent.svelte`)**: Pressing Enter commits search terms into removable `🔍 <term> ×` chips on Home and Explore, and `highlightHtmlMatches()` wraps case-insensitive text-node matches in `<mark class="xm-search-mark">` while skipping tags, attributes, `<svg>`, and `<math>`.

## FOSS Offline-First Map & OsmAnd Integration (v2.2.0)

- **1:1 Official Memos Location Schema**: Stores `{ placeholder, latitude, longitude }` inside `memo.payload.location` in `memos_prod.db` with zero proprietary extensions.
- **Leaflet 1.9.4 + OpenStreetMap + OsmAnd (`osm-tiles.ts`, `OsmMapCanvas.svelte`, `LocationPreviewPopover.svelte`, `/map`)**: Renders OpenStreetMap tiles via a shared Android WebView-safe tile loader (`xmemos-osm-tiles-v1` `Cache` API → CORS fetch → direct `<img>` fallback → legible light/dark offline SVG grid), backed by explicit FOSS OSM CSP rules in `tauri.conf.json`, compositor-safe dark-mode dimming (`opacity: 0.84` instead of GPU `filter` on `.leaflet-tile-pane`), definite mobile container sizing (`absolute inset-0` + `md:flex-1`), proximity marker clustering, tap-to-drop planning pins with **Write here**, and native Tauri `opener` `geo:lat,lng?q=...` intents to launch **OsmAnd** directly (hardened v2.7.30).
- **Separated Map Tags & `Show as tree` (`TagTree.svelte`, `tag-view.svelte.ts`)**: `/map` derives `mapTags` strictly from located memos while Home/Explore keep unified tags; all tag panels share the persistent `Show as tree` toggle (flat list vs. collapsible hierarchy).

## macOS 2026 Desktop Split-View Architecture (v2.3.0)
- **Strict Viewport Separation**: Mobile (`< 768px`) retains the native Konsta UI v5 `<Navbar>`, floating `<Fab>`, slide-over `<Panel>`, and persistent `<BottomTabbar>`; Tablet/iPad (`768px–1023px`) retains the `md:` 2-column grid (`228px` rail + scrollable main). Desktop (`lg >= 1024px`) transitions cleanly into an edge-to-edge **macOS 2026 Split-View** (`NavigationSplitView` + `52px` Unified `NSToolbar` pattern) without altering `< 1024px` behavior.
- **Persistent Collapsible Panes (`desktopLayoutState`)**: Stores `sidebarOpen` (`Ctrl+\` / `⌘\`) and `inspectorOpen` (`Ctrl+Shift+\` / `⌘⇧\`) in `localStorage` (`xmemos.desktop.layout.v1`), hydrated synchronously on frame 0 in `src/routes/+layout.svelte`.
- **Single-Mount Content Canvas**: `AppShell.svelte` uses `display: contents` below `lg` and flex split layout at `lg+` so `{@render children()}` is mounted strictly once, allowing `ResizeObserver`-driven components (`OsmMapCanvas`, `MemoMasonryGrid`) to adapt seamlessly whenever the sidebar or inspector toggles.
- **Liquid Glass Material & Accessibility Compliance**: Desktop tokens (`--xm-macos-sidebar-bg`, `--xm-macos-toolbar-bg`, `--xm-macos-inspector-bg`) automatically adapt across Light/Dark themes, Xmemos glass intensity modes (`full` / `subtle` / `solid`), and `@media (prefers-reduced-transparency: reduce)`.

## Nearest-Region Geocoding & Memo Mini-Map Popover (v2.3.1)
- **FOSS Reverse Geocoding (`src/lib/utils/geocode.ts`)**: Uses OpenStreetMap Nominatim (`format=jsonv2&zoom=14&addressdetails=1`) with a `3500ms` timeout, persistent `localStorage` cache (`xmemos.geocode.cache.v1`, 3-decimal coordinate key), and graceful offline coordinate fallback so pinned memos automatically display concise regional names (`locality, state, country`).
- **Viewport-Safe Mini-Map Popover (`MemoLocationPreview.svelte`)**: Clicking a memo's location row opens a `position: fixed` Liquid Glass popover clamped inside the viewport (flipping above when near the bottom edge), rendering a lazy Leaflet `1.9.4` mini-map sharing the `xmemos-osm-tiles-v1` offline tile cache with top-right full-map (`↗`), zoom (`+`/`-`), degree coordinates, and OsmAnd (`geo:`) launch.

## Desktop Left Rail Search, Filter & Display Controls (v2.3.2)
- **Dedicated Sidebar Slots (`desktopLeftSearch`, `desktopLeftControls`)**: `AppShell.svelte` exposes optional Desktop-only (`lg >= 1024px`) sidebar snippets used by Home (`/`) and Explore (`/explore`) to host the macOS search field at the top of the leading rail and grouped Quick Filters + active filter pills + `TimelineDisplayMenu (sidebarMode)` below navigation, keeping the center timeline distraction-free.
- **Viewport-Safe Sidebar Popover**: `TimelineDisplayMenu` uses `position: fixed` right-of-rail anchoring in `sidebarMode` so it never clips inside the scrollable sidebar, while `< 1024px` viewports keep the original inline search/filter/display header unchanged.

## Root-Mounted Location Popover, Multi-Tier Geocoding & Left-Rail Tags (v2.3.3)
- **Root Popover Layer (`LocationPreviewPopover.svelte`, `locationPreviewState`)**: Mounted at root in `src/routes/+layout.svelte` (`z-[80]`) so `position: fixed` mini-map previews are never clipped or shifted by `backdrop-filter` / `overflow-hidden` on `MemoCard`.
- **Multi-Tier FOSS Geocoding (`src/lib/utils/geocode.ts`)**: Chains OpenStreetMap Nominatim (`zoom=10`), Komoot Photon (`photon.komoot.io/reverse`), and Nominatim (`zoom=6`) with in-flight request deduplication and `localStorage` caching; `MemoEditor.handleSubmit()` awaits region resolution when a pinned label is still coordinate-only.
- **Desktop Left Rail Cohesion**: Places Search, Filters, and Display at the top of the Desktop Left Rail and moves `TagTree` (global on Home/Explore, `desktopLeftTags` map-scoped on `/map`) into the Left Rail while keeping Mobile/Tablet layouts (`< 1024px`) intact and enabling the mobile `+` FAB on Explore.
- **Independent Tags Scroll Pane (v2.3.5)**: Inside the Desktop Left Rail, `Workspace`/`Library` navigation (`shrink-0`, capped at `max-h-[48%]` for ultra-short viewports) is decoupled from the `Tags` pane (`min-h-0 flex-1 overflow-y-auto overscroll-contain`) so browsing long tag trees never scrolls navigation out of view.
- **Context-Aware Desktop Tags Rail Placement (v2.3.7)**: On Home (`/`) and Explore (`/explore`), `TagTree` renders in the Desktop Right Inspector Rail inside an independent scroll pane beneath the pinned `ActivityHeatmap`. On full-canvas routes (`/review` and `/map`), `TagTree` renders in the Desktop Left Rail inside an independent scroll pane beneath pinned `Workspace`/`Library` navigation.
- **Mobile Map Slide-Over Access (v2.3.10)**: `/map` is exposed inside the mobile slide-over `<Panel>` drawer (`MOBILE_EXTRA_ITEMS`) rather than crowding the 5-tab `BottomTabbar`, with mobile-first canvas ordering (`order-1`), smooth scroll-into-view for selected pin details (`order-2`), and Map-scoped tags (`order-3`).

## Frameless Integrated Desktop Window Header (v2.4.0)
- **Desktop-Only Frameless Window**: `src-tauri/tauri.conf.json` configures `"decorations": false` with `width: 1180, height: 780, minWidth: 360, minHeight: 540, center: true`, reinforced by `#[cfg(desktop)] window.set_decorations(false)` in `src-tauri/src/lib.rs`.
- **Least-Privilege Platform Capability**: `src-tauri/capabilities/desktop.json` scopes Tauri v2 window control permissions (`start-dragging`, `minimize`, `toggle-maximize`, `internal-toggle-maximize`, `close`, `is-maximized`) exclusively to `["linux", "macOS", "windows"]`, leaving `capabilities/mobile.json` (Android) completely isolated.
- **macOS 2026 Traffic-Light Integration**: `isDesktopTauri()` (`src/lib/utils/platform.ts`) gates `DesktopWindowControls.svelte` and `handleTitlebarMouseDown` so traffic-light controls and `data-tauri-drag-region` render only inside Tauri Desktop WebViews—embedded in the `52px` Sidebar header, automatically relocating to the `52px` Unified Toolbar when the sidebar collapses, falling back to a `34px` Liquid Glass strip on narrow desktop windows (`< 1024px`), and floating on `/auth`.

## macOS 2026 Desktop Pull-Down Memo Options Menu (v2.5.0)
- **Viewport-Scoped Presentation (`MemoActionsSheet.svelte`, `memo-actions.svelte.ts`)**: When `window.innerWidth >= 1024` and `MemoCard.svelte` provides the clicked `···` button's `anchorRect`, `MemoActionsSheet` renders an anchored `216px` Liquid Glass floating pull-down menu (`role="menu"`, `z-[80]`) with Apple HIG hairline grouping, macOS accent hover states, destructive red Delete styling, automatic top/bottom viewport edge flipping, scroll/resize dismissal, and `Escape` / Arrow-key navigation.
- **Mobile & Tablet Preservation**: Below `1024px`, `MemoActionsSheet` continues rendering the native Konsta UI v5 iOS `<Actions>` bottom sheet with zero layout or interaction changes.

## Centralized Keyboard-Driven Engine & Desktop GPU Performance (v2.6.0)
- **Single-Dispatcher Shortcut Architecture (`shortcuts.svelte.ts`, `+layout.svelte`, `KeyboardShortcutsModal.svelte`)**: Global shortcuts (`Ctrl+/` / `?` cheatsheet, `N`/`C` compose, `Ctrl/Cmd+K` or `/` search, `G` two-key chords, `Alt+1–5`, `Ctrl/Cmd+\` & `Ctrl/Cmd+Shift+\` split-view toggles) run through one root listener with IME/input/modal guards while active pages register contextual `onCompose` and `onFocusSearch` hooks cleanly. `MemoEditor.svelte` implements the full Official Memos 2026 Markdown editor keymap.
- **Zero-Reflow Desktop Compositing (`AppShell.svelte`, `app.css`)**: Desktop sidebar and inspector toggles avoid animating `width` (which triggers per-frame layout reflows and `@container` recalculations), using instant column switching with GPU-composited `transform`/`opacity` transitions, `contain: layout style` on desktop chrome, removal of `background-attachment: fixed`, and reduced multi-layer desktop `backdrop-filter` overhead for smooth WebKitGTK/macOS/Windows rendering.

## Adaptive Multi-Platform Build & Native Desktop File Dialogs (v2.6.0)
- **Compile-Time Platform Stripping (`vite.config.ts`, `src/lib/utils/platform.ts`)**: `XMEMOS_TARGET_PLATFORM` (`android` | `desktop` | `auto`) injects `__XMEMOS_TARGET__` at build time so `supportsNomedia()` and `supportsAndroidBiometric()` resolve statically to `false` on Desktop builds (hiding Android `.nomedia` and `BiometricPrompt` UI), while `isDesktopTauri()` resolves statically to `false` on Android APK builds.
- **Zero-Extra-Crate Native Desktop File Manager Dialogs (`plugins/tauri-plugin-xmemos-storage/src/desktop_dialog.rs`)**:
  - **GNU/Linux (AppImage & Flatpak)**: Dispatches `GtkFileChooserNative` on the GTK main thread via `app.run_on_main_thread()`, automatically using `org.freedesktop.portal.FileChooser` inside Flatpak sandboxes without requiring `--filesystem=host`.
  - **Windows 10/11**: Calls Win32 `comdlg32.dll` (`GetSaveFileNameW` / `GetOpenFileNameW`) with full UTF-16 Unicode paths and overwrite prompts.
  - **macOS**: Invokes `/usr/bin/osascript` with strict `argv` parameter passing to present the native Finder sheet.
- **Desktop Portable Mode (`desktop.rs`, `src-tauri/src/lib.rs`)**: Detecting `portable.flag` beside the executable (or `XMEMOS_PORTABLE=1`) redirects `db/`, `media/`, and `cache/` into `<exe_dir>/data/` for self-contained Windows Portable operation.

## Strict 6-Digit Local PIN Lock Architecture (v2.6.1, hardened v2.6.4)
- **Strict Numeric Sanitization & Validation (`src/lib/utils/pin.ts`)**: Exports `PIN_PATTERN = '[0-9]{6}'` (avoiding Svelte `{6}` template interpolation into `[0-9]6`), pairs `pattern={PIN_PATTERN}` with `novalidate` forms for localized inline/toast validation, and enforces ASCII digits `0–9` with a hard 6-digit ceiling via `handlePinKeydown` (Android/IME `Unidentified`/`Process`-safe) and `sanitizePinInput` across both **Settings → Account** (with matching **Confirm 6-Digit PIN** verification) and `PinLockOverlay.svelte`.
- **Cold-Start Enforcement & Rust Hardening (`src/lib/state/auth.svelte.ts`, `src-tauri/src/service/auth.rs`)**: `AuthState.refresh()` automatically locks on initial app launch when `hasPin` is enabled (with an on-demand **Lock Now** trigger in Account Settings), while Rust `set_user_lock_config` and `verify_user_pin` strictly validate 6 ASCII digits and hash PINs using `bcrypt` cost `10` inside `xmemos_local.db`.

## macOS 2026 Liquid Glass Application Icon & Cross-OS Asset Pipeline (v2.6.3)
- **Single-Source Canonical Vector (`src-tauri/icons/icon.svg`, `static/icon.svg`)**: Uses standard SVG 1.1/2.0 gradients, `clipPath`, and `userSpaceOnUse` drop-shadow filters (`1024×1024` viewBox, `G2`-continuous `864×864` Apple superellipse, frosted folded-corner memo card, and interlocking Liquid Glass **X** emblem) to guarantee identical, lightweight rendering across WebKitGTK, WKWebView, WebView2, Android WebView, and Linux `librsvg`.
- **Cross-OS Native Icon Bundles**: Regenerated from the canonical SVG into Tauri PNGs (`32x32.png`, `128x128.png`, `128x128@2x.png`, `icon.png`), Android Fastlane `512×512` `icon.png`, Windows 7-size PNG-compressed RGBA `icon.ico` (`16–256px`), macOS 11-chunk Retina `icon.icns` (`16–1024px`), and GNU/Linux Flatpak `hicolor/scalable/apps/org.xmemos.app.svg`.
- **In-App Brand Alignment (`src/lib/components/ui/AppIcon.svelte`)**: Renders the canonical vector inline with Svelte 5 `$props.id()`-namespaced gradient/filter IDs across `/auth` (replacing the former `<Shield>` icon), the Desktop Sidebar header (`AppShell.svelte`), **Settings → About**, and the `src/app.html` SVG favicon.

## 1:1 MemoCard PNG Export Share Card Engine (v2.6.5)
- **Modular Markdown & Canvas 2D Pipeline (`src/lib/utils/share-card-markdown.ts`, `src/lib/utils/share-card.ts`)**: Parses memo Markdown into structured blocks (headings, paragraphs, blockquotes, task checkboxes, lists, fenced code blocks, horizontal rules) and typed inline runs (bold, italic, strikethrough, inline code, `#tags`, links) and renders them via pure HTML5 Canvas 2D (avoiding WebKitGTK/WKWebView `foreignObject` canvas tainting) using exact `src/app.css` Light/Dark Liquid Glass tokens, amber pin corner fold, avatar/timestamp/visibility header, image grids, location/relations metadata, grouped reactions, and a minimal `Xmemos` emblem without `"Offline Memos"` text.

## Cross-OS Native MS Sticky Notes & Memo Shortcuts (v2.7.0, refined v2.7.6)
- **Android (`plugins/tauri-plugin-xmemos-storage/android/.../XmemosStoragePlugin.kt`)**: Uses official AndroidX `ShortcutManagerCompat.requestPinShortcut()` and `pushDynamicShortcut()` with a native `192×192` widget-style Liquid Glass adaptive bitmap (`IconCompat.createWithAdaptiveBitmap`) and handles cold (`Plugin.load`) + warm (`Plugin.onNewIntent`) Intent launches via `EXTRA_MEMO_UID` validated against `^[a-zA-Z0-9_-]{1,64}$`.
- **Desktop MS Sticky Notes Shortcuts (`plugins/tauri-plugin-xmemos-storage/src/{desktop_shortcut.rs,sticky_icon.rs}`, v2.7.31)**: Zero-extra-crate native shortcut & sticky-note icon generator producing FreeDesktop `.desktop` launchers with real memo-preview SVG sticky cards on GNU/Linux (Flatpak, AppImage `$APPIMAGE`, and native binary aware), native Shell `.lnk` files with 128×128 RGBA PNG-compressed `.ico` sticky cards on Windows 10/11 (`WScript.Shell`), and silent `LSUIElement` `.app` bundles with 128×128 `ic07` `.icns` sticky cards on macOS (`open -a ... --args --open-memo=<uid>`). Launching a Desktop shortcut opens directly into the floating MS Sticky Note window while keeping the main window hidden (unless PIN-locked) and exits cleanly when the last sticky closes.
- **Unified Frontend Routing (`src/lib/api/shortcut.ts`, `src/routes/+layout.svelte`, `MemoActionsSheet.svelte`)**: Exposes **Pin to Home Screen** / **Add to Desktop** in memo options and consumes pending/warm shortcut UIDs in `+layout.svelte` to navigate straight to `/memo/<uid>` beneath `PinLockOverlay` protection.
- **Desktop Floating MS Sticky Notes (`src/routes/sticky/[uid]/+page.svelte`, `system_open_sticky_memo_window`)**: Uses async `WebviewWindowBuilder` with `sticky-<uid>` deduplication, a max-8 window RAM cap, and `+layout.svelte` `isStickyRoute` chrome isolation; uses zero-flash `.visible(false)` + palette-matched `.background_color()` + double-rAF reveal, enforces `.maximizable(false)` + `.max_inner_size(720, 860)` + drag-only `handleStickyDragMouseDown` + `unmaximize()` guard to prevent KDE Plasma KWin/GNOME/Windows edge-snap maximize, persists per-memo position, size, 6 Konsta macOS 2026 palettes, and `alwaysOnTop` in `localStorage` while supporting rich `AttachmentGrid` + `ImageLightbox`, `MemoLocationPreview` + `LocationPreviewPopover`, linked memos, interactive reactions/checkboxes, inline quick edit, and a native Rust `window_pin.rs` Always-on-Top engine (Windows `HWND_TOPMOST`, macOS `NSFloatingWindowLevel`, Linux X11 `_NET_WM_STATE_ABOVE`, KDE Plasma 5/6 Wayland KWin D-Bus `/Scripting` `keepAbove`, and Hyprland `pin`).
- **Android Resizable MS Sticky Notes Widget (`XmemosMemoWidgetProvider.kt`)**: Zero-polling (`updatePeriodMillis="0"`) `AppWidgetProvider` rendered via `RemoteViews` + `SharedPreferences` and pinned via `AppWidgetManager.requestPinAppWidget()` (`EXTRA_APPWIDGET_PREVIEW`) on API 26+, falling back cleanly to `ShortcutManagerCompat.requestPinShortcut()`.

## v2.8.0 Offline Vector Map, Android MediaStore Downloads, Liquid Glass Tabbar & Single-Window Sticky Launch
- **100% Offline Vector World Map (`offline-world-data.ts`, `offline-vector-layer.ts`)**: Renders continents, islands, graticules, and major global/Indonesian cities via a 60fps HTML5 Canvas Leaflet `GridLayer` (`zIndex: 1`) beneath transparent-fallback OpenStreetMap tiles (`zIndex: 2`), eliminating blank offline maps completely.
- **Konsta UI v5 iOS 26 Liquid Glass Tabbar (`BottomTabbar.svelte`, `src/app.css`)**: Uses a neutral frosted glass lens pill (no blue active background) with `requestAnimationFrame` realtime page-switching scrub, hysteresis (`0.36`), and `pendingNavIndex` lock to prevent snap-back bounce.
- **Android Public Downloads (`XmemosStoragePlugin.saveToDownloads`)**: Commits exported static files, images, and Share Cards into `Download/Xmemos` via `MediaStore.Downloads` (`IS_PENDING = 1 -> 0` on API 29+) so files are immediately visible in Android File Manager.
- **Comment Visibility Inheritance & Mobile Editor Lock (`commands/memo.rs`, `MemoEditor.svelte`)**: Comments hide visibility controls and inherit their top-level root memo's visibility (cascading on root updates); mobile `MemoEditor` anchors to `top: var(--xm-vvt)` / `height: var(--xm-vvh)` for zero keyboard jump.
- **Desktop Single-Window Sticky Launch (`src-tauri/src/lib.rs`, `AppState.sticky_launch_only`)**: Desktop shortcut launches (`--open-memo=<uid>`) spawn only `sticky-<uid>` in Rust `setup()`, keeping `main` hidden and skipped from taskbar until explicitly opened via `↗ Open in Xmemos`.

## v2.9.3 Ephemeral Backup Export, Hybrid Media Compression & HOST/ADMIN Upload Limit
- **Temp-Staged Backup Export (`service/backup.rs`, `commands/backup.rs`, `BackupRestorePanel.svelte`)**: `backup_export_db` and `backup_export_full` stage `.db` / `.xmemos` files exclusively in `temp/backup_exports/` and invoke `backup_finalize_export` in `try/finally` after the OS/SAF Save dialog completes—guaranteeing zero duplicate copies in `backups/`, zero leaked files on cancel, and history entries strictly for confirmed exports.
- **Hybrid Rust + WebView Media Optimization & Upload Progress (`service/media.rs`, `media-compress.ts`, `AttachmentPreviewStrip.svelte`, `MemoEditor.svelte`)**: Static raster images are compressed via WebView native `libwebp` (VP8 lossy Q0.80, `>= 24 KB`, `<= 1920px`) and Rust `optimize_static_upload` (Lanczos3 `1920px`, lossless WebP fast-path & 8% preference, JPEG Q78/Q74, Best-Adaptive PNG); large files display live per-file progress bars before Save is enabled, and memo edits pass existing `resourceIds` without re-uploading.
- **HOST/ADMIN Upload Size Limit (`store/system_setting.rs`, `commands/system.rs`, `commands/resource.rs`, `settings/system/+page.svelte`)**: Stores `max-upload-size-mb` in `memos_prod.db`'s existing `system_setting` table (default `32` MB, `0` = unlimited, `0..=2048`), restricted to `HOST`/`ADMIN` via `require_admin_or_host` and enforced identically on both frontend and Rust backend.

## v2.9.4 Smart Cross-Platform Video Compression (Android Media3 Hardware + Desktop Optional FFmpeg)
- **Android Hardware Compression (`XmemosVideoCompressor.kt`, Jetpack Media3 `1.11.1`)**: Uses Apache 2.0 FOSS `media3-transformer`, `media3-effect`, and `media3-common` on the Main Looper via `XmemosStoragePlugin.compressVideo`, probing dimensions/bitrate with `MediaMetadataRetriever`, downscaling only when short side > `720px` via `Presentation.createForShortSide(720)`, encoding H.264/AAC with adaptive bitrate (`0.55x`, `0.9–2.2 Mbps`), and guarding with a 90s watchdog timeout + `>= 8%` anti-bloat fallback.
- **Desktop Optional Compression & Async Worker (`service/video.rs`, `commands/resource.rs`)**: Uses system `ffmpeg` if available (`libx264` CRF 26 + `-movflags +faststart` + 90s watchdog + `>= 8%` anti-bloat guard, zero bundled binary), executing inside `tauri::async_runtime::spawn_blocking` so UI/WebView remains 100% responsive.

## v2.9.5 Zero-Jitter Mobile Tabbar Scrub & PowerPoint Directional Slide Transitions
- **Imperative-Only GPU Pill Transform (`BottomTabbar.svelte`, `src/app.css`)**: Removes reactive `transform` from `pillEl`'s Svelte `style` attribute so `requestAnimationFrame` scrubbing never fights Svelte re-renders; applies `touch-action: none`, `contain: strict`, `5px` drag slop, and `165ms` dwell debounce for uninterrupted 60/120fps Android touch tracking.
- **SvelteKit `onNavigate` + View Transitions API (`route-transition.ts`, `+layout.svelte`, `src/app.css`)**: Computes deterministic `forward`/`backward` slide directions across the 5 primary tabs and sub-route depths, runs GPU-composited `210ms` horizontal push/slide keyframes while pinning `.xm-ios-tabbar-shell` (`view-transition-name: xm-bottom-tabbar`), and cancels in-flight transitions via `skipTransition()` for instant responsiveness.

## v2.9.6 Instant Cross-OS Native Startup & Bounded Zero-Bloat Cache
- **Android Native Kotlin First-Frame (`XmemosStartupOptimizer.kt`)**: Sets native Activity/WebView theme background color before first paint, tunes `WebSettings` (`LOAD_DEFAULT`, `offscreenPreRaster`, hardware layer), and prunes stale temp staging (>6h) + `thumbs/` (>48 MiB LRU) on a low-priority background executor.
- **Desktop Native per OS/DE (`src-tauri/src/main.rs`, `state.rs`, `lib.rs`)**: Direct Win32 `advapi32.dll` registry FFI on Windows (`<0.1ms`, zero `reg.exe` spawns) + fast WebView2 args, zero-subprocess NVIDIA+Wayland `WEBKIT_DISABLE_DMABUF_RENDERER=1` guard on GNU/Linux, native WKWebView fast path on macOS, and background Rust 48 MiB LRU `thumbs/` pruning + `PRAGMA optimize`.
- **Frame-0 Frontend Hydration (`+layout.svelte`, `memos.svelte.ts`)**: Synchronous Frame-0 memo snapshot hydration + single-rAF instant Desktop window reveal with strict `128 KB` / `7-day TTL` / `max 3 users` snapshot bounds.
