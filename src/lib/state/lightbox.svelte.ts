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

export interface LightboxMedia {
	uid?: string;
	src: string;
	filename: string;
	size?: number;
	mimeType?: string;
}

class LightboxState {
	items = $state<LightboxMedia[]>([]);
	index = $state<number>(0);

	get current(): LightboxMedia | null {
		if (this.items.length === 0) return null;
		return this.items[this.index] ?? this.items[0] ?? null;
	}

	get hasMultiple(): boolean {
		return this.items.length > 1;
	}

	open(media: LightboxMedia, gallery?: LightboxMedia[], startIndex = 0): void {
		if (gallery && gallery.length > 0) {
			this.items = gallery;
			this.index = Math.max(0, Math.min(startIndex, gallery.length - 1));
		} else {
			this.items = [media];
			this.index = 0;
		}
	}

	prev(): void {
		if (this.items.length <= 1) return;
		this.index = (this.index - 1 + this.items.length) % this.items.length;
	}

	next(): void {
		if (this.items.length <= 1) return;
		this.index = (this.index + 1) % this.items.length;
	}

	close(): void {
		this.items = [];
		this.index = 0;
	}
}

export const lightboxState = new LightboxState();
