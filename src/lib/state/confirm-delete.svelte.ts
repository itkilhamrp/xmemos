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

import { systemApi } from '#lib/api/system.ts';

export interface DeleteConfirmDetail {
	label: string;
	value: string;
}

export interface DeleteConfirmRequest {
	title: string;
	message: string;
	confirmLabel?: string;
	forceConfirm?: boolean;
	details?: DeleteConfirmDetail[];
	samples?: string[];
	onConfirm: () => void | Promise<void>;
}

const STORAGE_KEY = 'xmemos.confirm_before_delete';
const PREF_KEY = 'confirm_before_delete';

class ConfirmDeleteState {
	enabled = $state(true);
	pending = $state<DeleteConfirmRequest | null>(null);
	busy = $state(false);
	private initialized = false;

	init(): void {
		if (typeof window === 'undefined' || this.initialized) return;
		this.initialized = true;

		try {
			const stored = localStorage.getItem(STORAGE_KEY);
			if (stored === 'true' || stored === 'false') {
				this.enabled = stored === 'true';
			}
		} catch {
			// Ignore storage errors
		}

		const syncConfirmPref = () => {
			void systemApi
				.getLocalPref(PREF_KEY)
				.then((val) => {
					if (val === 'true' || val === 'false') {
						this.enabled = val === 'true';
						try {
							localStorage.setItem(STORAGE_KEY, val);
						} catch {
							// Ignore storage errors
						}
					}
				})
				.catch(() => {});
		};
		if ('requestIdleCallback' in window) {
			window.requestIdleCallback(syncConfirmPref, { timeout: 400 });
		} else {
			setTimeout(syncConfirmPref, 80);
		}
	}

	async setEnabled(next: boolean): Promise<void> {
		this.enabled = next;
		const serialized = next ? 'true' : 'false';
		try {
			localStorage.setItem(STORAGE_KEY, serialized);
		} catch {
			// Ignore storage errors
		}
		await systemApi.setLocalPref(PREF_KEY, serialized).catch(() => {});
	}

	async run(req: DeleteConfirmRequest): Promise<void> {
		if (!this.enabled && !req.forceConfirm) {
			await req.onConfirm();
			return;
		}
		this.pending = req;
	}

	cancel(): void {
		if (this.busy) return;
		this.pending = null;
	}

	async confirm(): Promise<void> {
		const req = this.pending;
		if (!req || this.busy) return;
		this.busy = true;
		try {
			await req.onConfirm();
		} finally {
			this.busy = false;
			this.pending = null;
		}
	}
}

export const confirmDeleteState = new ConfirmDeleteState();
