# F-Droid & IzzyOnDroid Distribution Guide

Xmemos is engineered to comply 100% with the [F-Droid Inclusion Policy](https://f-droid.org/docs/Inclusion_Policy/) and Reproducible Builds standard:

- **License:** `AGPL-3.0-or-later` (verified via `pnpm license:check`)
- **Zero Proprietary Dependencies:** 0 GMS, 0 Firebase, 0 Play Services, 0 trackers/analytics (verified via `pnpm foss:check`)
- **Zero Anti-Features:** 100% offline-first (`No NonFreeNet`, `No NonFreeDep`, `No Tracking`)
- **No Google Dependency Info Block:** `dependenciesInfo { includeInApk = false; includeInBundle = false }`
- **Fastlane Metadata:** Included in `fastlane/metadata/android/en-US/`
- **Per-ABI Split APKs:** `arm64-v8a`, `armeabi-v7a`, `x86_64` keep APK sizes compact for F-Droid and IzzyOnDroid

## Reference `fdroiddata` Metadata (`metadata/org.xmemos.app.yml`)

```yaml
Categories:
  - Writing
  - Productivity
License: AGPL-3.0-or-later
AuthorName: Xmemos Contributors
SourceCode: https://github.com/your-org/xmemos
IssueTracker: https://github.com/your-org/xmemos/issues
Changelog: https://github.com/your-org/xmemos/blob/main/CHANGELOG.md

AutoName: Xmemos

RepoType: git
Repo: https://github.com/your-org/xmemos.git
Binaries: https://github.com/your-org/xmemos/releases/download/v%v/Xmemos-v%v-android-arm64-v8a-release.apk

Builds:
  - versionName: 1.0.0
    versionCode: 1000000
    commit: v1.0.0
    subdir: src-tauri/gen/android
    sudo:
      - apt-get update
      - apt-get install -y curl pkg-config libssl-dev
    prebuild:
      - cd ../.. && pnpm install --frozen-lockfile
      - cd ../.. && pnpm build
      - cd ../.. && pnpm tauri android init --ci && pnpm tauri icon src-tauri/icons/icon.png
      - cd ../.. && pnpm version:sync
    build:
      - export CARGO_INCREMENTAL=0
      - export RUSTFLAGS="--remap-path-prefix=$HOME=/home --remap-path-prefix=$PWD=/workspace -C debuginfo=0"
      - cd ../.. && pnpm tauri android build --apk --target aarch64
    output: app/build/outputs/apk/universal/release/app-universal-release-unsigned.apk

AutoUpdateMode: Version
UpdateCheckMode: Tags
CurrentVersion: 1.0.0
CurrentVersionCode: 1000000
```
