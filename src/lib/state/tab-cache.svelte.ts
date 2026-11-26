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

import type { MemoView } from '#lib/types/memo.ts';
import type { Resource } from '#lib/types/resource.ts';

/**
 * Lightweight, bounded in-memory SWR cache for primary tab routes (/explore, /review, /resources).
 * Eliminates skeleton flashes and IPC wait delays on tab switches while revalidating silently in the
 * background. Scoped strictly by active userId so account switches never leak stale data.
 */
class TabCacheState {
	private ownerId: number | null = null;
	private exploreList: MemoView[] | null = null;
	private reviewList: MemoView[] | null = null;
	private resourceList: Resource[] | null = null;

	private syncOwner(userId: number | null): void {
		if (this.ownerId !== userId) {
			this.ownerId = userId;
			this.exploreList = null;
			this.reviewList = null;
			this.resourceList = null;
		}
	}

	getExplore(userId: number | null): MemoView[] | null {
		this.syncOwner(userId);
		return this.exploreList;
	}

	setExplore(userId: number | null, items: MemoView[]): void {
		this.syncOwner(userId);
		this.exploreList = items.slice(0, 60);
	}

	getReview(userId: number | null): MemoView[] | null {
		this.syncOwner(userId);
		return this.reviewList;
	}

	setReview(userId: number | null, items: MemoView[]): void {
		this.syncOwner(userId);
		this.reviewList = items.slice(0, 500);
	}

	getResources(userId: number | null): Resource[] | null {
		this.syncOwner(userId);
		return this.resourceList;
	}

	setResources(userId: number | null, items: Resource[]): void {
		this.syncOwner(userId);
		this.resourceList = items.slice(0, 200);
	}

	clear(): void {
		this.ownerId = null;
		this.exploreList = null;
		this.reviewList = null;
		this.resourceList = null;
	}
}

export const tabCacheState = new TabCacheState();
