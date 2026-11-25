# Deterministic & Reproducible Multi-Platform Builds

Xmemos is engineered to produce deterministic, auditable, and supply-chain-hardened release builds across **Android**, **GNU/Linux (AppImage & Flatpak)**, **Windows 10/11 (Offline Installer & Portable)**, and **macOS**.

## Determinism & Supply-Chain Controls

1. **Unified Pinned Timestamp:** Resolved once in the `preflight` job (`SOURCE_DATE_EPOCH=$(git log -1 --format=%ct)`) and propagated identically to all platform build jobs.
2. **Pinned Toolchains & Locked Dependencies:** `rust-toolchain.toml` (`1.100.0`), `.node-version`, `pnpm-lock.yaml` (`pnpm install --frozen-lockfile`), `src-tauri/Cargo.lock` (`cargo --locked`), and pinned Android NDK/Build-Tools.
3. **Path Remapping & LTO (`src-tauri/Cargo.toml`):**
   - `CARGO_INCREMENTAL=0`
   - `RUSTFLAGS="--remap-path-prefix=$HOME=/home --remap-path-prefix=$PWD=/workspace -C debuginfo=0"`
   - `opt-level = 3`, `lto = true`, `codegen-units = 1`, `panic = "abort"`, `strip = true`
4. **Adaptive Compile-Time Separation:**
   - `XMEMOS_TARGET_PLATFORM=android` strips Desktop window chrome and desktop dialog helpers from Android APKs.
   - `XMEMOS_TARGET_PLATFORM=desktop` strips Android `.nomedia` and `BiometricPrompt` UI from Desktop bundles.
5. **Android Reproducible APK Hygiene:**
   - `dependenciesInfo { includeInApk = false; includeInBundle = false }`
   - `isPreserveFileTimestamps = false` & `isReproducibleFileOrder = true`
   - `zipalign -p -f -v 4` followed by `apksigner sign --v1-signing-enabled false --v2-signing-enabled true --v3-signing-enabled true` (omitting non-deterministic v1 JAR timestamps).
6. **Desktop Offline Packaging Hygiene:**
   - **GNU/Linux AppImage (`x86_64` & `aarch64`):** Built inside official `ubuntu:22.04` (`glibc 2.35`) container with stripped binaries and lean packaging (`bundleMediaFramework: false`, avoiding ~60 MB GStreamer/FFmpeg bloat; for ultra-compact ~5 MB Linux installs, use the Flatpak bundle).
   - **GNU/Linux Flatpak (`x86_64` & `aarch64`):** Built with official `org.flatpak.Builder` on `org.gnome.Platform//51` with strict offline sandboxing (no `--share=network`, no `--filesystem=host`) and exported as a single-file `.flatpak` bundle.
   - **Windows 10/11 (`x64`):** 100% offline lean NSIS LZMA installer (`webviewInstallMode.type = "skip"`, ~5–6 MB, zero network requests during install or runtime, using the OS-preinstalled WebView2 Runtime—independent of the Edge browser) plus a deterministic `SOURCE_DATE_EPOCH`-normalized Portable `.zip` (`scripts/package-windows-portable.py`). At startup (`src-tauri/src/main.rs`), Xmemos auto-detects a local portable `webview2\` Fixed Version Runtime folder next to `Xmemos.exe` for debloated/air-gapped Windows PCs and shows a clean native Win32 dialog if WebView2 Runtime is completely absent.
   - **macOS (`arm64` & `x64`):** Per-architecture `.dmg` images targeting macOS 12.0+.
7. **Supply Chain Security & SLSA Build Provenance:**
   - Modular reusable workflows: `release.yml` (`workflow_dispatch` only) orchestrates `android-release.yml`, `linux-release.yml`, `windows-release.yml`, and `macos-release.yml` (`workflow_call` only) with strict top-level `permissions: contents: read`, `persist-credentials: false`, and explicit per-platform secret isolation (zero `secrets: inherit`).
   - Latest 2026 hardened GitHub Actions (`actions/checkout@v7`, `actions/setup-node@v7`, `actions/setup-java@v6`, `actions/upload-artifact@v7`, `actions/download-artifact@v8`, `pnpm/action-setup@v6`, `android-actions/setup-android@v4`, `gradle/actions/setup-gradle@v6` with open-source `cache-provider: basic`).
   - Cryptographic SLSA Build Provenance via `actions/attest-build-provenance@v4` (Sigstore OIDC) on public repositories.

## Independent Verification

```bash
# Verify two consecutive clean Android builds produce identical hashes
./scripts/verify-repro.sh aarch64

# Verify SHA-256 checksums of any downloaded platform package
sha256sum -c --ignore-missing Xmemos-v2.7.22-SHA256SUMS.txt

# Verify Android APK signature & certificate fingerprints
apksigner verify --verbose --print-certs Xmemos-v2.7.22-android-arm64-v8a-release.apk

# Verify GitHub SLSA Build Provenance attestation (public repositories)
gh attestation verify Xmemos-v2.7.22-linux-x86_64.AppImage --repo <OWNER>/<REPO>
```
