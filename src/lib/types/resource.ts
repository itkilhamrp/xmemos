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

export interface Resource {
	id: number;
	uid: string;
	creatorId: number;
	createdTs: number;
	updatedTs: number;
	filename: string;
	type: string;
	size: number;
	memoId: number | null;
	storageType: string;
	reference: string;
	payload: string;
	hasBlob: boolean;
}

export interface StorageBreakdown {
	mediaRoot: string;
	memosDbBytes: number;
	localDbBytes: number;
	assetsBytes: number;
	backupsBytes: number;
	thumbsBytes: number;
	resourceCount: number;
	blobResourceCount: number;
	orphanFileCount: number;
	nomediaEnabled: boolean;
}

export interface CleanupResult {
	removedFiles: number;
	freedBytes: number;
}

export interface OrphanScanSummary {
	unlinkedResourceCount: number;
	orphanDiskFileCount: number;
	totalOrphanCount: number;
	reclaimableBytes: number;
	sampleFilenames: string[];
}
