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

import { describe, expect, it } from 'vitest';
import { en } from './locales/en.ts';
import { id } from './locales/id.ts';
import { es } from './locales/es.ts';
import { formatMessage } from './index.svelte.ts';

function extractPlaceholders(value: string): string[] {
	const matches = value.match(/\{(\w+)\}/g) ?? [];
	return [...new Set(matches)].sort();
}

describe('i18n catalog integrity (en, id, es)', () => {
	const enKeys = Object.keys(en).sort();

	it('maintains 100% key parity across en, id, and es with zero empty strings', () => {
		expect(Object.keys(id).sort()).toEqual(enKeys);
		expect(Object.keys(es).sort()).toEqual(enKeys);

		for (const key of enKeys) {
			const k = key as keyof typeof en;
			expect(en[k].trim().length, `en.${key} is empty`).toBeGreaterThan(0);
			expect(id[k].trim().length, `id.${key} is empty`).toBeGreaterThan(0);
			expect(es[k].trim().length, `es.${key} is empty`).toBeGreaterThan(0);
		}
	});

	it('preserves identical interpolation placeholders across all locales', () => {
		for (const key of enKeys) {
			const k = key as keyof typeof en;
			const expected = extractPlaceholders(en[k]);
			expect(extractPlaceholders(id[k]), `placeholder mismatch in id.${key}`).toEqual(expected);
			expect(extractPlaceholders(es[k]), `placeholder mismatch in es.${key}`).toEqual(expected);
		}
	});

	it('interpolates named parameters accurately', () => {
		expect(formatMessage(en['resources.cleanedToast'], { count: 2, label: 'items', size: '1.2 MB' })).toBe(
			'Cleaned 2 orphan items (1.2 MB freed)'
		);
		expect(formatMessage(id['resources.cleanedToast'], { count: 2, label: 'item', size: '1.2 MB' })).toBe(
			'Membersihkan 2 item orphan (1.2 MB dibebaskan)'
		);
		expect(formatMessage(es['resources.cleanedToast'], { count: 2, label: 'elementos', size: '1.2 MB' })).toBe(
			'Se limpiaron 2 elementos huérfanos (1.2 MB liberados)'
		);
	});
});
