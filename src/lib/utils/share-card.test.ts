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
import { parseInlineRuns, parseShareCardBlocks } from './share-card-markdown';

describe('share-card Markdown parser', () => {
	it('parses headings, tasks, lists, quotes, code blocks, and horizontal rules', () => {
		const md = [
			'# Weekly Review',
			'- [x] Ship Liquid Glass icon',
			'- [ ] Verify export card #design/ui',
			'> Keep it clean and minimal.',
			'```ts',
			'const ok = true;',
			'```',
			'---',
			'1. First item'
		].join('\n');

		const blocks = parseShareCardBlocks(md);
		expect(blocks.map((b) => b.type)).toEqual([
			'heading',
			'task',
			'task',
			'blockquote',
			'code',
			'hr',
			'list'
		]);
		expect(blocks[0]).toMatchObject({ type: 'heading', level: 1 });
		expect(blocks[1]).toMatchObject({ type: 'task', checked: true });
		expect(blocks[2]).toMatchObject({ type: 'task', checked: false });
		expect(blocks[4]).toMatchObject({ type: 'code', lang: 'ts', lines: ['const ok = true;'] });
		expect(blocks[6]).toMatchObject({ type: 'list', marker: '1.' });
	});

	it('parses inline bold, italic, strike, code, links, and nested #tags', () => {
		const runs = parseInlineRuns(
			'Check **bold**, *italic*, ~~old~~, `pnpm test`, [Memos](https://usememos.com), and #work/notes'
		);
		expect(runs.some((r) => r.kind === 'bold' && r.text === 'bold')).toBe(true);
		expect(runs.some((r) => r.kind === 'italic' && r.text === 'italic')).toBe(true);
		expect(runs.some((r) => r.kind === 'strike' && r.text === 'old')).toBe(true);
		expect(runs.some((r) => r.kind === 'code' && r.text === 'pnpm test')).toBe(true);
		expect(runs.some((r) => r.kind === 'link' && r.text === 'Memos')).toBe(true);
		expect(runs.some((r) => r.kind === 'tag' && r.text === '#work/notes')).toBe(true);
	});
});
