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

import type { MemoLocation } from '#lib/types/memo.ts';

export interface ActiveLocationPreview {
	memoUid: string;
	location: MemoLocation;
	displayLabel: string;
	anchorRect: { top: number; left: number; bottom: number; right: number; width: number; height: number };
}

class LocationPreviewState {
	active = $state<ActiveLocationPreview | null>(null);

	toggle(
		memoUid: string,
		location: MemoLocation,
		displayLabel: string,
		rect: DOMRect
	): void {
		if (this.active && this.active.memoUid === memoUid) {
			this.active = null;
			return;
		}
		this.active = {
			memoUid,
			location,
			displayLabel,
			anchorRect: {
				top: rect.top,
				left: rect.left,
				bottom: rect.bottom,
				right: rect.right,
				width: rect.width,
				height: rect.height
			}
		};
	}

	updateLabel(memoUid: string, displayLabel: string): void {
		if (this.active && this.active.memoUid === memoUid) {
			this.active = { ...this.active, displayLabel };
		}
	}

	close(): void {
		this.active = null;
	}
}

export const locationPreviewState = new LocationPreviewState();
