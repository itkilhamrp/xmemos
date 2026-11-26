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

import { memoApi, type ListMemosParams } from '#lib/api/memo.ts';
import type { HeatmapDay, MemoLocation, MemoView, MemoVisibility, TagCount } from '#lib/types/memo.ts';
import { toLocalDateKey } from '#lib/utils/date.ts';
import { feedDisplayState } from './feed-display.svelte';
import { filterState } from './filters.svelte';

const PAGE_SIZE = 20;
const SNAPSHOT_PREFIX = 'xmemos.memo_snapshot.v1.';
const MAX_SNAPSHOT_ITEMS = 15;
const MAX_SNAPSHOT_BYTES = 128 * 1024;
const SNAPSHOT_TTL_MS = 7 * 24 * 60 * 60 * 1000;
const MAX_CACHED_USERS = 3;

interface MemoSnapshot {
	savedAt?: number;
	items: MemoView[];
	tags: TagCount[];
	heatmap: HeatmapDay[];
}

function pruneExcessUserSnapshots(keepKey: string): void {
	try {
		const entries: { key: string; savedAt: number }[] = [];
		for (let i = 0; i < localStorage.length; i += 1) {
			const k = localStorage.key(i);
			if (!k || !k.startsWith(SNAPSHOT_PREFIX) || k === keepKey) continue;
			let savedAt = 0;
			try {
				const parsed = JSON.parse(localStorage.getItem(k) ?? '{}') as Partial<MemoSnapshot>;
				savedAt = typeof parsed.savedAt === 'number' ? parsed.savedAt : 0;
			} catch {
				savedAt = 0;
			}
			entries.push({ key: k, savedAt });
		}
		if (entries.length >= MAX_CACHED_USERS) {
			entries.sort((a, b) => a.savedAt - b.savedAt);
			const removeCount = entries.length - (MAX_CACHED_USERS - 1);
			for (let i = 0; i < removeCount; i += 1) {
				const target = entries[i];
				if (target) localStorage.removeItem(target.key);
			}
		}
	} catch {
		// Ignore storage enumeration errors
	}
}

/**
 * Sorts memos in exact 1:1 parity with SQLite ORDER BY:
 * pinned DESC, {created_ts | updated_ts} {DESC | ASC}, id {DESC | ASC}
 */
function sortMemos(list: MemoView[]): MemoView[] {
	const useUpdated = feedDisplayState.orderBy === 'updated_ts';
	const asc = feedDisplayState.orderDirection === 'asc';
	return [...list].sort((a, b) => {
		if (a.memo.pinned !== b.memo.pinned) return a.memo.pinned ? -1 : 1;
		const aTs = useUpdated ? a.memo.updatedTs : a.memo.createdTs;
		const bTs = useUpdated ? b.memo.updatedTs : b.memo.createdTs;
		if (aTs !== bTs) return asc ? aTs - bTs : bTs - aTs;
		return asc ? a.memo.id - b.memo.id : b.memo.id - a.memo.id;
	});
}

class MemosState {
	items = $state<MemoView[]>([]);
	tags = $state<TagCount[]>([]);
	heatmap = $state<HeatmapDay[]>([]);
	loading = $state(false);
	loadingMore = $state(false);
	hasMore = $state(true);
	offset = $state(0);
	private hydratedForUser: number | null = null;
	private requestSeq = 0;

	hydrateSnapshot(creatorId: number | null): void {
		if (typeof window === 'undefined' || creatorId === null) return;
		if (feedDisplayState.hasNonDefaultSort) return;
		if (this.hydratedForUser === creatorId) return;
		const key = `${SNAPSHOT_PREFIX}${creatorId}`;
		try {
			const raw = localStorage.getItem(key);
			if (!raw) return;
			const snap = JSON.parse(raw) as Partial<MemoSnapshot>;
			if (typeof snap.savedAt === 'number' && Date.now() - snap.savedAt > SNAPSHOT_TTL_MS) {
				localStorage.removeItem(key);
				return;
			}
			if (Array.isArray(snap.items)) this.items = snap.items;
			if (Array.isArray(snap.tags)) this.tags = snap.tags;
			if (Array.isArray(snap.heatmap)) this.heatmap = snap.heatmap;
			this.hydratedForUser = creatorId;
		} catch {
			localStorage.removeItem(key);
		}
	}

	private persistSnapshot(creatorId?: number | null): void {
		if (typeof window === 'undefined' || creatorId === null || creatorId === undefined) return;
		if (filterState.hasActiveFilters || feedDisplayState.hasNonDefaultSort) return;
		const key = `${SNAPSHOT_PREFIX}${creatorId}`;
		try {
			pruneExcessUserSnapshots(key);
			let count = MAX_SNAPSHOT_ITEMS;
			let payload = '';
			while (count >= 3) {
				const snap: MemoSnapshot = {
					savedAt: Date.now(),
					items: this.items.slice(0, count),
					tags: this.tags.slice(0, 60),
					heatmap: this.heatmap.slice(-90)
				};
				payload = JSON.stringify(snap);
				if (payload.length <= MAX_SNAPSHOT_BYTES) break;
				count = Math.floor(count / 2);
			}
			if (payload && payload.length <= MAX_SNAPSHOT_BYTES) {
				localStorage.setItem(key, payload);
			}
		} catch {
			// Ignore storage quota errors
		}
	}

	async loadTimeline(creatorId?: number | null, reset = true): Promise<void> {
		if (!reset && (this.loading || this.loadingMore || !this.hasMore)) return;

		const seq = ++this.requestSeq;
		if (reset) {
			this.loading = true;
		} else {
			this.loadingMore = true;
		}

		try {
			const nextOffset = reset ? 0 : this.offset;
			const { startTs, endTs } = filterState.getDateBounds();

			const params: ListMemosParams = {
				creatorId: creatorId ?? null,
				rowStatus: 'NORMAL',
				tag: filterState.selectedTag,
				query: filterState.effectiveQuery || null,
				visibilityList: filterState.visibility ? [filterState.visibility] : undefined,
				hasTaskList: filterState.quickFilter === 'TASKS' ? true : undefined,
				hasMedia: filterState.quickFilter === 'MEDIA' ? true : undefined,
				hasLink: filterState.quickFilter === 'LINKS' ? true : undefined,
				hasCode: filterState.quickFilter === 'CODE' ? true : undefined,
				startTs,
				endTs,
				orderBy: feedDisplayState.orderBy,
				orderDirection: feedDisplayState.orderDirection,
				limit: PAGE_SIZE,
				offset: nextOffset
			};

			const batch = await memoApi.list(params);
			// Discard stale out-of-order responses
			if (seq !== this.requestSeq) return;

			if (reset) {
				this.items = batch;
			} else {
				const existingIds = new Set(this.items.map((m) => m.memo.id));
				const fresh = batch.filter((m) => !existingIds.has(m.memo.id));
				this.items = [...this.items, ...fresh];
			}
			this.offset = nextOffset + batch.length;
			this.hasMore = batch.length === PAGE_SIZE;
			if (reset) {
				this.persistSnapshot(creatorId);
			}
		} finally {
			if (seq === this.requestSeq) {
				this.loading = false;
				this.loadingMore = false;
			}
		}
	}

	async loadSidebarMetadata(creatorId?: number | null): Promise<void> {
		try {
			const [tags, heatmap] = await Promise.all([
				memoApi.listTags(creatorId),
				memoApi.getHeatmap(creatorId, 0)
			]);
			this.tags = tags;
			this.heatmap = heatmap;
			this.persistSnapshot(creatorId);
		} catch {
			// Ignore if uninitialized
		}
	}

	async createMemo(payload: {
		content: string;
		visibility: MemoVisibility;
		pinned?: boolean;
		createdTs?: number | null;
		location?: MemoLocation | null;
		resourceIds?: number[];
		relationIds?: number[];
	}): Promise<MemoView> {
		const created = await memoApi.create(payload);
		const matchesDateFilter =
			!filterState.selectedDate ||
			toLocalDateKey(new Date(created.memo.createdTs * 1000)) === filterState.selectedDate;
		if (matchesDateFilter) {
			this.items = sortMemos([created, ...this.items.filter((v) => v.memo.id !== created.memo.id)]);
			this.offset += 1;
		}
		await this.loadSidebarMetadata(created.memo.creatorId);
		return created;
	}

	async togglePin(view: MemoView): Promise<void> {
		const updated = await memoApi.update({
			id: view.memo.id,
			pinned: !view.memo.pinned
		});
		this.replaceItem(updated);
	}

	async toggleTask(memoId: number, taskIndex: number): Promise<void> {
		const updated = await memoApi.toggleTask(memoId, taskIndex);
		this.replaceItem(updated);
	}

	async archiveMemo(memoId: number): Promise<void> {
		await memoApi.update({ id: memoId, rowStatus: 'ARCHIVED' });
		this.items = this.items.filter((v) => v.memo.id !== memoId);
	}

	async deleteMemo(memoId: number): Promise<void> {
		await memoApi.delete(memoId);
		this.items = this.items.filter((v) => v.memo.id !== memoId);
	}

	async toggleReaction(view: MemoView, emoji: string): Promise<void> {
		const reactions = await memoApi.toggleReaction(view.memo.uid, emoji);
		this.replaceItem({ ...view, reactions });
	}

	replaceItem(updated: MemoView): void {
		const exists = this.items.some((v) => v.memo.id === updated.memo.id);
		if (!exists) return;
		const matchesDateFilter =
			!filterState.selectedDate ||
			toLocalDateKey(new Date(updated.memo.createdTs * 1000)) === filterState.selectedDate;
		if (!matchesDateFilter) {
			this.items = this.items.filter((v) => v.memo.id !== updated.memo.id);
			return;
		}
		this.items = sortMemos(this.items.map((v) => (v.memo.id === updated.memo.id ? updated : v)));
	}
}

export const memosState = new MemosState();
