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
import { formatCoordinates, formatNominatimPlace, formatPhotonPlace, isCoordinateOnlyLabel } from './geocode.ts';
import { detectCodeLanguage } from './code-highlight.ts';
import { buildTagTree } from './tag';
import {
	formatBytes,
	formatDateKeyWithCurrentTime,
	formatLocalDateTimeInput,
	getLocalDayBounds,
	parseLocalDateTimeInput,
	toLocalDateKey
} from './date';
import { highlightHtmlMatches, renderMemoMarkdown, stripMarkdownToPlainText } from './markdown.ts';

describe('Xmemos frontend utilities', () => {
	it('builds hierarchical nested tag trees accurately', () => {
		const tree = buildTagTree([
			{ name: 'work/backend', count: 3 },
			{ name: 'work/frontend', count: 2 },
			{ name: 'personal', count: 1 }
		]);
		expect(tree).toHaveLength(2);
		expect(tree[0]?.segment).toBe('work');
		expect(tree[0]?.count).toBe(5);
		expect(tree[0]?.children).toHaveLength(2);
	});

	it('formats byte sizes cleanly', () => {
		expect(formatBytes(0)).toBe('0 B');
		expect(formatBytes(1024)).toBe('1.0 KB');
		expect(formatBytes(1024 * 1024 * 2.5)).toBe('2.5 MB');
	});

	it('strips markdown formatting cleanly to plain text', () => {
		expect(stripMarkdownToPlainText('# Title\n**Bold** [link](https://usememos.com)')).toBe(
			'Title\nBold link'
		);
	});

	it('preserves local device timezone in datetime-local and date bounds helpers', () => {
		const sample = new Date(2026, 10, 25, 5, 35, 0, 0);
		const unixSec = Math.floor(sample.getTime() / 1000);
		expect(toLocalDateKey(sample)).toBe('2026-11-25');
		const formatted = formatLocalDateTimeInput(unixSec);
		expect(formatted).toBe('2026-11-25T05:35');
		expect(parseLocalDateTimeInput(formatted)).toBe(unixSec);

		const bounds = getLocalDayBounds('2026-11-25');
		expect(bounds.startTs).not.toBeNull();
		expect(bounds.endTs).not.toBeNull();
		expect((bounds.endTs ?? 0) - (bounds.startTs ?? 0)).toBe(86_399);
	});

	it('renders exactly one checkbox per Markdown task list item without duplicates', () => {
		const html = renderMemoMarkdown('- [ ] Mic\n- [x] Bendera');
		const checkboxMatches = html.match(/<input[^>]*type="checkbox"/g) ?? [];
		expect(checkboxMatches).toHaveLength(2);
		expect(html).toContain('data-task-index="0"');
		expect(html).toContain('data-task-index="1" checked');
	});
});

describe('smart code block detection & syntax highlighting', () => {
	it('auto-detects bash, python, rust, sql, json, and typescript when language is omitted', () => {
		expect(detectCodeLanguage('# Summarize\ngit show --stat HEAD')).toBe('bash');
		expect(detectCodeLanguage('def greet(name):\n    return f"hi {name}"')).toBe('python');
		expect(detectCodeLanguage('pub fn main() {\n    println!("ok");\n}')).toBe('rust');
		expect(detectCodeLanguage('SELECT id, content FROM memo WHERE id = 1;')).toBe('sql');
		expect(detectCodeLanguage('{"name": "xmemos", "offline": true}')).toBe('json');
		expect(detectCodeLanguage('interface User { id: number; name: string; }')).toBe('typescript');
	});

	it('highlights bash comments and CLI tools without corrupting # comments into #tags', () => {
		const md = '```bash\n# Summarize the last commit\ngit stash push -u -m "before refactor"\n```\n\n#dev/git';
		const html = renderMemoMarkdown(md);
		expect(html).toContain('class="xm-code-lang">bash</span>');
		expect(html).toContain('token comment');
		expect(html).toContain('data-tag="dev/git"');
		expect(html).not.toContain('data-tag="Summarize"');
	});
});

describe('custom timestamp helpers (past & future dates)', () => {
	it('parses and round-trips arbitrary past and future local timestamps', () => {
		const pastTs = parseLocalDateTimeInput('2020-01-15T08:30');
		const futureTs = parseLocalDateTimeInput('2028-12-31T23:45');
		expect(pastTs).toBeTypeOf('number');
		expect(futureTs).toBeTypeOf('number');
		expect(futureTs!).toBeGreaterThan(pastTs!);
		expect(formatLocalDateTimeInput(pastTs!)).toBe('2020-01-15T08:30');
		expect(formatLocalDateTimeInput(futureTs!)).toBe('2028-12-31T23:45');
	});

	it('combines a selected calendar date with current local clock time', () => {
		const refNow = new Date(2026, 10, 25, 14, 7, 0);
		expect(formatDateKeyWithCurrentTime('2024-06-03', refNow)).toBe('2024-06-03T14:07');
		expect(formatDateKeyWithCurrentTime('2027-08-19', refNow)).toBe('2027-08-19T14:07');
	});
});

describe('highlightHtmlMatches (Official Memos search highlighting)', () => {
	it('highlights case-insensitive substring matches without corrupting HTML attributes or URLs', () => {
		const html = '<p>TestMu AI at <a href="https://testmuai.com">testmuai.com</a></p>';
		const out = highlightHtmlMatches(html, ['test']);
		expect(out).toContain('<mark class="xm-search-mark">Test</mark>Mu AI');
		expect(out).toContain('href="https://testmuai.com"');
		expect(out).toContain('><mark class="xm-search-mark">test</mark>muai.com</a>');
	});

	it('handles regex metacharacters safely and skips SVG content', () => {
		const html = '<p>Price is $100 (USD)</p><svg><text>$100</text></svg>';
		const out = highlightHtmlMatches(html, ['$100', '(USD)']);
		expect(out).toContain('<mark class="xm-search-mark">$100</mark>');
		expect(out).toContain('<mark class="xm-search-mark">(USD)</mark>');
		expect(out).toContain('<svg><text>$100</text></svg>');
	});
});

describe('geocode place formatting', () => {
	it('detects empty and coordinate-only location labels', () => {
		expect(isCoordinateOnlyLabel('')).toBe(true);
		expect(isCoordinateOnlyLabel('-6.9147, 107.6098')).toBe(true);
		expect(isCoordinateOnlyLabel('37.995400°, -123.020400°')).toBe(true);
		expect(isCoordinateOnlyLabel('Point Reyes, California, United States')).toBe(false);
	});

	it('formats Nominatim address into concise deduplicated region name', () => {
		expect(
			formatNominatimPlace({
				address: {
					village: 'Point Reyes',
					state: 'California',
					country: 'United States'
				}
			})
		).toBe('Point Reyes, California, United States');
		expect(
			formatNominatimPlace({
				address: {
					city: 'Bandung',
					state: 'Jawa Barat',
					country: 'Indonesia'
				}
			})
		).toBe('Bandung, Jawa Barat, Indonesia');
	});

	it('formats Photon OpenStreetMap fallback response into concise region name', () => {
		expect(
			formatPhotonPlace({
				features: [{ properties: { name: 'Timbedra', state: 'Hodh Ech Chargui', country: 'Mauritania' } }]
			})
		).toBe('Timbedra, Hodh Ech Chargui, Mauritania');
	});

	it('formats coordinates in Official Memos degree notation', () => {
		expect(formatCoordinates(37.9954, -123.0204)).toBe('37.995400°, -123.020400°');
	});
});
