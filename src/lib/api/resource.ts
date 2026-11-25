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

import { callIpc } from './invoke';
import type { CleanupResult, OrphanScanSummary, Resource, StorageBreakdown } from '#lib/types/resource.ts';

export interface PickDocumentResponse {
	cancelled: boolean;
	localPath?: string | null;
	filename?: string | null;
	mimeType?: string | null;
	size: number;
}

export interface NomediaStatus {
	enabled: boolean;
	path: string;
}

export const resourceApi = {
	uploadBytes: (payload: {
		filename: string;
		mimeType?: string;
		base64Data: string;
		memoId?: number | null;
	}): Promise<Resource> => callIpc<Resource>('resource_upload_bytes', { payload }),

	importFile: (payload: {
		localPath: string;
		filename: string;
		mimeType?: string;
		memoId?: number | null;
	}): Promise<Resource> => callIpc<Resource>('resource_import_file', { payload }),

	list: (): Promise<Resource[]> => callIpc<Resource[]>('resource_list'),

	delete: (resourceId: number): Promise<void> =>
		callIpc<void>('resource_delete', { resourceId }),

	storageBreakdown: (): Promise<StorageBreakdown> =>
		callIpc<StorageBreakdown>('resource_storage_breakdown'),

	scanOrphans: (): Promise<OrphanScanSummary> => callIpc<OrphanScanSummary>('resource_scan_orphans'),

	cleanOrphans: (): Promise<CleanupResult> => callIpc<CleanupResult>('resource_clean_orphans'),

	externalizeBlobs: (): Promise<number> => callIpc<number>('resource_externalize_blobs'),

	/**
	 * Exports a memo attachment/static file.
	 * On Desktop (GNU/Linux, Windows, macOS), opens the native OS File Manager Save dialog
	 * and returns `null` if the user cancels. On Android, saves directly to Downloads.
	 */
	exportToDownloads: (uid: string): Promise<string | null> =>
		callIpc<string | null>('resource_export_to_downloads', { uid }),

	/**
	 * Saves arbitrary binary data (such as rendered share cards or unsaved blobs).
	 * On Desktop, opens the native OS File Manager Save dialog and returns `null` if cancelled.
	 */
	saveBytes: (payload: {
		filename: string;
		mimeType?: string;
		base64Data: string;
	}): Promise<string | null> => callIpc<string | null>('resource_save_bytes', { payload }),

	pickDocument: (mimeTypes: string[], copyToTemp = true): Promise<PickDocumentResponse> =>
		callIpc<PickDocumentResponse>('plugin:xmemos-storage|pick_document', {
			payload: { mimeTypes, copyToTemp }
		}),

	setNomedia: (enabled: boolean): Promise<NomediaStatus> =>
		callIpc<NomediaStatus>('plugin:xmemos-storage|set_nomedia', { payload: { enabled } }),

	getNomediaStatus: (): Promise<NomediaStatus> =>
		callIpc<NomediaStatus>('plugin:xmemos-storage|get_nomedia_status')
};
