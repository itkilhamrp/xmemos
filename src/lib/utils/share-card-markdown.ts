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

export type InlineRunKind = 'text' | 'bold' | 'italic' | 'strike' | 'code' | 'tag' | 'link';

export interface InlineRun {
	kind: InlineRunKind;
	text: string;
}

export type ShareCardBlock =
	| { type: 'heading'; level: 1 | 2 | 3; runs: InlineRun[] }
	| { type: 'paragraph'; runs: InlineRun[] }
	| { type: 'blockquote'; runs: InlineRun[] }
	| { type: 'task'; checked: boolean; runs: InlineRun[] }
	| { type: 'list'; marker: string; runs: InlineRun[] }
	| { type: 'code'; lang: string; lines: string[] }
	| { type: 'hr' };

const INLINE_TOKEN_RE =
	/(`[^`\n]+`)|(\[([^\]\n]+)\]\(([^)\n]+)\))|(\*\*([^*\n]+)\*\*|__([^_\n]+)__)|(~~([^~\n]+)~~)|(\*([^*\n]+)\*|_([^_\n]+)_)|((?:^|(?<=[\s(\[{,;]))#[\p{L}\p{N}_\-/]+[\p{L}\p{N}])/gu;

/**
 * Splits a single line of Markdown text into typed inline style runs
 * matching MemoContent.svelte (.memo-prose) rendering.
 */
export function parseInlineRuns(raw: string): InlineRun[] {
	if (!raw) return [];
	const runs: InlineRun[] = [];
	let lastIndex = 0;

	INLINE_TOKEN_RE.lastIndex = 0;
	let match: RegExpExecArray | null;
	while ((match = INLINE_TOKEN_RE.exec(raw)) !== null) {
		if (match.index > lastIndex) {
			runs.push({ kind: 'text', text: raw.slice(lastIndex, match.index) });
		}

		const full = match[0];
		if (match[1]) {
			runs.push({ kind: 'code', text: full.slice(1, -1) });
		} else if (match[2]) {
			runs.push({ kind: 'link', text: match[3] ?? full });
		} else if (match[5]) {
			runs.push({ kind: 'bold', text: match[6] ?? match[7] ?? '' });
		} else if (match[8]) {
			runs.push({ kind: 'strike', text: match[9] ?? '' });
		} else if (match[10]) {
			runs.push({ kind: 'italic', text: match[11] ?? match[12] ?? '' });
		} else if (match[13]) {
			runs.push({ kind: 'tag', text: match[13] });
		} else {
			runs.push({ kind: 'text', text: full });
		}

		lastIndex = match.index + full.length;
	}

	if (lastIndex < raw.length) {
		runs.push({ kind: 'text', text: raw.slice(lastIndex) });
	}
	return runs;
}

/**
 * Parses Memo Markdown into structured visual blocks so the PNG share card
 * mirrors MemoContent.svelte 1:1 (headings, tasks, lists, quotes, code blocks, tags).
 */
export function parseShareCardBlocks(content: string): ShareCardBlock[] {
	const rawLines = content.replace(/\r\n/g, '\n').split('\n');
	const blocks: ShareCardBlock[] = [];
	let i = 0;

	while (i < rawLines.length) {
		const line = rawLines[i] ?? '';
		const trimmed = line.trim();

		if (!trimmed) {
			i += 1;
			continue;
		}

		// Fenced code block
		const fenceMatch = /^```([^\s`]*)/.exec(trimmed);
		if (fenceMatch) {
			const lang = (fenceMatch[1] ?? '').trim();
			const codeLines: string[] = [];
			i += 1;
			while (i < rawLines.length) {
				const cur = rawLines[i] ?? '';
				if (/^```\s*$/.test(cur.trim())) {
					i += 1;
					break;
				}
				codeLines.push(cur);
				i += 1;
			}
			blocks.push({ type: 'code', lang, lines: codeLines.length > 0 ? codeLines : [''] });
			continue;
		}

		// Horizontal rule
		if (/^(?:[-*_]\s*){3,}$/.test(trimmed)) {
			blocks.push({ type: 'hr' });
			i += 1;
			continue;
		}

		// Heading (#, ##, ###)
		const headingMatch = /^(#{1,6})\s+(.+)$/.exec(trimmed);
		if (headingMatch) {
			const rawLevel = headingMatch[1]?.length ?? 1;
			const level: 1 | 2 | 3 = rawLevel <= 1 ? 1 : rawLevel === 2 ? 2 : 3;
			blocks.push({ type: 'heading', level, runs: parseInlineRuns(headingMatch[2] ?? '') });
			i += 1;
			continue;
		}

		// Task list item (- [ ] or - [x])
		const taskMatch = /^[-*+]\s+\[([ xX])\]\s*(.*)$/.exec(trimmed);
		if (taskMatch) {
			const checked = (taskMatch[1] ?? '').toLowerCase() === 'x';
			blocks.push({ type: 'task', checked, runs: parseInlineRuns(taskMatch[2] ?? '') });
			i += 1;
			continue;
		}

		// Unordered list item
		const ulMatch = /^[-*+]\s+(.+)$/.exec(trimmed);
		if (ulMatch) {
			blocks.push({ type: 'list', marker: '•', runs: parseInlineRuns(ulMatch[1] ?? '') });
			i += 1;
			continue;
		}

		// Ordered list item
		const olMatch = /^(\d+\.)\s+(.+)$/.exec(trimmed);
		if (olMatch) {
			blocks.push({
				type: 'list',
				marker: olMatch[1] ?? '1.',
				runs: parseInlineRuns(olMatch[2] ?? '')
			});
			i += 1;
			continue;
		}

		// Blockquote (merge consecutive > lines)
		if (trimmed.startsWith('>')) {
			const quoteParts: string[] = [];
			while (i < rawLines.length) {
				const qLine = (rawLines[i] ?? '').trim();
				if (!qLine.startsWith('>')) break;
				quoteParts.push(qLine.replace(/^>\s?/, ''));
				i += 1;
			}
			blocks.push({ type: 'blockquote', runs: parseInlineRuns(quoteParts.join(' ')) });
			continue;
		}

		// Standard paragraph (keep single-line breaks clean)
		blocks.push({ type: 'paragraph', runs: parseInlineRuns(trimmed) });
		i += 1;
	}

	return blocks;
}
