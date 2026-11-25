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

/**
 * Tracks mobile soft-keyboard geometry via the standard Visual Viewport API.
 * Syncs `--xm-vvh`, `--xm-vvt`, and `--xm-kb-inset` CSS custom properties so
 * sheets, dialogs, and input modals stay stably positioned above the Android
 * / iOS on-screen keyboard without jumping or clipping.
 */
class KeyboardViewportState {
	open = $state(false);
	inset = $state(0);
	viewportHeight = $state(0);
}

export const keyboardState = new KeyboardViewportState();

const KEYBOARD_THRESHOLD_PX = 100;

export function initKeyboardViewportSync(): () => void {
	if (typeof window === 'undefined' || typeof document === 'undefined') {
		return () => {};
	}

	const root = document.documentElement;
	let rafId = 0;

	const sync = (): void => {
		rafId = 0;
		const vv = window.visualViewport;
		const vh = vv ? Math.round(vv.height) : window.innerHeight;
		const vt = vv ? Math.max(0, Math.round(vv.offsetTop)) : 0;
		const rawInset = vv ? Math.max(0, Math.round(window.innerHeight - vv.height - vv.offsetTop)) : 0;
		const isKeyboardOpen = rawInset >= KEYBOARD_THRESHOLD_PX;

		keyboardState.viewportHeight = vh;
		keyboardState.inset = isKeyboardOpen ? rawInset : 0;
		keyboardState.open = isKeyboardOpen;

		root.style.setProperty('--xm-vvh', `${vh}px`);
		root.style.setProperty('--xm-vvt', `${vt}px`);
		root.style.setProperty('--xm-kb-inset', `${isKeyboardOpen ? rawInset : 0}px`);
		root.classList.toggle('xm-kb-open', isKeyboardOpen);
	};

	const scheduleSync = (): void => {
		if (rafId !== 0) return;
		rafId = window.requestAnimationFrame(sync);
	};

	sync();

	const vv = window.visualViewport;
	vv?.addEventListener('resize', scheduleSync, { passive: true });
	vv?.addEventListener('scroll', scheduleSync, { passive: true });
	window.addEventListener('resize', scheduleSync, { passive: true });
	window.addEventListener('orientationchange', scheduleSync, { passive: true });

	return () => {
		if (rafId !== 0) {
			window.cancelAnimationFrame(rafId);
		}
		vv?.removeEventListener('resize', scheduleSync);
		vv?.removeEventListener('scroll', scheduleSync);
		window.removeEventListener('resize', scheduleSync);
		window.removeEventListener('orientationchange', scheduleSync);
	};
}

export function scrollFieldIntoSafeView(el: HTMLElement | null): void {
	if (!el || typeof window === 'undefined') return;
	window.setTimeout(() => {
		try {
			el.scrollIntoView({ block: 'center', inline: 'nearest', behavior: 'smooth' });
		} catch {
			// Ignore scrollIntoView errors on detached nodes
		}
	}, 120);
}
