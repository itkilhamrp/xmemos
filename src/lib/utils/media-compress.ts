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

const MAX_EDGE_PX = 2048;
const WEBP_QUALITY = 0.86;
const MIN_COMPRESS_THRESHOLD_BYTES = 128 * 1024;

export interface PreparedUpload {
	filename: string;
	mimeType: string;
	base64Data: string;
}

function arrayBufferToBase64(buf: ArrayBuffer): string {
	const bytes = new Uint8Array(buf);
	const chunks: string[] = [];
	const chunkSize = 0x8000;
	for (let i = 0; i < bytes.length; i += chunkSize) {
		chunks.push(String.fromCharCode(...bytes.subarray(i, i + chunkSize)));
	}
	return btoa(chunks.join(''));
}

function replaceExt(name: string, ext: string): string {
	const dot = name.lastIndexOf('.');
	const stem = dot > 0 ? name.slice(0, dot) : name;
	return `${stem}.${ext}`;
}

/**
 * Uses the Android/Chromium native WebP encoder (libwebp) via Canvas when uploading
 * large static photos from the WebView, keeping only if strictly smaller than original.
 */
export async function prepareSmartUpload(file: File): Promise<PreparedUpload> {
	const origBuf = await file.arrayBuffer();
	const isRasterPhoto =
		file.type === 'image/jpeg' ||
		file.type === 'image/png' ||
		file.type === 'image/webp' ||
		file.type === 'image/bmp';

	if (!isRasterPhoto || file.size < MIN_COMPRESS_THRESHOLD_BYTES || typeof createImageBitmap === 'undefined') {
		return {
			filename: file.name,
			mimeType: file.type || 'application/octet-stream',
			base64Data: arrayBufferToBase64(origBuf)
		};
	}

	try {
		const bitmap = await createImageBitmap(file);
		const scale = Math.min(1, MAX_EDGE_PX / Math.max(bitmap.width, bitmap.height));
		const targetW = Math.max(1, Math.round(bitmap.width * scale));
		const targetH = Math.max(1, Math.round(bitmap.height * scale));

		const canvas = document.createElement('canvas');
		canvas.width = targetW;
		canvas.height = targetH;
		const ctx = canvas.getContext('2d');
		if (!ctx) {
			bitmap.close();
			return {
				filename: file.name,
				mimeType: file.type,
				base64Data: arrayBufferToBase64(origBuf)
			};
		}

		ctx.imageSmoothingEnabled = true;
		ctx.imageSmoothingQuality = 'high';
		ctx.drawImage(bitmap, 0, 0, targetW, targetH);
		bitmap.close();

		const webpBlob = await new Promise<Blob | null>((resolve) =>
			canvas.toBlob((b) => resolve(b), 'image/webp', WEBP_QUALITY)
		);

		if (webpBlob && webpBlob.size > 0 && webpBlob.size < file.size) {
			const webpBuf = await webpBlob.arrayBuffer();
			return {
				filename: replaceExt(file.name, 'webp'),
				mimeType: 'image/webp',
				base64Data: arrayBufferToBase64(webpBuf)
			};
		}
	} catch {
		// Fall back safely to Rust backend compression
	}

	return {
		filename: file.name,
		mimeType: file.type || 'application/octet-stream',
		base64Data: arrayBufferToBase64(origBuf)
	};
}
