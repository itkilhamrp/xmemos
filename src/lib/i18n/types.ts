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

import type { en } from './locales/en.ts';

export type Locale = 'en' | 'id' | 'es';

export type TranslationKey = keyof typeof en;

export type TranslationParams = Record<string, string | number>;

export interface LocaleMeta {
	code: Locale;
	nativeName: string;
	englishName: string;
	bcp47: string;
}

export const SUPPORTED_LOCALES: readonly LocaleMeta[] = [
	{ code: 'en', nativeName: 'English', englishName: 'English', bcp47: 'en-US' },
	{ code: 'id', nativeName: 'Indonesia', englishName: 'Indonesian', bcp47: 'id-ID' },
	{ code: 'es', nativeName: 'Español', englishName: 'Spanish', bcp47: 'es-ES' }
] as const;

export function isSupportedLocale(value: string | null | undefined): value is Locale {
	return value === 'en' || value === 'id' || value === 'es';
}

export function detectBrowserLocale(): Locale {
	if (typeof navigator === 'undefined') return 'en';
	const langs = navigator.languages?.length ? navigator.languages : [navigator.language];
	for (const raw of langs) {
		const prefix = raw?.toLowerCase().split('-')[0];
		if (prefix === 'id' || prefix === 'in') return 'id';
		if (prefix === 'es') return 'es';
		if (prefix === 'en') return 'en';
	}
	return 'en';
}
