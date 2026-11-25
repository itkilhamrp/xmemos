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

export type ThemeMode = 'system' | 'light' | 'dark';
export type GlassIntensity = 'liquid' | 'subtle' | 'solid';
export type DensityMode = 'comfortable' | 'compact';

class ThemeState {
	mode = $state<ThemeMode>('system');
	glass = $state<GlassIntensity>('liquid');
	density = $state<DensityMode>('comfortable');
	systemPrefersDark = $state(false);
	private initialized = false;
	private mediaQuery: MediaQueryList | null = null;
	private mediaListener: ((e: MediaQueryListEvent) => void) | null = null;

	isDark = $derived(
		this.mode === 'dark' || (this.mode === 'system' && this.systemPrefersDark)
	);

	init(): void {
		if (typeof window === 'undefined' || this.initialized) return;
		this.initialized = true;

		const mq = window.matchMedia('(prefers-color-scheme: dark)');
		this.mediaQuery = mq;
		this.systemPrefersDark = mq.matches;
		this.mediaListener = (e: MediaQueryListEvent) => {
			this.systemPrefersDark = e.matches;
			this.applyToDom();
		};
		mq.addEventListener('change', this.mediaListener);

		try {
			const storedMode = localStorage.getItem('xmemos.theme');
			const storedGlass = localStorage.getItem('xmemos.glass');
			const storedDensity = localStorage.getItem('xmemos.density');
			if (storedMode === 'system' || storedMode === 'light' || storedMode === 'dark') {
				this.mode = storedMode;
			}
			if (storedGlass === 'liquid' || storedGlass === 'subtle' || storedGlass === 'solid') {
				this.glass = storedGlass;
			}
			if (storedDensity === 'comfortable' || storedDensity === 'compact') {
				this.density = storedDensity;
			}
		} catch {
			// Storage unavailable
		}
		this.applyToDom();

		// Defer xmemos_local.db preference sync until idle so critical auth/timeline IPC never contends
		const syncPrefs = () => {
			void Promise.all([
				systemApi.getLocalPref('theme_mode').catch(() => null),
				systemApi.getLocalPref('glass_intensity').catch(() => null),
				systemApi.getLocalPref('density_mode').catch(() => null)
			]).then(([m, g, d]) => {
				if (m === 'system' || m === 'light' || m === 'dark') this.mode = m;
				if (g === 'liquid' || g === 'subtle' || g === 'solid') this.glass = g;
				if (d === 'comfortable' || d === 'compact') this.density = d;
				this.applyToDom();
			});
		};
		if ('requestIdleCallback' in window) {
			window.requestIdleCallback(syncPrefs, { timeout: 400 });
		} else {
			setTimeout(syncPrefs, 80);
		}
	}

	async setMode(next: ThemeMode): Promise<void> {
		this.mode = next;
		try {
			localStorage.setItem('xmemos.theme', next);
		} catch {
			// Ignore storage error
		}
		this.applyToDom();
		await systemApi.setLocalPref('theme_mode', next).catch(() => {});
	}

	async setGlass(next: GlassIntensity): Promise<void> {
		this.glass = next;
		try {
			localStorage.setItem('xmemos.glass', next);
		} catch {
			// Ignore storage error
		}
		this.applyToDom();
		await systemApi.setLocalPref('glass_intensity', next).catch(() => {});
	}

	async setDensity(next: DensityMode): Promise<void> {
		this.density = next;
		try {
			localStorage.setItem('xmemos.density', next);
		} catch {
			// Ignore storage error
		}
		await systemApi.setLocalPref('density_mode', next).catch(() => {});
	}

	applyToDom(): void {
		if (typeof document === 'undefined') return;
		const root = document.documentElement;
		const dark = this.isDark;
		root.classList.toggle('dark', dark);
		root.dataset.glass = this.glass;
		root.style.colorScheme = dark ? 'dark' : 'light';
		void systemApi.setWindowThemeBg(dark).catch(() => {});
	}
}

export const themeState = new ThemeState();
