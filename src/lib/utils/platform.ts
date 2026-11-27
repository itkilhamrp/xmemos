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

import { isTauri } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';

declare const __XMEMOS_TARGET__: 'android' | 'desktop' | 'auto' | undefined;

export type XmemosBuildTarget = 'android' | 'desktop' | 'auto';

/**
 * Compile-time build target injected by Vite (`XMEMOS_TARGET_PLATFORM`).
 * Enables dead-code elimination so Android-only UI/logic is stripped from Desktop builds
 * and Desktop-only window chrome/logic is stripped from Android APK builds.
 */
export const BUILD_TARGET_PLATFORM: XmemosBuildTarget =
	typeof __XMEMOS_TARGET__ !== 'undefined' ? __XMEMOS_TARGET__ : 'auto';

const MOBILE_UA_PATTERN = /Android|iPhone|iPad|iPod/i;
const ANDROID_UA_PATTERN = /Android/i;
const INTERACTIVE_DRAG_IGNORE =
	'button, a, input, textarea, select, [role="button"], [contenteditable="true"], [data-no-drag]';

/**
 * Returns true only when running inside a Tauri Desktop WebView on GNU/Linux, macOS, or Windows.
 * Statically resolves to `false` on Android APK builds (`XMEMOS_TARGET_PLATFORM=android`).
 */
export function isDesktopTauri(): boolean {
	if (BUILD_TARGET_PLATFORM === 'android') return false;
	if (typeof window === 'undefined' || !isTauri()) return false;
	if (BUILD_TARGET_PLATFORM === 'desktop') return true;
	const ua = typeof navigator !== 'undefined' ? navigator.userAgent : '';
	return !MOBILE_UA_PATTERN.test(ua);
}

/**
 * Returns true only when running inside the Android APK build.
 * Statically resolves to `false` on Desktop builds (`XMEMOS_TARGET_PLATFORM=desktop`).
 */
export function isAndroidTauri(): boolean {
	if (BUILD_TARGET_PLATFORM === 'desktop') return false;
	if (typeof window === 'undefined' || !isTauri()) return false;
	if (BUILD_TARGET_PLATFORM === 'android') return true;
	const ua = typeof navigator !== 'undefined' ? navigator.userAgent : '';
	return ANDROID_UA_PATTERN.test(ua);
}

/** Android MediaStore `.nomedia` toggle is strictly Android-only. */
export function supportsNomedia(): boolean {
	return isAndroidTauri();
}

/** Android BiometricPrompt hardware unlock is strictly Android-only. */
export function supportsAndroidBiometric(): boolean {
	return isAndroidTauri();
}

/** Native OS File Manager Save/Open dialogs are enabled on Desktop (GNU/Linux, Windows, macOS). */
export function supportsNativeSaveDialog(): boolean {
	return isDesktopTauri();
}

export async function minimizeDesktopWindow(): Promise<void> {
	if (!isDesktopTauri()) return;
	await getCurrentWindow().minimize();
}

export async function toggleMaximizeDesktopWindow(): Promise<void> {
	if (!isDesktopTauri()) return;
	await getCurrentWindow().toggleMaximize();
}

export async function closeDesktopWindow(): Promise<void> {
	if (!isDesktopTauri()) return;
	await getCurrentWindow().close();
}

export async function startDesktopWindowDrag(): Promise<void> {
	if (!isDesktopTauri()) return;
	await getCurrentWindow().startDragging();
}

export async function isDesktopWindowMaximized(): Promise<boolean> {
	if (!isDesktopTauri()) return false;
	try {
		return await getCurrentWindow().isMaximized();
	} catch {
		return false;
	}
}

let lastTitlebarClickTs = 0;
let lastTitlebarClickX = 0;
let lastTitlebarClickY = 0;
let lastMaximizeToggleTs = 0;

/**
 * Cross-OS frameless desktop header drag & double-click maximize/restore handler.
 * Uses a 4px drag-slop threshold before calling `startDragging()` so Windows Win32
 * `WM_NCLBUTTONDOWN` never swallows click #1 or breaks double-click maximize/restore.
 */
export function handleTitlebarMouseDown(event: MouseEvent): void {
	if (!isDesktopTauri()) return;
	if (event.buttons !== 1 || event.button !== 0) return;
	const target = event.target as HTMLElement | null;
	if (target?.closest(INTERACTIVE_DRAG_IGNORE)) return;

	const now = Date.now();
	const distFromPrev = Math.hypot(event.clientX - lastTitlebarClickX, event.clientY - lastTitlebarClickY);
	const isDoubleClick =
		event.detail === 2 || (now - lastTitlebarClickTs <= 360 && distFromPrev <= 6);

	if (isDoubleClick) {
		event.preventDefault();
		lastTitlebarClickTs = 0;
		if (now - lastMaximizeToggleTs > 220) {
			lastMaximizeToggleTs = now;
			void toggleMaximizeDesktopWindow().catch(() => {});
		}
		return;
	}

	lastTitlebarClickTs = now;
	lastTitlebarClickX = event.clientX;
	lastTitlebarClickY = event.clientY;

	const startX = event.clientX;
	const startY = event.clientY;
	let dragStarted = false;

	const cleanup = () => {
		window.removeEventListener('mousemove', onMouseMove);
		window.removeEventListener('mouseup', cleanup);
		window.removeEventListener('blur', cleanup);
	};

	const onMouseMove = (moveEvent: MouseEvent) => {
		if (dragStarted) return;
		if ((moveEvent.buttons & 1) === 0) {
			cleanup();
			return;
		}
		if (Math.hypot(moveEvent.clientX - startX, moveEvent.clientY - startY) >= 4) {
			dragStarted = true;
			lastTitlebarClickTs = 0;
			cleanup();
			void startDesktopWindowDrag().catch(() => {});
		}
	};

	window.addEventListener('mousemove', onMouseMove);
	window.addEventListener('mouseup', cleanup);
	window.addEventListener('blur', cleanup);
}

/**
 * Drag-only mousedown handler for floating Desktop Sticky Note headers.
 * Never triggers maximize on double-click so sticky notes stay compact.
 */
export function handleStickyDragMouseDown(event: MouseEvent): void {
	if (!isDesktopTauri()) return;
	if (event.buttons !== 1 || event.button !== 0) return;
	const target = event.target as HTMLElement | null;
	if (target?.closest(INTERACTIVE_DRAG_IGNORE)) return;
	void startDesktopWindowDrag().catch(() => {});
}

/**
 * Subscribes to desktop window maximize and focus state changes for authentic macOS traffic-light
 * dimming and zoom/restore glyph updates.
 */
export async function subscribeDesktopWindowState(
	onChange: (state: { maximized: boolean; focused: boolean }) => void
): Promise<() => void> {
	if (!isDesktopTauri()) return () => {};

	const appWindow = getCurrentWindow();
	let maximized = await isDesktopWindowMaximized();
	let focused = typeof document !== 'undefined' ? document.hasFocus() : true;
	onChange({ maximized, focused });

	const cleanups: (() => void)[] = [];

	try {
		const unlistenResize = await appWindow.onResized(async () => {
			const nextMax = await isDesktopWindowMaximized();
			if (nextMax !== maximized) {
				maximized = nextMax;
				onChange({ maximized, focused });
			}
		});
		cleanups.push(unlistenResize);
	} catch {
		// Safe fallback if resize listener is unavailable
	}

	try {
		const unlistenFocus = await appWindow.onFocusChanged(({ payload }) => {
			if (payload !== focused) {
				focused = payload;
				onChange({ maximized, focused });
			}
		});
		cleanups.push(unlistenFocus);
	} catch {
		const handleFocus = () => {
			focused = true;
			onChange({ maximized, focused });
		};
		const handleBlur = () => {
			focused = false;
			onChange({ maximized, focused });
		};
		window.addEventListener('focus', handleFocus);
		window.addEventListener('blur', handleBlur);
		cleanups.push(() => {
			window.removeEventListener('focus', handleFocus);
			window.removeEventListener('blur', handleBlur);
		});
	}

	return () => {
		for (const fn of cleanups) {
			try {
				fn();
			} catch {
				// Ignore teardown errors
			}
		}
	};
}
