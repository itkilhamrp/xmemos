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

# Generates a permanent, 10,000-day RSA-4096 PKCS12 Android release signing keystore
# so every future Xmemos APK update installs cleanly in-place without package conflicts.

KEYSTORE_FILE="${1:-xmemos-release.keystore}"
KEY_ALIAS="${2:-xmemos-release}"

if [ -f "${KEYSTORE_FILE}" ]; then
  echo "ERROR: ${KEYSTORE_FILE} already exists. Refusing to overwrite an existing signing key!" >&2
  echo "Keep your original keystore safe — replacing it will prevent in-place Android updates." >&2
  exit 1
fi

if ! command -v keytool >/dev/null 2>&1; then
  echo "ERROR: 'keytool' (JDK 17+) is required. Install OpenJDK 17 first." >&2
  exit 1
fi

read -r -s -p "Enter new keystore password (min 8 chars): " STORE_PASS
echo ""
read -r -s -p "Confirm keystore password: " STORE_PASS_CONFIRM
echo ""

if [ "${STORE_PASS}" != "${STORE_PASS_CONFIRM}" ]; then
  echo "ERROR: Passwords do not match." >&2
  exit 1
fi
if [ "${#STORE_PASS}" -lt 8 ]; then
  echo "ERROR: Password must be at least 8 characters." >&2
  exit 1
fi

keytool -genkeypair -v \
  -storetype PKCS12 \
  -keystore "${KEYSTORE_FILE}" \
  -alias "${KEY_ALIAS}" \
  -keyalg RSA \
  -keysize 4096 \
  -validity 10000 \
  -storepass "${STORE_PASS}" \
  -keypass "${STORE_PASS}" \
  -dname "CN=Xmemos Release, O=Xmemos, C=ID"

chmod 600 "${KEYSTORE_FILE}"

B64_FILE="${KEYSTORE_FILE}.base64.txt"
base64 "${KEYSTORE_FILE}" | tr -d '\r\n' > "${B64_FILE}"
chmod 600 "${B64_FILE}"

echo ""
echo "========================================================================"
echo "Permanent Android Release Keystore created: ${KEYSTORE_FILE}"
echo "Single-line Base64 saved to: ${B64_FILE}"
echo "========================================================================"
echo "Add these 4 GitHub Repository Secrets (Settings -> Secrets -> Actions):"
echo "  1. ANDROID_KEYSTORE_BASE64   = contents of ${B64_FILE}"
echo "  2. ANDROID_KEYSTORE_PASSWORD = <your password>"
echo "  3. ANDROID_KEY_ALIAS         = ${KEY_ALIAS}"
echo "  4. ANDROID_KEY_PASSWORD      = <your password>"
echo ""
echo "Certificate Fingerprint (back up ${KEYSTORE_FILE} offline safely!):"
keytool -list -v -keystore "${KEYSTORE_FILE}" -alias "${KEY_ALIAS}" -storepass "${STORE_PASS}" | grep -E "SHA256:|SHA1:" || true
echo ""
echo "IMPORTANT: Never commit ${KEYSTORE_FILE} or ${B64_FILE} to Git!"
