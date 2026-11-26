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

import { addPluginListener, invoke, isTauri, type PluginListener } from '@tauri-apps/api/core';
import { stripMarkdownToPlainText } from '#lib/utils/markdown.ts';

export type StickyColorId = 'slate' | 'yellow' | 'blue' | 'green' | 'pink' | 'purple';

export interface StickyWindowPrefs {
	x?: number;
	y?: number;
	width?: number;
	height?: number;
	color: StickyColorId;
	alwaysOnTop: boolean;
}

export interface CreateMemoShortcutPayload {
	memoUid: string;
	title: string;
	preview: string;
	pinned: boolean;
	color?: StickyColorId;
	tags?: string[];
	location?: string | null;
	attachmentCount?: number;
	timestamp?: string | null;
}

export interface CreateMemoShortcutResult {
	created: boolean;
	unsupported: boolean;
	path?: string | null;
	message?: string | null;
}

export interface AlwaysOnTopResult {
	active: boolean;
	supported: boolean;
	backend: string;
}

export interface PendingMemoShortcutResult {
	memoUid?: string | null;
}

const VALID_UID_RE = /^[a-zA-Z0-9_-]{1,64}$/;
const STICKY_PREFS_PREFIX = 'xmemos.sticky.prefs.v1.';

const STICKY_NATIVE_BG_LIGHT: Record<StickyColorId, [number, number, number]> = {
	slate: [248, 249, 252],
	yellow: [255, 251, 235],
	blue: [240, 249, 255],
	green: [240, 253, 244],
	pink: [253, 242, 248],
	purple: [245, 243, 255]
};

const STICKY_NATIVE_BG_DARK: Record<StickyColorId, [number, number, number]> = {
	slate: [28, 28, 30],
	yellow: [31, 28, 20],
	blue: [17, 28, 41],
	green: [17, 33, 25],
	pink: [35, 20, 30],
	purple: [25, 22, 40]
};

export function resolveStickyNativeBg(color: StickyColorId, isDark: boolean): [number, number, number] {
	return isDark ? STICKY_NATIVE_BG_DARK[color] : STICKY_NATIVE_BG_LIGHT[color];
}

export function loadStickyPrefs(memoUid: string): StickyWindowPrefs {
	const defaults: StickyWindowPrefs = { color: 'yellow', alwaysOnTop: false };
	if (typeof window === 'undefined' || !VALID_UID_RE.test(memoUid)) return defaults;
	try {
		const raw = localStorage.getItem(`${STICKY_PREFS_PREFIX}${memoUid}`);
		if (!raw) return defaults;
		const parsed = JSON.parse(raw) as Partial<StickyWindowPrefs>;
		const validColor: StickyColorId =
			parsed.color === 'slate' ||
			parsed.color === 'yellow' ||
			parsed.color === 'blue' ||
			parsed.color === 'green' ||
			parsed.color === 'pink' ||
			parsed.color === 'purple'
				? parsed.color
				: 'slate';
		return {
			x: typeof parsed.x === 'number' && Number.isFinite(parsed.x) ? parsed.x : undefined,
			y: typeof parsed.y === 'number' && Number.isFinite(parsed.y) ? parsed.y : undefined,
			width:
				typeof parsed.width === 'number' && Number.isFinite(parsed.width)
					? parsed.width
					: undefined,
			height:
				typeof parsed.height === 'number' && Number.isFinite(parsed.height)
					? parsed.height
					: undefined,
			color: validColor,
			alwaysOnTop: Boolean(parsed.alwaysOnTop)
		};
	} catch {
		return defaults;
	}
}

export function saveStickyPrefs(memoUid: string, prefs: StickyWindowPrefs): void {
	if (typeof window === 'undefined' || !VALID_UID_RE.test(memoUid)) return;
	try {
		localStorage.setItem(`${STICKY_PREFS_PREFIX}${memoUid}`, JSON.stringify(prefs));
	} catch {
		// Ignore storage quota errors
	}
}

/**
 * Extracts a clean, concise title and preview snippet from Markdown memo content
 * for Android Home Screen Sticky Widgets/Shortcuts and Desktop OS launchers.
 */
export function buildMemoShortcutMeta(content: string): { title: string; preview: string } {
	const normalized = content
		.replace(/\r\n/g, '\n')
		.replace(/^\s*[-*+]\s+\[[xX]\]\s+/gm, '☑ ')
		.replace(/^\s*[-*+]\s+\[\s?\]\s+/gm, '☐ ')
		.replace(/^\s*[-*+]\s+/gm, '• ');
	const plain = stripMarkdownToPlainText(normalized);
	const lines = plain
		.split('\n')
		.map((l) => l.trim())
		.filter((l) => l.length > 0);

	const firstLine = lines[0] ?? 'Xmemos Memo';
	const title = firstLine.length > 48 ? `${firstLine.slice(0, 45).trimEnd()}...` : firstLine;
	const joined = lines.join('\n');
	const preview = joined.length > 360 ? `${joined.slice(0, 357).trimEnd()}...` : joined || title;
	return { title, preview };
}

export const shortcutApi = {
	async createMemoShortcut(payload: CreateMemoShortcutPayload): Promise<CreateMemoShortcutResult> {
		if (!isTauri()) {
			return {
				created: false,
				unsupported: true,
				message: 'Shortcuts require the native Xmemos Android or Desktop app'
			};
		}
		return invoke<CreateMemoShortcutResult>('plugin:xmemos-storage|create_memo_shortcut', {
			payload
		});
	},

	async openStickyMemoWindow(memoUid: string, title?: string): Promise<void> {
		if (!isTauri() || !VALID_UID_RE.test(memoUid)) return;
		const saved = loadStickyPrefs(memoUid);
		const isDark =
			typeof document !== 'undefined' && document.documentElement.classList.contains('dark');
		const bgRgb = resolveStickyNativeBg(saved.color, isDark);
		await invoke('system_open_sticky_memo_window', {
			payload: {
				memoUid,
				title: title ?? 'Xmemos Sticky Note',
				x: saved.x ?? null,
				y: saved.y ?? null,
				width: saved.width ?? 332,
				height: saved.height ?? 356,
				alwaysOnTop: saved.alwaysOnTop,
				bgRgb
			}
		});
	},

	async focusMainMemo(memoUid: string): Promise<void> {
		if (!isTauri() || !VALID_UID_RE.test(memoUid)) return;
		await invoke('system_focus_main_memo', { memoUid });
	},

	async focusMainRoute(route: string): Promise<void> {
		if (!isTauri() || !route.startsWith('/') || route.startsWith('//')) return;
		await invoke('system_focus_main_route', { route });
	},

	async setStickyAlwaysOnTop(label: string, alwaysOnTop: boolean): Promise<AlwaysOnTopResult> {
		if (!isTauri()) return { active: alwaysOnTop, supported: false, backend: 'browser' };
		return invoke<AlwaysOnTopResult>('system_set_sticky_always_on_top', { label, alwaysOnTop });
	},

	async consumePendingMemoShortcut(): Promise<string | null> {
		if (!isTauri()) return null;
		try {
			const res = await invoke<PendingMemoShortcutResult>(
				'plugin:xmemos-storage|consume_pending_memo_shortcut'
			);
			const uid = res?.memoUid?.trim();
			return uid && VALID_UID_RE.test(uid) ? uid : null;
		} catch {
			return null;
		}
	},

	async onMemoShortcutOpened(handler: (memoUid: string) => void): Promise<(() => void) | null> {
		if (!isTauri()) return null;
		try {
			const listener: PluginListener = await addPluginListener<{ memoUid?: string }>(
				'xmemos-storage',
				'memoShortcutOpened',
				(event) => {
					const uid = event?.memoUid?.trim();
					if (uid && VALID_UID_RE.test(uid)) {
						handler(uid);
					}
				}
			);
			return () => {
				void listener.unregister();
			};
		} catch {
			return null;
		}
	}
};
