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
import type { BackupHistoryEntry, BackupScope, ExportResult, ImportPreview, ImportResult } from '#lib/types/backup.ts';

export interface SaveDocumentResponse {
	cancelled: boolean;
	savedUri?: string | null;
	bytesWritten: number;
}

export const backupApi = {
	exportDb: (scope: BackupScope = 'ACTIVE_USER'): Promise<ExportResult> =>
		callIpc<ExportResult>('backup_export_db', { scope }),

	exportFull: (scope: BackupScope = 'ACTIVE_USER'): Promise<ExportResult> =>
		callIpc<ExportResult>('backup_export_full', { scope }),

	finalizeExport: (
		stagedPath: string,
		savedDestination: string | null,
		result: ExportResult | null
	): Promise<void> =>
		callIpc<void>('backup_finalize_export', {
			stagedPath,
			savedDestination,
			result
		}),

	listHistory: (): Promise<BackupHistoryEntry[]> =>
		callIpc<BackupHistoryEntry[]>('backup_list_history'),

	previewImport: (sourcePath: string): Promise<ImportPreview> =>
		callIpc<ImportPreview>('backup_preview_import', { sourcePath }),

	executeImport: (sourcePath: string): Promise<ImportResult> =>
		callIpc<ImportResult>('backup_execute_import', { sourcePath }),

	saveToSaf: (
		sourcePath: string,
		suggestedFilename: string,
		mimeType: string
	): Promise<SaveDocumentResponse> =>
		callIpc<SaveDocumentResponse>('plugin:xmemos-storage|save_document', {
			payload: { sourcePath, suggestedFilename, mimeType }
		})
};
