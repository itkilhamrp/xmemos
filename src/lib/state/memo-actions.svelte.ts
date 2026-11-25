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

export interface MemoActionsAnchorRect {
	top: number;
	bottom: number;
	left: number;
	right: number;
	width: number;
	height: number;
}

export interface MemoActionsRequest {
	view: MemoView;
	canManage: boolean;
	anchorRect?: MemoActionsAnchorRect;
	onEdit?: (view: MemoView) => void;
	onPin?: (view: MemoView) => void;
	onArchive?: (view: MemoView) => void;
	onRestore?: (view: MemoView) => void;
	onDelete?: (view: MemoView) => void;
}

const DESKTOP_MIN_WIDTH = 1024;

class MemoActionsState {
	active = $state<MemoActionsRequest | null>(null);
	opened = $state(false);
	mode = $state<'desktop' | 'mobile'>('mobile');
	private closeTimer: ReturnType<typeof setTimeout> | null = null;

	open(req: MemoActionsRequest): void {
		const isDesktop =
			typeof window !== 'undefined' &&
			window.innerWidth >= DESKTOP_MIN_WIDTH &&
			Boolean(req.anchorRect);

		// Native macOS pull-down toggle: clicking the same trigger again closes the open menu
		if (isDesktop && this.opened && this.mode === 'desktop' && this.active?.view.memo.id === req.view.memo.id) {
			this.close();
			return;
		}

		if (this.closeTimer) {
			clearTimeout(this.closeTimer);
			this.closeTimer = null;
		}

		this.mode = isDesktop ? 'desktop' : 'mobile';
		this.active = req;

		if (isDesktop) {
			this.opened = true;
		} else if (typeof requestAnimationFrame !== 'undefined') {
			requestAnimationFrame(() => {
				this.opened = true;
			});
		} else {
			this.opened = true;
		}
	}

	close(): void {
		this.opened = false;
		if (this.closeTimer) clearTimeout(this.closeTimer);
		const delay = this.mode === 'desktop' ? 140 : 280;
		this.closeTimer = setTimeout(() => {
			if (!this.opened) {
				this.active = null;
			}
			this.closeTimer = null;
		}, delay);
	}
}

export const memoActionsState = new MemoActionsState();
