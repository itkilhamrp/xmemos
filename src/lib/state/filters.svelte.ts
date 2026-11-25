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

import type { MemoVisibility } from '#lib/types/memo.ts';
import { getLocalDayBounds } from '#lib/utils/date.ts';

export type QuickFilterType = 'ALL' | 'TASKS' | 'MEDIA' | 'LINKS' | 'CODE';

class FilterState {
	query = $state('');
	searchTerms = $state<string[]>([]);
	selectedTag = $state<string | null>(null);
	selectedDate = $state<string | null>(null);
	quickFilter = $state<QuickFilterType>('ALL');
	visibility = $state<MemoVisibility | null>(null);

	activeHighlightTerms = $derived.by((): string[] => {
		const list = [...this.searchTerms];
		const live = this.query.trim();
		if (live && !list.some((t) => t.toLowerCase() === live.toLowerCase())) {
			list.push(live);
		}
		return list;
	});

	effectiveQuery = $derived(this.activeHighlightTerms.join(' '));

	hasActiveFilters = $derived(
		this.query.trim().length > 0 ||
			this.searchTerms.length > 0 ||
			this.selectedTag !== null ||
			this.selectedDate !== null ||
			this.quickFilter !== 'ALL' ||
			this.visibility !== null
	);

	commitQuery(): boolean {
		const clean = this.query.trim();
		if (!clean) return false;
		if (!this.searchTerms.some((t) => t.toLowerCase() === clean.toLowerCase())) {
			this.searchTerms = [...this.searchTerms, clean];
		}
		this.query = '';
		return true;
	}

	removeSearchTerm(term: string): void {
		this.searchTerms = this.searchTerms.filter((t) => t !== term);
	}

	reset(): void {
		this.query = '';
		this.searchTerms = [];
		this.selectedTag = null;
		this.selectedDate = null;
		this.quickFilter = 'ALL';
		this.visibility = null;
	}

	getDateBounds(): { startTs: number | null; endTs: number | null } {
		if (!this.selectedDate) return { startTs: null, endTs: null };
		return getLocalDayBounds(this.selectedDate);
	}
}

export const filterState = new FilterState();
