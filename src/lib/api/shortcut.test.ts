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
import { buildMemoShortcutMeta } from './shortcut';

describe('buildMemoShortcutMeta', () => {
	it('extracts a clean title and preview from Markdown memos', () => {
		const meta = buildMemoShortcutMeta(
			'# Project Roadmap\n- [x] Ship Liquid Glass icons\n- [ ] Test desktop shortcuts'
		);
		expect(meta.title).toBe('Project Roadmap');
		expect(meta.preview).toContain('Project Roadmap');
		expect(meta.preview.length).toBeLessThanOrEqual(120);
	});

	it('falls back cleanly on empty content and truncates long titles', () => {
		expect(buildMemoShortcutMeta('   \n\t  ')).toEqual({
			title: 'Xmemos Memo',
			preview: 'Xmemos Memo'
		});
		const longMeta = buildMemoShortcutMeta('A'.repeat(80));
		expect(longMeta.title.length).toBeLessThanOrEqual(48);
		expect(longMeta.title.endsWith('...')).toBe(true);
	});
});
