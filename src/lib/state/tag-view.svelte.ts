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

const STORAGE_KEY = 'xmemos.tags_show_as_tree.v1';

class TagViewState {
	showAsTree = $state<boolean>(false);

	constructor() {
		if (typeof window !== 'undefined') {
			try {
				const raw = localStorage.getItem(STORAGE_KEY);
				if (raw === 'true') this.showAsTree = true;
				else if (raw === 'false') this.showAsTree = false;
			} catch {
				// Ignore storage access errors
			}
		}
	}

	toggleTree(): void {
		this.setShowAsTree(!this.showAsTree);
	}

	setShowAsTree(next: boolean): void {
		this.showAsTree = next;
		if (typeof window !== 'undefined') {
			try {
				localStorage.setItem(STORAGE_KEY, String(next));
			} catch {
				// Ignore storage quota errors
			}
		}
	}
}

export const tagViewState = new TagViewState();
