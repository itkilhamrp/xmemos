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

export interface ShortcutHandlers {
	onCompose?: () => void;
	onFocusSearch?: () => void;
}

class ShortcutsState {
	cheatsheetOpened = $state(false);
	private composeHandler: (() => void) | null = null;
	private focusSearchHandler: (() => void) | null = null;

	get isMac(): boolean {
		if (typeof navigator === 'undefined') return false;
		return /Mac|iPhone|iPad|iPod/i.test(navigator.platform || navigator.userAgent);
	}

	get modKey(): string {
		return this.isMac ? '⌘' : 'Ctrl';
	}

	openCheatsheet(): void {
		this.cheatsheetOpened = true;
	}

	closeCheatsheet(): void {
		this.cheatsheetOpened = false;
	}

	toggleCheatsheet(): void {
		this.cheatsheetOpened = !this.cheatsheetOpened;
	}

	registerCompose(fn: () => void): () => void {
		this.composeHandler = fn;
		return () => {
			if (this.composeHandler === fn) {
				this.composeHandler = null;
			}
		};
	}

	registerFocusSearch(fn: () => void): () => void {
		this.focusSearchHandler = fn;
		return () => {
			if (this.focusSearchHandler === fn) {
				this.focusSearchHandler = null;
			}
		};
	}

	triggerCompose(): boolean {
		if (!this.composeHandler) return false;
		this.composeHandler();
		return true;
	}

	triggerFocusSearch(): boolean {
		if (!this.focusSearchHandler) return false;
		this.focusSearchHandler();
		return true;
	}
}

export const shortcutsState = new ShortcutsState();
