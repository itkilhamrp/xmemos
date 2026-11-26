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

export const PRIMARY_TAB_PATHS = ['/', '/explore', '/review', '/resources', '/settings'] as const;

export type NavSlideDirection = 'forward' | 'backward' | 'none';

export function getPrimaryTabIndex(pathname: string): number {
	for (let i = 0; i < PRIMARY_TAB_PATHS.length; i += 1) {
		const base = PRIMARY_TAB_PATHS[i];
		if (base === '/') {
			if (pathname === '/') return 0;
		} else if (pathname === base || pathname.startsWith(`${base}/`)) {
			return i;
		}
	}
	return -1;
}

function pathDepth(pathname: string): number {
	return pathname.split('/').filter(Boolean).length;
}

/**
 * Resolves a deterministic horizontal slide direction for PowerPoint-like GPU page transitions:
 * - Between primary bottom-tabbar routes: left-to-right index order (`forward` / `backward`).
 * - Within the same section (e.g. `/settings` -> `/settings/system`): depth-based drill-in/back.
 * - Skips transitions on `/auth`, `/sticky/*`, or identical pathnames.
 */
export function resolveNavigationDirection(fromPath: string, toPath: string): NavSlideDirection {
	if (!fromPath || !toPath || fromPath === toPath) return 'none';
	if (
		fromPath === '/auth' ||
		toPath === '/auth' ||
		fromPath.startsWith('/sticky/') ||
		toPath.startsWith('/sticky/')
	) {
		return 'none';
	}

	const fromTab = getPrimaryTabIndex(fromPath);
	const toTab = getPrimaryTabIndex(toPath);

	if (fromTab !== -1 && toTab !== -1 && fromTab !== toTab) {
		return toTab > fromTab ? 'forward' : 'backward';
	}

	const fromDepth = pathDepth(fromPath);
	const toDepth = pathDepth(toPath);
	if (toDepth !== fromDepth) {
		return toDepth > fromDepth ? 'forward' : 'backward';
	}

	return 'forward';
}
