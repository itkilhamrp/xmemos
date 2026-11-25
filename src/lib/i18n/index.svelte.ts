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
import { en } from './locales/en.ts';
import { id } from './locales/id.ts';
import { es } from './locales/es.ts';
import {
	SUPPORTED_LOCALES,
	detectBrowserLocale,
	isSupportedLocale,
	type Locale,
	type LocaleMeta,
	type TranslationKey,
	type TranslationParams
} from './types.ts';

const CATALOGS: Record<Locale, Record<TranslationKey, string>> = {
	en,
	id,
	es
};

const STORAGE_KEY = 'xmemos.locale';
const PREF_KEY = 'locale';

export function formatMessage(template: string, params?: TranslationParams): string {
	if (!params) return template;
	return template.replace(/\{(\w+)\}/g, (match, key: string) =>
		Object.prototype.hasOwnProperty.call(params, key) ? String(params[key]) : match
	);
}

class I18nState {
	locale = $state<Locale>('en');
	private initialized = false;

	get meta(): LocaleMeta {
		return SUPPORTED_LOCALES.find((l) => l.code === this.locale) ?? SUPPORTED_LOCALES[0]!;
	}

	get bcp47(): string {
		return this.meta.bcp47;
	}

	init(): void {
		if (typeof window === 'undefined' || this.initialized) return;
		this.initialized = true;

		let initial: Locale = detectBrowserLocale();
		try {
			const stored = localStorage.getItem(STORAGE_KEY);
			if (isSupportedLocale(stored)) {
				initial = stored;
			}
		} catch {
			// Ignore storage access errors
		}
		this.applyLocale(initial);

		const syncLocale = () => {
			void systemApi
				.getLocalPref(PREF_KEY)
				.then((val) => {
					if (isSupportedLocale(val) && val !== this.locale) {
						this.applyLocale(val);
						try {
							localStorage.setItem(STORAGE_KEY, val);
						} catch {
							// Ignore storage access errors
						}
					}
				})
				.catch(() => {});
		};
		if ('requestIdleCallback' in window) {
			window.requestIdleCallback(syncLocale, { timeout: 400 });
		} else {
			setTimeout(syncLocale, 80);
		}
	}

	async setLocale(next: Locale): Promise<void> {
		if (!isSupportedLocale(next)) return;
		this.applyLocale(next);
		try {
			localStorage.setItem(STORAGE_KEY, next);
		} catch {
			// Ignore storage access errors
		}
		await systemApi.setLocalPref(PREF_KEY, next).catch(() => {});
	}

	t(key: TranslationKey, params?: TranslationParams): string {
		const dict = CATALOGS[this.locale] ?? en;
		const template = dict[key] ?? en[key] ?? key;
		return formatMessage(template, params);
	}

	private applyLocale(next: Locale): void {
		this.locale = next;
		if (typeof document !== 'undefined') {
			document.documentElement.lang = next;
		}
	}
}

export const i18n = new I18nState();
export { SUPPORTED_LOCALES, type Locale, type LocaleMeta, type TranslationKey, type TranslationParams };
