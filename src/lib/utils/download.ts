// Copyright (C) 2026 Xmemos Contributors
//
// This file is part of Xmemos.
//
// Xmemos is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Xmemos is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with Xmemos. If not, see https://www.gnu.org/licenses/.
//
// SPDX-License-Identifier: AGPL-3.0-or-later

import { isTauri } from '@tauri-apps/api/core';
import { resourceApi } from '#lib/api/resource.ts';

function uint8ArrayToBase64(bytes: Uint8Array): string {
	let binary = '';
	const chunkSize = 0x8000;
	for (let i = 0; i < bytes.length; i += chunkSize) {
		const chunk = bytes.subarray(i, i + chunkSize);
		binary += String.fromCharCode(...chunk);
	}
	return btoa(binary);
}

/**
 * Extracts a clean display name from a saved file path or filename for toast notifications.
 */
export function formatSavedLabel(savedPath: string, fallback: string): string {
	const trimmed = savedPath.trim();
	if (!trimmed) return fallback;
	const parts = trimmed.split(/[\\/]/);
	return parts[parts.length - 1] || trimmed;
}

/**
 * Saves an in-memory Blob cleanly:
 * - Inside Tauri Desktop (GNU/Linux, Windows, macOS): opens the native OS File Manager Save dialog
 *   and returns `null` if the user cancels.
 * - Inside Tauri Android: saves to the OS Downloads folder.
 * - Inside a plain web browser preview: falls back to an anchor download.
 */
export async function saveBlobWithNativeDialog(
	blob: Blob,
	filename: string,
	mimeType?: string
): Promise<string | null> {
	if (isTauri()) {
		const buf = await blob.arrayBuffer();
		const base64Data = uint8ArrayToBase64(new Uint8Array(buf));
		return resourceApi.saveBytes({
			filename,
			mimeType: mimeType || blob.type || 'application/octet-stream',
			base64Data
		});
	}

	const objUrl = URL.createObjectURL(blob);
	const a = document.createElement('a');
	a.href = objUrl;
	a.download = filename;
	document.body.appendChild(a);
	a.click();
	document.body.removeChild(a);
	setTimeout(() => URL.revokeObjectURL(objUrl), 2000);
	return filename;
}
