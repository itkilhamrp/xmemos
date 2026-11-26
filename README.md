# Xmemos

[![License: AGPL v3](https://img.shields.io/badge/License-AGPL_v3-blue.svg)](LICENSE)

**Xmemos** is a privacy-first, 100% offline-first multi-platform (**Android**, **GNU/Linux**, **Windows 10/11**, and **macOS**) microblogging and knowledge-capture application built with **Tauri v2**, **Rust**, **SQLite (WAL mode)**, **SvelteKit 3 + Svelte 5**, and **Konsta UI v5 (`theme="ios"`) with Liquid Glass aesthetics**.

## Key Highlights

- **1:1 Official Memos SQLite Compatibility:** Uses the exact `memos_prod.db` schema of official FOSS [Memos](https://usememos.com/). Export your `.db` or `.xmemos` bundle and drop it directly into an official self-hosted Memos server, or import an official Memos SQLite database into Xmemos with zero data loss.
- **100% Offline Multi-User on One Device:** Full local `HOST`, `ADMIN`, and `USER` roles with official Memos-compatible `bcrypt` password hashes (`$2a$`/`$2b$`), strict `PRIVATE`/`PROTECTED`/`PUBLIC` visibility isolation, quick account switching, and optional per-user 6-digit PIN (+ Android `BiometricPrompt` lock on Android builds).
- **Adaptive Android & Desktop Architecture (`XMEMOS_TARGET_PLATFORM`):** Compile-time dead-code elimination ensures Android-only controls (`.nomedia` MediaStore toggle, `BiometricPrompt`) never appear in Desktop builds, while Desktop-only frameless window chrome and native Save dialogs are stripped from Android APK builds.
- **Cross-OS Native Memo Shortcuts (Android & Desktop):** Pin any memo directly to the **Android Home Screen** (native `ShortcutManagerCompat` widget-style adaptive Liquid Glass icon + dynamic long-press shortcuts) or to **GNU/Linux** (`.desktop`), **Windows 10/11** (`.lnk`), and **macOS** (`.app` launcher) Desktops for 1-tap direct opening.
- **Native Desktop File Manager Save & Open Dialogs:** On GNU/Linux (AppImage & Flatpak via `GtkFileChooserNative` + `xdg-desktop-portal`), Windows 10/11 (Win32 `comdlg32`), and macOS (native Finder sheet), downloading static memo attachments, share-card PNGs, and database/bundle backups opens the OS File Manager to pick the destination.
- **WhatsApp-Style Media Storage (`/Android/media/` on Android & Portable Mode on Desktop):** On Android, static media lives in `/storage/emulated/0/Android/media/org.xmemos.app/assets/` via `tauri-plugin-xmemos-storage` with SAF support and `.nomedia` toggle. On Desktop Portable builds (`portable.flag`), all databases, media, and cache live self-contained inside `./data/`.
- **Dual-Mode Atomic Backup & Restore:**
  - **Mode 1 (`.db`):** WAL-checkpointed, `VACUUM INTO` single-file SQLite snapshot.
  - **Mode 2 (`.xmemos`):** Standard ZIP archive bundling `memos_prod.db`, `assets/`, and SHA-256 `manifest.json` with atomic swap and automatic rollback on failure.
- **Konsta UI v5 iOS Liquid Glass UX, Native 2026 App Icon & WCAG 2.2 Accessibility:** Styled with Konsta UI v5 (`theme="ios"`) across Light and Dark modes (with zero-FOUC initialization), unified **macOS 2026 Liquid Glass** SVG & multi-OS icon suite (Android, GNU/Linux X11/Wayland scalable SVG/PNG, Windows ICO, macOS Retina ICNS), macOS 2026 Desktop Split-View, keyboard shortcuts (`Ctrl+/`), KaTeX math, PrismJS syntax highlighting, nested `#tags`, offline-cached OpenStreetMap `/map`, monthly activity calendar, and PNG share-card export.

## Supported Offline Release Packages

| Platform | Package Format | Offline & Compatibility Notes |
| :--- | :--- | :--- |
| **Android** (API 24–36) | `.apk` (`arm64-v8a`, `armeabi-v7a`, `x86_64`, `universal`) | Deterministic v2/v3 signed, 100% GMS-free, F-Droid friendly |
| **GNU/Linux (AppImage)** | `.AppImage` (`x86_64`, `aarch64`) | Built on Ubuntu 22.04 (`glibc 2.35`), stripped & lean (`bundleMediaFramework: false`); cross-distro executable |
| **GNU/Linux (Flatpak)** | `.flatpak` (`x86_64`, `aarch64`) | Sandboxed GNOME 51 single-file bundle (Portal file dialogs; `--share=network` strictly CSP-scoped to FOSS OpenStreetMap tiles/geocoding) |
| **Windows 10/11 (Installer)** | `-setup.exe` (`x64`) | 100% offline lean NSIS LZMA installer (~5–6 MB, `webviewInstallMode: skip`, 0 network calls; uses OS-preinstalled Windows 10/11 WebView2 Runtime) |
| **Windows 10/11 (Portable)** | `-portable.zip` (`x64`) | Extract & run `Xmemos.exe`; self-contained `./data/` via `portable.flag` |
| **macOS 12+** | `.dmg` (`arm64` Apple Silicon & `x64` Intel) | Native per-architecture disk images |

> **Debloated / Air-Gapped Windows (Microsoft Edge Removed):** Removing the Microsoft Edge browser does **not** affect Xmemos—it only uses the separate **WebView2 Runtime** preinstalled on Windows 10/11. If an aggressive debloater also removed WebView2 Runtime, Xmemos automatically detects a portable **WebView2 Fixed Version Runtime** placed in a `webview2\` folder next to `Xmemos.exe` (zero system install, 100% offline), or displays a guided Win32 dialog instead of crashing.

## Quick Start

```bash
# Install pinned dependencies
pnpm install --frozen-lockfile

# Verify version alignment, FOSS purity & license headers
pnpm version:check
pnpm license:check
pnpm foss:check

# Typecheck, test, and build frontend
pnpm check
pnpm test
pnpm build

# Run desktop Tauri dev app
pnpm tauri dev --release

# Run Rust integration & 1:1 SQLite compatibility tests
cd src-tauri && cargo test --locked && cargo clippy --locked --all-targets -- -D warnings

# Run Android dev build (requires Android SDK/NDK)
pnpm tauri android dev
```

## Manual Deterministic Multi-Platform Release

Releases are orchestrated by `.github/workflows/release.yml` (**manual `workflow_dispatch` only** — never on `push`, `pull_request`, or `schedule`), which invokes dedicated modular reusable workflows (`workflow_call`) per OS:

- `.github/workflows/android-release.yml` (Android APKs)
- `.github/workflows/linux-release.yml` (GNU/Linux AppImage & Flatpak)
- `.github/workflows/windows-release.yml` (Windows 10/11 Offline Installer & Portable)
- `.github/workflows/macos-release.yml` (macOS Apple Silicon & Intel DMG)

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

## Documentation

- [Architecture](docs/ARCHITECTURE.md)
- [Database Compatibility (1:1 Official Memos)](docs/DATABASE_COMPATIBILITY.md)
- [Local Development Setup](docs/DEVELOPMENT.md)
- [Reproducible Builds & Supply Chain Security](docs/REPRODUCIBLE_BUILDS.md)
- [Security Policy](SECURITY.md)
- [Contributing Guide](CONTRIBUTING.md)

## License

Copyright (C) 2026 Xmemos Contributors. Licensed under the **GNU Affero General Public License v3.0 or later** (`AGPL-3.0-or-later`). See [LICENSE](LICENSE).

## Multi-Language Support (i18n)

Xmemos v2.1.0+ ships with a strict, modular, zero-hardcode i18n architecture (`src/lib/i18n/`) supporting **English (`en`)**, **Bahasa Indonesia (`id`)**, and **Español (`es`)** with instant switching in **Settings → Preferences → Language**.
