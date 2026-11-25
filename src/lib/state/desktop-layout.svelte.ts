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

const STORAGE_KEY = 'xmemos.desktop.layout.v1';

interface StoredDesktopLayout {
	sidebarOpen?: boolean;
	inspectorOpen?: boolean;
}

class DesktopLayoutState {
	sidebarOpen = $state<boolean>(true);
	inspectorOpen = $state<boolean>(true);
	private initialized = false;

	init(): void {
		if (this.initialized || typeof window === 'undefined') return;
		this.initialized = true;
		try {
			const raw = window.localStorage.getItem(STORAGE_KEY);
			if (!raw) return;
			const parsed = JSON.parse(raw) as StoredDesktopLayout;
			if (typeof parsed.sidebarOpen === 'boolean') {
				this.sidebarOpen = parsed.sidebarOpen;
			}
			if (typeof parsed.inspectorOpen === 'boolean') {
				this.inspectorOpen = parsed.inspectorOpen;
			}
		} catch {
			// Ignore malformed storage values and keep defaults
		}
	}

	private persist(): void {
		if (typeof window === 'undefined') return;
		try {
			window.localStorage.setItem(
				STORAGE_KEY,
				JSON.stringify({
					sidebarOpen: this.sidebarOpen,
					inspectorOpen: this.inspectorOpen
				})
			);
		} catch {
			// Storage quota or restricted mode
		}
	}

	toggleSidebar(): void {
		this.sidebarOpen = !this.sidebarOpen;
		this.persist();
	}

	toggleInspector(): void {
		this.inspectorOpen = !this.inspectorOpen;
		this.persist();
	}

	setSidebarOpen(open: boolean): void {
		if (this.sidebarOpen === open) return;
		this.sidebarOpen = open;
		this.persist();
	}

	setInspectorOpen(open: boolean): void {
		if (this.inspectorOpen === open) return;
		this.inspectorOpen = open;
		this.persist();
	}
}

export const desktopLayoutState = new DesktopLayoutState();
