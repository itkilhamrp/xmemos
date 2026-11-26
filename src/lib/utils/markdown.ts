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

import { marked } from 'marked';
import DOMPurify from 'dompurify';

import { escapeHtml, highlightCodeBlock } from './code-highlight.ts';

const MAX_RENDER_CACHE = 250;
const renderCache = new Map<string, string>();

type KatexLike = {
	renderToString: (tex: string, opts: { displayMode: boolean; throwOnError: boolean }) => string;
};

let katexEngine: KatexLike | null = null;
let katexLoadingPromise: Promise<void> | null = null;
const katexListeners = new Set<() => void>();

export function onKatexReady(listener: () => void): () => void {
	katexListeners.add(listener);
	return () => {
		katexListeners.delete(listener);
	};
}

function triggerLazyKatexLoad(): void {
	if (katexEngine || katexLoadingPromise || typeof window === 'undefined') return;
	katexLoadingPromise = Promise.all([import('katex'), import('katex/dist/katex.min.css')])
		.then(([mod]) => {
			katexEngine = mod.default;
			renderCache.clear();
			for (const cb of katexListeners) cb();
		})
		.catch(() => {
			katexLoadingPromise = null;
		});
}

function renderMathExpressions(input: string): string {
	if (!input.includes('$')) return input;

	// Preserve fenced and inline code segments so `$` inside shell/code blocks is never parsed as KaTeX
	const segments = input.split(/(```[\s\S]*?```|`[^`\n]+`)/g);
	return segments
		.map((seg, idx) => {
			if (idx % 2 === 1) return seg;

			// Block math: $$ ... $$
			let out = seg.replace(/\$\$([\s\S]+?)\$\$/g, (_match, expr: string) => {
				if (!katexEngine) {
					triggerLazyKatexLoad();
					return `<code>${escapeHtml(expr.trim())}</code>`;
				}
				try {
					return `<div class="my-2 overflow-x-auto">${katexEngine.renderToString(expr.trim(), {
						displayMode: true,
						throwOnError: false
					})}</div>`;
				} catch {
					return `<code>${escapeHtml(expr)}</code>`;
				}
			});

			// Inline math: $...$
			out = out.replace(/(?<!\$)\$([^$\n]+?)\$(?!\$)/g, (_match, expr: string) => {
				if (!katexEngine) {
					triggerLazyKatexLoad();
					return `<code>${escapeHtml(expr.trim())}</code>`;
				}
				try {
					return katexEngine.renderToString(expr.trim(), {
						displayMode: false,
						throwOnError: false
					});
				} catch {
					return `<code>${escapeHtml(expr)}</code>`;
				}
			});

			return out;
		})
		.join('');
}

/**
 * Highlights #tags strictly outside <pre>, <code>, and HTML attributes so code comments
 * like `# Summarize the last commit` or `#include <stdio.h>` are never corrupted.
 */
function highlightProseTags(html: string): string {
	const parts = html.split(/(<pre[\s\S]*?<\/pre>|<code[\s\S]*?<\/code>|<[^>]+>)/g);
	return parts
		.map((part, idx) => {
			if (idx % 2 === 1) return part;
			return part.replace(
				/(^|[\s(])#([\p{L}\p{N}_\-/]+)/gu,
				(_m, prefix: string, tag: string) =>
					`${prefix}<button type="button" class="xm-tag" data-tag="${escapeHtml(tag)}">#${escapeHtml(tag)}</button>`
			);
		})
		.join('');
}

/**
 * Renders GFM Markdown + KaTeX math + smart multi-language PrismJS code highlighting
 * + clickable #tags + indexed interactive task checkboxes, strictly sanitized via DOMPurify,
 * with a bounded LRU cache for instant 60/120fps timeline scrolling.
 */
export function renderMemoMarkdown(markdown: string): string {
	const cacheKey = `${typeof window === 'undefined' ? 'ssr' : 'dom'}:${markdown}`;
	const cached = renderCache.get(cacheKey);
	if (cached !== undefined) {
		renderCache.delete(cacheKey);
		renderCache.set(cacheKey, cached);
		return cached;
	}

	let taskIndex = 0;

	const renderer = new marked.Renderer();
	renderer.code = ({ text, lang }) => {
		const { language, displayLabel, html } = highlightCodeBlock(text, lang);
		const safeLang = escapeHtml(language || 'text');
		const safeLabel = escapeHtml(displayLabel || 'text');
		return `<div class="xm-code-block"><div class="xm-code-header"><span class="xm-code-lang">${safeLabel}</span><button type="button" class="xm-code-copy" data-copy-code="1" aria-label="Copy code">Copy</button></div><pre class="language-${safeLang}"><code class="language-${safeLang}">${html}</code></pre></div>`;
	};

	// Suppress marked's built-in disabled <input type="checkbox"> so we never render duplicate boxes
	renderer.checkbox = () => '';

	renderer.listitem = function (token) {
		const contentTokens = token.task
			? token.tokens.filter((t) => t.type !== 'checkbox')
			: token.tokens;
		const rawInner = this.parser.parse(contentTokens).trim();
		const inner = rawInner
			.replace(/^<p>([\s\S]*?)<\/p>$/, '$1')
			.replace(/<input(?![^>]*class="xm-task-checkbox")\b[^>]*type=["']checkbox["'][^>]*>\s*/gi, '')
			.trim();
		if (token.task) {
			const idx = taskIndex++;
			const checkedAttr = token.checked ? ' checked' : '';
			return `<li class="xm-task-item"><input type="checkbox" class="xm-task-checkbox" data-task-index="${idx}"${checkedAttr} /><span class="xm-task-label">${inner}</span></li>`;
		}
		return `<li>${inner}</li>`;
	};

	const withMath = renderMathExpressions(markdown);
	const rawHtml = marked.parse(withMath, {
		gfm: true,
		breaks: true,
		renderer
	}) as string;

	const withTags = highlightProseTags(rawHtml);

	const result =
		typeof window === 'undefined'
			? withTags
			: DOMPurify.sanitize(withTags, {
					ADD_TAGS: [
						'input',
						'button',
						'math',
						'semantics',
						'mrow',
						'mi',
						'mo',
						'mn',
						'msup',
						'msub',
						'mfrac',
						'annotation'
					],
					ADD_ATTR: [
						'type',
						'checked',
						'data-task-index',
						'data-tag',
						'data-copy-code',
						'class',
						'aria-hidden',
						'aria-label',
						'xmlns'
					]
				});

	if (renderCache.size >= MAX_RENDER_CACHE) {
		const oldestKey = renderCache.keys().next().value;
		if (oldestKey !== undefined) renderCache.delete(oldestKey);
	}
	renderCache.set(cacheKey, result);
	return result;
}

export function stripMarkdownToPlainText(markdown: string): string {
	return markdown
		.replace(/```[\s\S]*?```/g, (m) => m.replace(/```[a-zA-Z0-9_-]*\n?/g, '').trim())
		.replace(/`([^`]+)`/g, '$1')
		.replace(/!\[([^\]]*)\]\([^)]+\)/g, '$1')
		.replace(/\[([^\]]+)\]\([^)]+\)/g, '$1')
		.replace(/^#{1,6}\s+/gm, '')
		.replace(/[*_~>]/g, '')
		.trim();
}

function escapeRegExp(raw: string): string {
	return raw.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

/**
 * Safely highlights case-insensitive search term matches inside rendered HTML text nodes ONLY.
 * Never mutates HTML tags, attributes, href URLs, SVG icons, or KaTeX <math> nodes.
 */
export function highlightHtmlMatches(html: string, terms: string[]): string {
	const cleaned = Array.from(
		new Set(
			terms
				.flatMap((t) => t.trim().split(/\s+/))
				.map((t) => t.trim())
				.filter((t) => t.length > 0)
		)
	).sort((a, b) => b.length - a.length);

	if (cleaned.length === 0) return html;

	const pattern = new RegExp(`(${cleaned.map(escapeRegExp).join('|')})`, 'gi');
	const parts = html.split(/(<[^>]+>)/g);
	let skipDepth = 0;

	for (let i = 0; i < parts.length; i++) {
		const part = parts[i] ?? '';
		if (!part) continue;
		if (part.startsWith('<')) {
			const lower = part.toLowerCase();
			if (/^<(svg|math|script|style)(\s|>)/.test(lower) && !lower.endsWith('/>')) {
				skipDepth += 1;
			} else if (/^<\/(svg|math|script|style)>/.test(lower) && skipDepth > 0) {
				skipDepth -= 1;
			}
			continue;
		}
		if (skipDepth > 0) continue;
		parts[i] = part.replace(pattern, '<mark class="xm-search-mark">$1</mark>');
	}

	return parts.join('');
}
