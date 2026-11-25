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

import { i18n } from '#lib/i18n/index.svelte.ts';

function pad2(n: number): string {
	return String(n).padStart(2, '0');
}

/**
 * Formats a Date into local YYYY-MM-DD calendar date matching the device's timezone.
 */
export function toLocalDateKey(date: Date): string {
	return `${date.getFullYear()}-${pad2(date.getMonth() + 1)}-${pad2(date.getDate())}`;
}

/**
 * Formats a Unix timestamp (seconds) into local YYYY-MM-DDTHH:mm for <input type="datetime-local">.
 */
export function formatLocalDateTimeInput(unixSeconds: number): string {
	const d = new Date(unixSeconds * 1000);
	if (Number.isNaN(d.getTime())) return '';
	return `${d.getFullYear()}-${pad2(d.getMonth() + 1)}-${pad2(d.getDate())}T${pad2(d.getHours())}:${pad2(d.getMinutes())}`;
}

/**
 * Formats a local Date (default: current local time) into YYYY-MM-DDTHH:mm for <input type="datetime-local">.
 */
export function formatCurrentLocalDateTimeInput(now = new Date()): string {
	return `${now.getFullYear()}-${pad2(now.getMonth() + 1)}-${pad2(now.getDate())}T${pad2(now.getHours())}:${pad2(now.getMinutes())}`;
}

/**
 * Combines a YYYY-MM-DD calendar date key with the current local HH:mm time
 * (Official Memos "New memo on this day" behavior).
 */
export function formatDateKeyWithCurrentTime(dateKey: string, now = new Date()): string {
	const m = /^(\d{4})-(\d{2})-(\d{2})$/.exec(dateKey.trim());
	if (!m) return formatCurrentLocalDateTimeInput(now);
	return `${m[1]}-${m[2]}-${m[3]}T${pad2(now.getHours())}:${pad2(now.getMinutes())}`;
}

/**
 * Parses a local YYYY-MM-DDTHH:mm string explicitly in the device's local timezone into Unix seconds.
 */
export function parseLocalDateTimeInput(value: string): number | undefined {
	const m = /^(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2})(?::(\d{2}))?$/.exec(value.trim());
	if (!m) return undefined;
	const year = Number(m[1]);
	const month = Number(m[2]);
	const day = Number(m[3]);
	const hour = Number(m[4]);
	const minute = Number(m[5]);
	const second = m[6] ? Number(m[6]) : 0;
	const d = new Date(year, month - 1, day, hour, minute, second, 0);
	const ts = Math.floor(d.getTime() / 1000);
	return Number.isNaN(ts) ? undefined : ts;
}

/**
 * Returns local-timezone Unix start/end second bounds for a YYYY-MM-DD date key.
 */
export function getLocalDayBounds(dateKey: string): { startTs: number | null; endTs: number | null } {
	const m = /^(\d{4})-(\d{2})-(\d{2})$/.exec(dateKey.trim());
	if (!m) return { startTs: null, endTs: null };
	const year = Number(m[1]);
	const month = Number(m[2]);
	const day = Number(m[3]);
	const start = new Date(year, month - 1, day, 0, 0, 0, 0);
	const end = new Date(year, month - 1, day, 23, 59, 59, 999);
	const startTs = Math.floor(start.getTime() / 1000);
	const endTs = Math.floor(end.getTime() / 1000);
	if (Number.isNaN(startTs) || Number.isNaN(endTs)) return { startTs: null, endTs: null };
	return { startTs, endTs };
}

export function formatRelativeTime(unixSeconds: number, nowMs = Date.now()): string {
	const nowSec = Math.floor(nowMs / 1000);
	const diff = nowSec - unixSeconds;
	if (diff < -60) return formatFullTimestamp(unixSeconds);
	if (diff < 60) return i18n.t('time.justNow');
	if (diff < 3600) return i18n.t('time.minutesAgo', { n: Math.floor(diff / 60) });
	if (diff < 86400) return i18n.t('time.hoursAgo', { n: Math.floor(diff / 3600) });
	if (diff < 604800) return i18n.t('time.daysAgo', { n: Math.floor(diff / 86400) });

	return new Date(unixSeconds * 1000).toLocaleDateString(undefined, {
		year: 'numeric',
		month: 'short',
		day: 'numeric'
	});
}

export function formatFullTimestamp(unixSeconds: number): string {
	return new Date(unixSeconds * 1000).toLocaleString(i18n.bcp47, {
		year: 'numeric',
		month: 'short',
		day: 'numeric',
		hour: '2-digit',
		minute: '2-digit'
	});
}

export function formatBytes(bytes: number): string {
	if (bytes <= 0) return '0 B';
	const units = ['B', 'KB', 'MB', 'GB'];
	const idx = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
	const value = bytes / Math.pow(1024, idx);
	return `${value.toFixed(idx === 0 ? 0 : 1)} ${units[idx]}`;
}
