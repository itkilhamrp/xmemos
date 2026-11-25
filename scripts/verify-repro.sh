#!/usr/bin/env bash
# Copyright (C) 2026 Xmemos Contributors
#
# This file is part of Xmemos.
#
# Xmemos is free software: you can redistribute it and/or modify
# it under the terms of the GNU Affero General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
#
# Xmemos is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
# GNU Affero General Public License for more details.
#
# You should have received a copy of the GNU Affero General Public License
# along with Xmemos. If not, see https://www.gnu.org/licenses/.
#
# SPDX-License-Identifier: AGPL-3.0-or-later

set -euo pipefail

ABI="${1:-aarch64}"
BUILD_DIR_A="/tmp/xmemos-repro-a"
BUILD_DIR_B="/tmp/xmemos-repro-b"

export SOURCE_DATE_EPOCH="${SOURCE_DATE_EPOCH:-$(git log -1 --format=%ct)}"
export CARGO_INCREMENTAL=0
export RUSTFLAGS="--remap-path-prefix=$HOME=/home --remap-path-prefix=$PWD=/workspace -C debuginfo=0"

echo "Verifying deterministic Android APK build for target: ${ABI}"
echo "SOURCE_DATE_EPOCH=${SOURCE_DATE_EPOCH}"

rm -rf "${BUILD_DIR_A}" "${BUILD_DIR_B}"
mkdir -p "${BUILD_DIR_A}" "${BUILD_DIR_B}"

build_once() {
  local out_dir="$1"
  rm -rf build src-tauri/target/release src-tauri/gen/android/app/build
  pnpm build
  pnpm tauri android build --apk --target "${ABI}"
  local apk
  apk="$(find src-tauri/gen/android/app/build/outputs/apk -type f \( -name '*release-unsigned.apk' -o -name '*release.apk' \) | sort | head -n 1)"
  if [[ -z "${apk}" ]]; then
    echo "ERROR: No APK produced during build." >&2
    exit 1
  fi
  cp "${apk}" "${out_dir}/xmemos.apk"
}

echo "--- Pass 1 ---"
build_once "${BUILD_DIR_A}"
HASH_A="$(sha256sum "${BUILD_DIR_A}/xmemos.apk" | awk '{print $1}')"

echo "--- Pass 2 ---"
build_once "${BUILD_DIR_B}"
HASH_B="$(sha256sum "${BUILD_DIR_B}/xmemos.apk" | awk '{print $1}')"

echo "Pass 1 SHA-256: ${HASH_A}"
echo "Pass 2 SHA-256: ${HASH_B}"

if [[ "${HASH_A}" != "${HASH_B}" ]]; then
  echo "ERROR: Reproducibility check FAILED! Hashes differ." >&2
  if command -v diffoscope >/dev/null 2>&1; then
    diffoscope "${BUILD_DIR_A}/xmemos.apk" "${BUILD_DIR_B}/xmemos.apk" || true
  fi
  exit 1
fi

echo "SUCCESS: Deterministic 1:1 identical APK hashes verified!"
