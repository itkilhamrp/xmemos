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

export type FeedColumnLayout = 'list' | '2' | '3' | 'auto';
export type FeedOrderBy = 'created_ts' | 'updated_ts';
export type FeedOrderDirection = 'desc' | 'asc';

export interface DisplayPopoverAnchor {
	placement: 'right' | 'below';
	rect: { top: number; left: number; bottom: number; right: number; width: number; height: number };
}

const STORAGE_KEY = 'xmemos.feed_display.v1';

interface StoredFeedDisplay {
	layout?: FeedColumnLayout;
	orderBy?: FeedOrderBy;
	orderDirection?: FeedOrderDirection;
	fullWidth?: boolean;
}

class FeedDisplayState {
	layout = $state<FeedColumnLayout>('list');
	orderBy = $state<FeedOrderBy>('created_ts');
	orderDirection = $state<FeedOrderDirection>('desc');
	fullWidth = $state<boolean>(false);
	popoverAnchor = $state<DisplayPopoverAnchor | null>(null);

	constructor() {
		if (typeof window !== 'undefined') {
			try {
				const raw = localStorage.getItem(STORAGE_KEY);
				if (raw) {
					const parsed = JSON.parse(raw) as StoredFeedDisplay;
					if (
						parsed.layout === 'list' ||
						parsed.layout === '2' ||
						parsed.layout === '3' ||
						parsed.layout === 'auto'
					) {
						this.layout = parsed.layout;
					}
					if (parsed.orderBy === 'created_ts' || parsed.orderBy === 'updated_ts') {
						this.orderBy = parsed.orderBy;
					}
					if (parsed.orderDirection === 'desc' || parsed.orderDirection === 'asc') {
						this.orderDirection = parsed.orderDirection;
					}
					if (typeof parsed.fullWidth === 'boolean') {
						this.fullWidth = parsed.fullWidth;
					}
				}
			} catch {
				// Ignore corrupted local preference
			}
		}
	}

	private persist(): void {
		if (typeof window === 'undefined') return;
		try {
			const data: StoredFeedDisplay = {
				layout: this.layout,
				orderBy: this.orderBy,
				orderDirection: this.orderDirection,
				fullWidth: this.fullWidth
			};
			localStorage.setItem(STORAGE_KEY, JSON.stringify(data));
		} catch {
			// Ignore storage quota errors
		}
	}

	setLayout(next: FeedColumnLayout): void {
		this.layout = next;
		this.persist();
	}

	setOrderBy(next: FeedOrderBy): void {
		this.orderBy = next;
		this.persist();
	}

	setOrderDirection(next: FeedOrderDirection): void {
		this.orderDirection = next;
		this.persist();
	}

	setFullWidth(next: boolean): void {
		this.fullWidth = next;
		this.persist();
	}

	togglePopover(rect: DOMRect, placement: 'right' | 'below'): void {
		if (this.popoverAnchor) {
			this.popoverAnchor = null;
			return;
		}
		this.popoverAnchor = {
			placement,
			rect: {
				top: rect.top,
				left: rect.left,
				bottom: rect.bottom,
				right: rect.right,
				width: rect.width,
				height: rect.height
			}
		};
	}

	closePopover(): void {
		this.popoverAnchor = null;
	}

	/**
	 * Expands the center timeline column cleanly inside the stable 3-column shell
	 * without ever hiding the Left Rail or Right Rail.
	 */
	get isWideDesktopFeed(): boolean {
		return this.fullWidth || this.layout !== 'list';
	}

	get hasNonDefaultSort(): boolean {
		return this.orderBy !== 'created_ts' || this.orderDirection !== 'desc';
	}
}

export const feedDisplayState = new FeedDisplayState();
