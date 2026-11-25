# Local Development Guide

## Prerequisites

- **Node.js**: `>= 22.17` (see `.node-version`)
- **pnpm**: `12.x` (`corepack enable`)
- **Rust**: Pinned in `rust-toolchain.toml` (`1.100.0` + Android targets `aarch64-linux-android`, `armv7-linux-androideabi`, `x86_64-linux-android`)
- **Java**: Temurin JDK 17
- **Android SDK & NDK**: API 36 Platform, Build-Tools `35.0.0`, NDK `27.2.12479018`
- **Linux Desktop Dev Libraries** (Debian/Ubuntu):
  ```bash
  sudo apt-get install -y pkg-config libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev libssl-dev
  ```

## Environment Variables (Android)

```bash
export JAVA_HOME=/usr/lib/jvm/temurin-17-jdk-amd64
export ANDROID_HOME="$HOME/Android/Sdk"
export NDK_HOME="$ANDROID_HOME/ndk/27.2.12479018"
export PATH="$ANDROID_HOME/platform-tools:$ANDROID_HOME/build-tools/35.0.0:$PATH"
```

## Common Commands

```bash
# Install dependencies
pnpm install --frozen-lockfile

# Initialize Android Gradle project once
pnpm tauri android init

# Run live reload on connected Android device or emulator
pnpm tauri android dev

# Inspect Android WebView via Chromium DevTools
# Open chrome://inspect/#devices in Chrome/Brave on your host machine

# Run full validation suite
pnpm version:check && pnpm license:check && pnpm check && pnpm test && pnpm build
cd src-tauri && cargo clippy --all-targets -- -D warnings && cargo test
```
