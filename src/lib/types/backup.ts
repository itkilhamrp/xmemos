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

export type BackupScope = 'ALL_USERS' | 'ACTIVE_USER';

export interface ExportResult {
	mode: 'DB_ONLY' | 'FULL_BUNDLE';
	scope: BackupScope;
	filename: string;
	path: string;
	sizeBytes: number;
	sha256: string;
	mediaCount: number;
}

export interface ImportPreview {
	sourcePath: string;
	detectedFormat: 'SQLITE_DB' | 'XMEMOS_BUNDLE';
	userCount: number;
	memoCount: number;
	resourceCount: number;
	mediaFileCount: number;
	archiveSizeBytes: number;
	checksumVerified: boolean;
	compatibilityNotes: string[];
}

export interface ImportResult {
	userCount: number;
	memoCount: number;
	resourceCount: number;
	restoredMediaFiles: number;
	ftsIndexedMemos: number;
	compatibilityNotes: string[];
}

export interface BackupHistoryEntry {
	id: number;
	mode: 'DB_ONLY' | 'FULL_BUNDLE';
	filename: string;
	path: string;
	sizeBytes: number;
	sha256: string;
	createdTs: number;
}
