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

import { getMediaUrl } from '#lib/api/invoke.ts';
import type { MemoView, MemoVisibility } from '#lib/types/memo.ts';
import { formatFullTimestamp } from './date';
import {
	parseShareCardBlocks,
	type InlineRun,
	type InlineRunKind,
	type ShareCardBlock
} from './share-card-markdown';

const FONT_SANS =
	'-apple-system, BlinkMacSystemFont, "SF Pro Text", "SF UI Text", system-ui, "Helvetica Neue", sans-serif';
const FONT_MONO = 'ui-monospace, "SF Mono", Menlo, Monaco, Consolas, monospace';

interface CardTheme {
	bgBase: string;
	meshA: string;
	meshB: string;
	meshC: string;
	cardFill: string;
	cardBorder: string;
	cardSpecular: string;
	cardShadow: string;
	textPrimary: string;
	textSecondary: string;
	textMuted: string;
	accent: string;
	accentSoft: string;
	accentBorder: string;
	accentBtn: string;
	surfaceSubtle: string;
	divider: string;
	codeBlockBg: string;
	codeBlockBorder: string;
	codeLang: string;
	codeText: string;
	inlineCodeBg: string;
	inlineCodeText: string;
	checkboxBorder: string;
	pinAmber: string;
	chipBg: string;
	chipBorder: string;
}

function getTheme(isDark: boolean): CardTheme {
	if (isDark) {
		return {
			bgBase: '#000000',
			meshA: 'rgba(14, 165, 233, 0.20)',
			meshB: 'rgba(99, 102, 241, 0.16)',
			meshC: 'rgba(20, 184, 166, 0.11)',
			cardFill: 'rgba(28, 28, 30, 0.88)',
			cardBorder: 'rgba(255, 255, 255, 0.12)',
			cardSpecular: 'rgba(255, 255, 255, 0.14)',
			cardShadow: 'rgba(0, 0, 0, 0.56)',
			textPrimary: '#f8fafc',
			textSecondary: '#cbd5e1',
			textMuted: '#94a3b8',
			accent: '#60a5fa',
			accentSoft: 'rgba(59, 158, 255, 0.18)',
			accentBorder: 'rgba(96, 165, 250, 0.35)',
			accentBtn: '#0a84ff',
			surfaceSubtle: 'rgba(255, 255, 255, 0.07)',
			divider: 'rgba(255, 255, 255, 0.12)',
			codeBlockBg: '#222326',
			codeBlockBorder: 'rgba(255, 255, 255, 0.08)',
			codeLang: '#8b949e',
			codeText: '#e6edf3',
			inlineCodeBg: 'rgba(255, 255, 255, 0.10)',
			inlineCodeText: '#93c5fd',
			checkboxBorder: 'rgba(255, 255, 255, 0.30)',
			pinAmber: '#fbbf24',
			chipBg: 'rgba(255, 255, 255, 0.06)',
			chipBorder: 'rgba(255, 255, 255, 0.14)'
		};
	}

	return {
		bgBase: '#f2f2f7',
		meshA: 'rgba(56, 189, 248, 0.19)',
		meshB: 'rgba(99, 102, 241, 0.13)',
		meshC: 'rgba(14, 165, 233, 0.11)',
		cardFill: 'rgba(255, 255, 255, 0.90)',
		cardBorder: 'rgba(15, 23, 42, 0.10)',
		cardSpecular: 'rgba(255, 255, 255, 0.95)',
		cardShadow: 'rgba(15, 23, 42, 0.10)',
		textPrimary: '#0f172a',
		textSecondary: '#334155',
		textMuted: '#475569',
		accent: '#0066cc',
		accentSoft: 'rgba(0, 102, 204, 0.12)',
		accentBorder: 'rgba(0, 102, 204, 0.28)',
		accentBtn: '#0066cc',
		surfaceSubtle: 'rgba(15, 23, 42, 0.05)',
		divider: 'rgba(15, 23, 42, 0.10)',
		codeBlockBg: '#efefe9',
		codeBlockBorder: 'rgba(15, 23, 42, 0.09)',
		codeLang: '#6e7781',
		codeText: '#1f2328',
		inlineCodeBg: 'rgba(15, 23, 42, 0.06)',
		inlineCodeText: '#0052a3',
		checkboxBorder: 'rgba(15, 23, 42, 0.26)',
		pinAmber: '#fbbf24',
		chipBg: 'rgba(15, 23, 42, 0.04)',
		chipBorder: 'rgba(15, 23, 42, 0.12)'
	};
}

interface PositionedSegment {
	kind: InlineRunKind;
	text: string;
	width: number;
	font: string;
	padX: number;
}

interface WrappedRichLine {
	segments: PositionedSegment[];
}

function fontForRun(kind: InlineRunKind, baseSize: number, baseWeight: number): string {
	switch (kind) {
		case 'bold':
			return `700 ${baseSize}px ${FONT_SANS}`;
		case 'italic':
			return `italic ${baseWeight} ${baseSize}px ${FONT_SANS}`;
		case 'code':
			return `500 ${Math.round(baseSize * 0.88)}px ${FONT_MONO}`;
		case 'tag':
			return `600 ${Math.round(baseSize * 0.92)}px ${FONT_SANS}`;
		case 'link':
			return `500 ${baseSize}px ${FONT_SANS}`;
		default:
			return `${baseWeight} ${baseSize}px ${FONT_SANS}`;
	}
}

function wrapRichRuns(
	ctx: CanvasRenderingContext2D,
	runs: InlineRun[],
	maxWidth: number,
	baseSize: number,
	baseWeight = 400
): WrappedRichLine[] {
	const tokens: PositionedSegment[] = [];

	for (const run of runs) {
		const font = fontForRun(run.kind, baseSize, baseWeight);
		ctx.font = font;

		if (run.kind === 'code' || run.kind === 'tag') {
			const padX = run.kind === 'tag' ? 12 : 8;
			const w = ctx.measureText(run.text).width + padX * 2;
			tokens.push({ kind: run.kind, text: run.text, width: w, font, padX });
			continue;
		}

		// Preserve spaces while splitting wrap points cleanly
		const parts = run.text.split(/(\s+)/);
		for (const part of parts) {
			if (!part) continue;
			const w = ctx.measureText(part).width;
			if (w <= maxWidth || /^\s+$/.test(part)) {
				tokens.push({ kind: run.kind, text: part, width: w, font, padX: 0 });
			} else {
				// Break long unspaced tokens (URLs/hashes) cleanly by character
				let chunk = '';
				for (const ch of part) {
					const next = chunk + ch;
					if (ctx.measureText(next).width > maxWidth && chunk) {
						tokens.push({
							kind: run.kind,
							text: chunk,
							width: ctx.measureText(chunk).width,
							font,
							padX: 0
						});
						chunk = ch;
					} else {
						chunk = next;
					}
				}
				if (chunk) {
					tokens.push({
						kind: run.kind,
						text: chunk,
						width: ctx.measureText(chunk).width,
						font,
						padX: 0
					});
				}
			}
		}
	}

	const lines: WrappedRichLine[] = [];
	let current: PositionedSegment[] = [];
	let curWidth = 0;

	for (const tok of tokens) {
		const isSpace = tok.kind === 'text' && /^\s+$/.test(tok.text);
		if (current.length === 0 && isSpace) continue;

		if (curWidth + tok.width > maxWidth && current.length > 0) {
			lines.push({ segments: current });
			current = isSpace ? [] : [tok];
			curWidth = isSpace ? 0 : tok.width;
		} else {
			current.push(tok);
			curWidth += tok.width;
		}
	}

	if (current.length > 0) {
		lines.push({ segments: current });
	}
	return lines.length > 0 ? lines : [{ segments: [] }];
}

function wrapCodeLines(
	ctx: CanvasRenderingContext2D,
	lines: string[],
	maxWidth: number,
	fontSize: number
): string[] {
	ctx.font = `400 ${fontSize}px ${FONT_MONO}`;
	const out: string[] = [];
	for (const raw of lines) {
		const expanded = raw.replace(/\t/g, '  ');
		if (!expanded) {
			out.push('');
			continue;
		}
		if (ctx.measureText(expanded).width <= maxWidth) {
			out.push(expanded);
			continue;
		}
		let chunk = '';
		for (const ch of expanded) {
			const next = chunk + ch;
			if (ctx.measureText(next).width > maxWidth && chunk) {
				out.push(chunk);
				chunk = ch;
			} else {
				chunk = next;
			}
		}
		if (chunk) out.push(chunk);
	}
	return out;
}

function drawRichLine(
	ctx: CanvasRenderingContext2D,
	line: WrappedRichLine,
	x: number,
	baselineY: number,
	baseSize: number,
	theme: CardTheme,
	textColor: string,
	mutedStrike = false
): void {
	let cursorX = x;

	for (const seg of line.segments) {
		ctx.font = seg.font;

		if (seg.kind === 'tag') {
			const pillH = Math.round(baseSize * 1.24);
			const pillY = baselineY - Math.round(baseSize * 0.94);
			ctx.save();
			ctx.beginPath();
			ctx.roundRect(cursorX, pillY, seg.width, pillH, pillH / 2);
			ctx.fillStyle = theme.accentSoft;
			ctx.fill();
			ctx.fillStyle = theme.accent;
			ctx.fillText(seg.text, cursorX + seg.padX, baselineY);
			ctx.restore();
			cursorX += seg.width;
			continue;
		}

		if (seg.kind === 'code') {
			const pillH = Math.round(baseSize * 1.18);
			const pillY = baselineY - Math.round(baseSize * 0.90);
			ctx.save();
			ctx.beginPath();
			ctx.roundRect(cursorX, pillY, seg.width, pillH, 10);
			ctx.fillStyle = theme.inlineCodeBg;
			ctx.fill();
			ctx.fillStyle = theme.inlineCodeText;
			ctx.fillText(seg.text, cursorX + seg.padX, baselineY - 1);
			ctx.restore();
			cursorX += seg.width;
			continue;
		}

		ctx.fillStyle = seg.kind === 'link' ? theme.accent : textColor;
		ctx.fillText(seg.text, cursorX, baselineY);

		if (seg.kind === 'link') {
			ctx.save();
			ctx.strokeStyle = theme.accent;
			ctx.lineWidth = 2;
			ctx.beginPath();
			ctx.moveTo(cursorX, baselineY + 4);
			ctx.lineTo(cursorX + seg.width, baselineY + 4);
			ctx.stroke();
			ctx.restore();
		} else if (seg.kind === 'strike' || mutedStrike) {
			ctx.save();
			ctx.strokeStyle = textColor;
			ctx.lineWidth = 2;
			const strikeY = baselineY - Math.round(baseSize * 0.30);
			ctx.beginPath();
			ctx.moveTo(cursorX, strikeY);
			ctx.lineTo(cursorX + seg.width, strikeY);
			ctx.stroke();
			ctx.restore();
		}

		cursorX += seg.width;
	}
}

function drawVisibilityIcon(
	ctx: CanvasRenderingContext2D,
	visibility: MemoVisibility,
	cx: number,
	cy: number,
	color: string
): void {
	ctx.save();
	ctx.strokeStyle = color;
	ctx.lineWidth = 2.4;
	ctx.lineCap = 'round';
	ctx.lineJoin = 'round';

	if (visibility === 'PUBLIC') {
		// Lucide Globe icon
		const r = 13;
		ctx.beginPath();
		ctx.arc(cx, cy, r, 0, Math.PI * 2);
		ctx.stroke();
		ctx.beginPath();
		ctx.moveTo(cx - r, cy);
		ctx.lineTo(cx + r, cy);
		ctx.stroke();
		ctx.beginPath();
		ctx.ellipse(cx, cy, r * 0.48, r, 0, 0, Math.PI * 2);
		ctx.stroke();
	} else if (visibility === 'PROTECTED') {
		// Lucide Users icon
		ctx.beginPath();
		ctx.arc(cx - 4, cy - 5, 4.8, 0, Math.PI * 2);
		ctx.stroke();
		ctx.beginPath();
		ctx.arc(cx - 4, cy + 12, 9, Math.PI * 1.12, Math.PI * 1.88);
		ctx.stroke();
		ctx.beginPath();
		ctx.arc(cx + 6.5, cy - 4, 3.6, -Math.PI * 0.4, Math.PI * 0.45);
		ctx.stroke();
		ctx.beginPath();
		ctx.arc(cx + 6, cy + 12, 7.5, Math.PI * 1.58, Math.PI * 1.92);
		ctx.stroke();
	} else {
		// Lucide Lock icon
		ctx.beginPath();
		ctx.roundRect(cx - 10, cy - 1, 20, 14, 3.5);
		ctx.stroke();
		ctx.beginPath();
		ctx.arc(cx, cy - 2, 6, Math.PI, 0);
		ctx.stroke();
	}

	ctx.restore();
}

function drawMapPinIcon(ctx: CanvasRenderingContext2D, x: number, y: number, color: string): void {
	ctx.save();
	ctx.strokeStyle = color;
	ctx.lineWidth = 2.2;
	ctx.lineCap = 'round';
	ctx.lineJoin = 'round';
	ctx.beginPath();
	ctx.arc(x + 10, y - 9, 7, Math.PI * 0.82, Math.PI * 2.18);
	ctx.lineTo(x + 10, y + 3);
	ctx.closePath();
	ctx.stroke();
	ctx.beginPath();
	ctx.arc(x + 10, y - 9, 2.4, 0, Math.PI * 2);
	ctx.stroke();
	ctx.restore();
}

function drawLinkIcon(ctx: CanvasRenderingContext2D, x: number, y: number, color: string): void {
	ctx.save();
	ctx.strokeStyle = color;
	ctx.lineWidth = 2.2;
	ctx.lineCap = 'round';
	ctx.beginPath();
	ctx.roundRect(x + 1, y - 12, 10, 8, 4);
	ctx.stroke();
	ctx.beginPath();
	ctx.roundRect(x + 11, y - 12, 10, 8, 4);
	ctx.stroke();
	ctx.beginPath();
	ctx.moveTo(x + 8, y - 8);
	ctx.lineTo(x + 14, y - 8);
	ctx.stroke();
	ctx.restore();
}

function drawCommentIcon(ctx: CanvasRenderingContext2D, x: number, y: number, color: string): void {
	ctx.save();
	ctx.strokeStyle = color;
	ctx.lineWidth = 2.2;
	ctx.lineCap = 'round';
	ctx.lineJoin = 'round';
	ctx.beginPath();
	ctx.arc(x + 10, y - 7, 9, Math.PI * 0.85, Math.PI * 2.65);
	ctx.lineTo(x + 2, y + 2);
	ctx.closePath();
	ctx.stroke();
	ctx.restore();
}

function drawMiniAppIcon(ctx: CanvasRenderingContext2D, x: number, y: number, size: number): void {
	ctx.save();
	const r = size * 0.24;
	// Liquid Glass Sapphire Squircle
	const bg = ctx.createLinearGradient(x, y, x + size, y + size);
	bg.addColorStop(0, '#3AC0FF');
	bg.addColorStop(0.45, '#0A84FF');
	bg.addColorStop(1, '#06287A');
	ctx.beginPath();
	ctx.roundRect(x, y, size, size, r);
	ctx.fillStyle = bg;
	ctx.fill();
	ctx.lineWidth = 1.2;
	ctx.strokeStyle = 'rgba(255,255,255,0.45)';
	ctx.stroke();

	// Mini Frosted Memo Card
	const cw = size * 0.54;
	const ch = size * 0.60;
	const cx = x + (size - cw) / 2;
	const cy = y + (size - ch) / 2;
	ctx.beginPath();
	ctx.roundRect(cx, cy, cw, ch, size * 0.10);
	ctx.fillStyle = 'rgba(255, 255, 255, 0.95)';
	ctx.fill();

	// Mini Precision X
	const pad = cw * 0.26;
	ctx.strokeStyle = '#0A84FF';
	ctx.lineWidth = Math.max(2.2, size * 0.085);
	ctx.lineCap = 'round';
	ctx.beginPath();
	ctx.moveTo(cx + pad, cy + pad + ch * 0.04);
	ctx.lineTo(cx + cw - pad, cy + ch - pad);
	ctx.moveTo(cx + cw - pad, cy + pad + ch * 0.04);
	ctx.lineTo(cx + pad, cy + ch - pad);
	ctx.stroke();
	ctx.restore();
}

async function loadImageSafe(url: string): Promise<HTMLImageElement | null> {
	try {
		const res = await fetch(url);
		if (!res.ok) return null;
		const blob = await res.blob();
		const objectUrl = URL.createObjectURL(blob);
		try {
			return await new Promise<HTMLImageElement | null>((resolve) => {
				const img = new Image();
				img.onload = () => resolve(img);
				img.onerror = () => resolve(null);
				img.src = objectUrl;
			});
		} finally {
			URL.revokeObjectURL(objectUrl);
		}
	} catch {
		return null;
	}
}

function drawCoverImage(
	ctx: CanvasRenderingContext2D,
	img: HTMLImageElement,
	x: number,
	y: number,
	w: number,
	h: number,
	radius: number
): void {
	ctx.save();
	ctx.beginPath();
	ctx.roundRect(x, y, w, h, radius);
	ctx.clip();

	const imgRatio = img.naturalWidth / Math.max(1, img.naturalHeight);
	const boxRatio = w / Math.max(1, h);
	let drawW = w;
	let drawH = h;
	let drawX = x;
	let drawY = y;

	if (imgRatio > boxRatio) {
		drawW = h * imgRatio;
		drawX = x - (drawW - w) / 2;
	} else {
		drawH = w / imgRatio;
		drawY = y - (drawH - h) / 2;
	}

	ctx.drawImage(img, drawX, drawY, drawW, drawH);
	ctx.restore();
}

interface LayoutBlock {
	block: ShareCardBlock;
	height: number;
	gapAfter: number;
	richLines?: WrappedRichLine[];
	codeLines?: string[];
	fontSize?: number;
	lineHeight?: number;
}

/**
 * Renders a memo as a crisp 1:1 MemoCard Liquid Glass PNG share card using HTML5 Canvas 2D.
 * Matches the in-app MemoCard layout, typography, colors, pin corner fold, visibility icon,
 * rich Markdown blocks, image attachments, location/relations metadata, and reactions.
 */
export async function renderMemoShareCardPng(view: MemoView, isDark: boolean): Promise<Blob> {
	const theme = getTheme(isDark);
	const width = 1120;
	const outerPad = 64;
	const cardW = width - outerPad * 2;
	const cardPadX = 48;
	const cardPadTop = 44;
	const cardPadBottom = 38;
	const contentW = cardW - cardPadX * 2;

	const measureCanvas = document.createElement('canvas');
	const mCtx = measureCanvas.getContext('2d');
	if (!mCtx) throw new Error('Canvas 2D context unavailable');

	// 1. Measure Markdown blocks (matching .memo-prose at 2x scale: 15px -> 30px)
	const parsedBlocks = parseShareCardBlocks(view.memo.content);
	const layoutBlocks: LayoutBlock[] = [];

	for (const block of parsedBlocks) {
		if (block.type === 'hr') {
			layoutBlocks.push({ block, height: 16, gapAfter: 18 });
			continue;
		}
		if (block.type === 'heading') {
			const fontSize = block.level === 1 ? 40 : block.level === 2 ? 36 : 32;
			const lineHeight = Math.round(fontSize * 1.32);
			const richLines = wrapRichRuns(mCtx, block.runs, contentW, fontSize, 700);
			layoutBlocks.push({
				block,
				richLines,
				fontSize,
				lineHeight,
				height: richLines.length * lineHeight,
				gapAfter: 16
			});
			continue;
		}
		if (block.type === 'paragraph') {
			const fontSize = 30;
			const lineHeight = 48;
			const richLines = wrapRichRuns(mCtx, block.runs, contentW, fontSize, 400);
			layoutBlocks.push({
				block,
				richLines,
				fontSize,
				lineHeight,
				height: richLines.length * lineHeight,
				gapAfter: 16
			});
			continue;
		}
		if (block.type === 'blockquote') {
			const fontSize = 29;
			const lineHeight = 46;
			const richLines = wrapRichRuns(mCtx, block.runs, contentW - 44, fontSize, 400);
			layoutBlocks.push({
				block,
				richLines,
				fontSize,
				lineHeight,
				height: richLines.length * lineHeight + 28,
				gapAfter: 18
			});
			continue;
		}
		if (block.type === 'task') {
			const fontSize = 30;
			const lineHeight = 46;
			const richLines = wrapRichRuns(mCtx, block.runs, contentW - 48, fontSize, 400);
			layoutBlocks.push({
				block,
				richLines,
				fontSize,
				lineHeight,
				height: richLines.length * lineHeight,
				gapAfter: 12
			});
			continue;
		}
		if (block.type === 'list') {
			const fontSize = 30;
			const lineHeight = 46;
			const richLines = wrapRichRuns(mCtx, block.runs, contentW - 42, fontSize, 400);
			layoutBlocks.push({
				block,
				richLines,
				fontSize,
				lineHeight,
				height: richLines.length * lineHeight,
				gapAfter: 12
			});
			continue;
		}
		if (block.type === 'code') {
			const fontSize = 25;
			const lineHeight = 38;
			const codeLines = wrapCodeLines(mCtx, block.lines, contentW - 56, fontSize);
			const headerH = block.lang ? 42 : 14;
			layoutBlocks.push({
				block,
				codeLines,
				fontSize,
				lineHeight,
				height: headerH + codeLines.length * lineHeight + 24,
				gapAfter: 20
			});
		}
	}

	// 2. Load up to 4 image attachments (best-effort, non-blocking on failure)
	const imageResources = view.resources.filter((r) => r.type.startsWith('image/'));
	const nonImageResources = view.resources.filter((r) => !r.type.startsWith('image/'));
	const loadedImages: HTMLImageElement[] = [];
	for (const res of imageResources.slice(0, 4)) {
		const img = await loadImageSafe(getMediaUrl(res.uid, false));
		if (img) loadedImages.push(img);
	}
	const extraImagesCount = Math.max(0, imageResources.length - loadedImages.length);

	// 3. Calculate vertical heights cleanly
	const headerH = 62;
	const headerGap = 24;

	let bodyH = 0;
	for (let idx = 0; idx < layoutBlocks.length; idx++) {
		const b = layoutBlocks[idx];
		if (!b) continue;
		bodyH += b.height + (idx < layoutBlocks.length - 1 ? b.gapAfter : 0);
	}
	if (bodyH === 0) bodyH = 44;

	let mediaH = 0;
	if (loadedImages.length === 1) {
		const first = loadedImages[0];
		const ratio = first ? first.naturalHeight / Math.max(1, first.naturalWidth) : 0.6;
		mediaH = Math.round(Math.min(520, Math.max(260, contentW * ratio)));
	} else if (loadedImages.length === 2) {
		mediaH = 300;
	} else if (loadedImages.length >= 3) {
		mediaH = 520;
	}
	const mediaGap = mediaH > 0 ? 24 : 0;

	const filePillsH = nonImageResources.length > 0 ? 46 : 0;
	const filePillsGap = filePillsH > 0 ? 18 : 0;

	const referenceRelations = view.relations.filter((r) => r.type === 'REFERENCE');
	const hasLocation = Boolean(view.memo.payload.location);
	const hasRelations = referenceRelations.length > 0;
	const metaRowsCount = (hasLocation ? 1 : 0) + (hasRelations ? 1 : 0);
	const metaH = metaRowsCount > 0 ? metaRowsCount * 36 : 0;
	const metaGap = metaH > 0 ? 22 : 0;

	// Group reactions (exact MemoCard logic)
	const reactionMap = new Map<string, { emoji: string; count: number }>();
	for (const r of view.reactions) {
		const prev = reactionMap.get(r.reactionType);
		if (prev) prev.count += 1;
		else reactionMap.set(r.reactionType, { emoji: r.reactionType, count: 1 });
	}
	const groupedReactions = Array.from(reactionMap.values());

	const footerGap = 28;
	const footerH = 48;

	const cardH = Math.max(
		300,
		cardPadTop +
			headerH +
			headerGap +
			bodyH +
			mediaGap +
			mediaH +
			filePillsGap +
			filePillsH +
			metaGap +
			metaH +
			footerGap +
			footerH +
			cardPadBottom
	);
	const height = cardH + outerPad * 2;

	// 4. Create HiDPI output canvas
	const canvas = document.createElement('canvas');
	canvas.width = width;
	canvas.height = height;
	const ctx = canvas.getContext('2d');
	if (!ctx) throw new Error('Canvas 2D context unavailable');

	// Ambient app mesh backdrop (exact --xm-bg-mesh from src/app.css)
	ctx.fillStyle = theme.bgBase;
	ctx.fillRect(0, 0, width, height);

	const drawMeshOrb = (cx: number, cy: number, radius: number, color: string) => {
		const orb = ctx.createRadialGradient(cx, cy, 0, cx, cy, radius);
		orb.addColorStop(0, color);
		orb.addColorStop(1, 'rgba(0,0,0,0)');
		ctx.fillStyle = orb;
		ctx.fillRect(0, 0, width, height);
	};
	drawMeshOrb(width * 0.14, height * 0.10, width * 0.58, theme.meshA);
	drawMeshOrb(width * 0.86, height * 0.22, width * 0.52, theme.meshB);
	drawMeshOrb(width * 0.50, height * 0.92, width * 0.60, theme.meshC);

	// 5. Draw Liquid Glass MemoCard container (exact .glass-surface rounded-2xl)
	const rx = outerPad;
	const ry = outerPad;
	const radius = 32;

	ctx.save();
	ctx.shadowColor = theme.cardShadow;
	ctx.shadowBlur = isDark ? 48 : 36;
	ctx.shadowOffsetX = 0;
	ctx.shadowOffsetY = isDark ? 18 : 12;
	ctx.beginPath();
	ctx.roundRect(rx, ry, cardW, cardH, radius);
	ctx.fillStyle = theme.cardFill;
	ctx.fill();
	ctx.restore();

	// Clipped inner specular top highlight + pinned amber corner fold
	ctx.save();
	ctx.beginPath();
	ctx.roundRect(rx, ry, cardW, cardH, radius);
	ctx.clip();

	// Subtle top glass specular rim (--xm-glass-specular)
	ctx.fillStyle = theme.cardSpecular;
	ctx.fillRect(rx, ry, cardW, 2);

	// Exact Official Memos 26.11 Top-Right Corner Fold Pin Indicator
	if (view.memo.pinned) {
		ctx.fillStyle = theme.pinAmber;
		ctx.beginPath();
		ctx.moveTo(rx + cardW, ry);
		ctx.lineTo(rx + cardW - 32, ry);
		ctx.lineTo(rx + cardW, ry + 32);
		ctx.closePath();
		ctx.fill();
	}
	ctx.restore();

	// Crisp outer border
	ctx.save();
	ctx.beginPath();
	ctx.roundRect(rx, ry, cardW, cardH, radius);
	ctx.lineWidth = 2;
	ctx.strokeStyle = theme.cardBorder;
	ctx.stroke();
	ctx.restore();

	// 6. Draw 1:1 MemoCard Header Row
	const contentX = rx + cardPadX;
	let cursorY = ry + cardPadTop;

	const authorName = view.creator?.nickname || view.creator?.username || 'Unknown';
	const avatarInitial = authorName.slice(0, 1).toUpperCase();
	const avatarR = 28; // 56px circle (h-7 w-7 @2x)
	const avatarCx = contentX + avatarR;
	const avatarCy = cursorY + avatarR;

	// Circular avatar badge
	ctx.save();
	ctx.beginPath();
	ctx.arc(avatarCx, avatarCy, avatarR, 0, Math.PI * 2);
	ctx.fillStyle = theme.accentSoft;
	ctx.fill();
	ctx.lineWidth = 2;
	ctx.strokeStyle = theme.accentBorder;
	ctx.stroke();
	ctx.fillStyle = theme.accent;
	ctx.font = `700 24px ${FONT_SANS}`;
	ctx.textAlign = 'center';
	ctx.textBaseline = 'middle';
	ctx.fillText(avatarInitial, avatarCx, avatarCy + 1);
	ctx.restore();

	// Author · Timestamp (single-line, exact MemoCard desktop/wide layout)
	const headerTextX = contentX + avatarR * 2 + 18;
	const headerTextY = avatarCy + 8;

	ctx.save();
	ctx.font = `600 26px ${FONT_SANS}`;
	ctx.fillStyle = theme.textPrimary;
	const maxAuthorW = contentW - 220;
	let displayAuthor = authorName;
	while (ctx.measureText(displayAuthor).width > maxAuthorW && displayAuthor.length > 1) {
		displayAuthor = displayAuthor.slice(0, -1);
	}
	if (displayAuthor !== authorName) displayAuthor += '…';
	ctx.fillText(displayAuthor, headerTextX, headerTextY);
	const authorW = ctx.measureText(displayAuthor).width;

	ctx.font = `400 24px ${FONT_SANS}`;
	ctx.fillStyle = theme.textMuted;
	const sepText = '  ·  ';
	ctx.fillText(sepText, headerTextX + authorW, headerTextY);
	const sepW = ctx.measureText(sepText).width;
	ctx.fillText(formatFullTimestamp(view.memo.createdTs), headerTextX + authorW + sepW, headerTextY);
	ctx.restore();

	// Right-aligned Visibility Icon (Globe / Users / Lock)
	drawVisibilityIcon(ctx, view.memo.visibility, contentX + contentW - 18, avatarCy, theme.textMuted);

	cursorY += headerH + headerGap;

	// 7. Draw 1:1 Rich Markdown Body (.memo-prose)
	for (let idx = 0; idx < layoutBlocks.length; idx++) {
		const item = layoutBlocks[idx];
		if (!item) continue;
		const { block } = item;

		if (block.type === 'hr') {
			ctx.save();
			ctx.fillStyle = theme.divider;
			ctx.fillRect(contentX, cursorY + 7, contentW, 2);
			ctx.restore();
		} else if (block.type === 'heading' || block.type === 'paragraph') {
			const lines = item.richLines ?? [];
			const fSize = item.fontSize ?? 30;
			const lHeight = item.lineHeight ?? 48;
			for (let li = 0; li < lines.length; li++) {
				const line = lines[li];
				if (!line) continue;
				const baseline = cursorY + li * lHeight + Math.round(fSize * 1.05);
				drawRichLine(ctx, line, contentX, baseline, fSize, theme, theme.textPrimary);
			}
		} else if (block.type === 'blockquote') {
			const lines = item.richLines ?? [];
			const fSize = item.fontSize ?? 29;
			const lHeight = item.lineHeight ?? 46;
			ctx.save();
			ctx.beginPath();
			ctx.roundRect(contentX, cursorY, contentW, item.height, [4, 20, 20, 4]);
			ctx.fillStyle = theme.surfaceSubtle;
			ctx.fill();
			ctx.fillStyle = theme.accent;
			ctx.beginPath();
			ctx.roundRect(contentX, cursorY, 6, item.height, 3);
			ctx.fill();
			ctx.restore();

			for (let li = 0; li < lines.length; li++) {
				const line = lines[li];
				if (!line) continue;
				const baseline = cursorY + 14 + li * lHeight + Math.round(fSize * 1.02);
				drawRichLine(ctx, line, contentX + 26, baseline, fSize, theme, theme.textSecondary);
			}
		} else if (block.type === 'task') {
			const lines = item.richLines ?? [];
			const fSize = item.fontSize ?? 30;
			const lHeight = item.lineHeight ?? 46;
			const boxSize = 28;
			const boxY = cursorY + 8;

			ctx.save();
			ctx.beginPath();
			ctx.roundRect(contentX + 2, boxY, boxSize, boxSize, 8);
			if (block.checked) {
				ctx.fillStyle = theme.accentBtn;
				ctx.fill();
				ctx.strokeStyle = '#ffffff';
				ctx.lineWidth = 3;
				ctx.lineCap = 'round';
				ctx.lineJoin = 'round';
				ctx.beginPath();
				ctx.moveTo(contentX + 9, boxY + 14);
				ctx.lineTo(contentX + 14, boxY + 20);
				ctx.lineTo(contentX + 23, boxY + 9);
				ctx.stroke();
			} else {
				ctx.lineWidth = 2.4;
				ctx.strokeStyle = theme.checkboxBorder;
				ctx.stroke();
			}
			ctx.restore();

			for (let li = 0; li < lines.length; li++) {
				const line = lines[li];
				if (!line) continue;
				const baseline = cursorY + li * lHeight + Math.round(fSize * 1.05);
				drawRichLine(
					ctx,
					line,
					contentX + 46,
					baseline,
					fSize,
					theme,
					block.checked ? theme.textMuted : theme.textPrimary,
					block.checked
				);
			}
		} else if (block.type === 'list') {
			const lines = item.richLines ?? [];
			const fSize = item.fontSize ?? 30;
			const lHeight = item.lineHeight ?? 46;
			const firstBaseline = cursorY + Math.round(fSize * 1.05);

			ctx.save();
			ctx.font = `600 ${fSize}px ${FONT_SANS}`;
			ctx.fillStyle = theme.textMuted;
			ctx.fillText(block.marker, contentX + 6, firstBaseline);
			ctx.restore();

			for (let li = 0; li < lines.length; li++) {
				const line = lines[li];
				if (!line) continue;
				const baseline = cursorY + li * lHeight + Math.round(fSize * 1.05);
				drawRichLine(ctx, line, contentX + 42, baseline, fSize, theme, theme.textPrimary);
			}
		} else if (block.type === 'code') {
			const codeLines = item.codeLines ?? [];
			const fSize = item.fontSize ?? 25;
			const lHeight = item.lineHeight ?? 38;
			const headerOff = block.lang ? 42 : 14;

			ctx.save();
			ctx.beginPath();
			ctx.roundRect(contentX, cursorY, contentW, item.height, 24);
			ctx.fillStyle = theme.codeBlockBg;
			ctx.fill();
			ctx.lineWidth = 2;
			ctx.strokeStyle = theme.codeBlockBorder;
			ctx.stroke();

			if (block.lang) {
				ctx.font = `500 22px ${FONT_MONO}`;
				ctx.fillStyle = theme.codeLang;
				ctx.fillText(block.lang, contentX + 28, cursorY + 30);
			}

			ctx.font = `400 ${fSize}px ${FONT_MONO}`;
			ctx.fillStyle = theme.codeText;
			for (let li = 0; li < codeLines.length; li++) {
				const cLine = codeLines[li] ?? '';
				ctx.fillText(cLine, contentX + 28, cursorY + headerOff + li * lHeight + 28);
			}
			ctx.restore();
		}

		cursorY += item.height + (idx < layoutBlocks.length - 1 ? item.gapAfter : 0);
	}

	// 8. Draw 1:1 Attachment Image Grid (if present)
	if (loadedImages.length > 0 && mediaH > 0) {
		cursorY += mediaGap;
		const gap = 14;
		if (loadedImages.length === 1) {
			const img0 = loadedImages[0];
			if (img0) drawCoverImage(ctx, img0, contentX, cursorY, contentW, mediaH, 20);
		} else if (loadedImages.length === 2) {
			const cellW = (contentW - gap) / 2;
			const img0 = loadedImages[0];
			const img1 = loadedImages[1];
			if (img0) drawCoverImage(ctx, img0, contentX, cursorY, cellW, mediaH, 18);
			if (img1) drawCoverImage(ctx, img1, contentX + cellW + gap, cursorY, cellW, mediaH, 18);
		} else {
			const cellW = (contentW - gap) / 2;
			const cellH = (mediaH - gap) / 2;
			for (let i = 0; i < loadedImages.length; i++) {
				const img = loadedImages[i];
				if (!img) continue;
				const col = i % 2;
				const row = Math.floor(i / 2);
				const cx = contentX + col * (cellW + gap);
				const cy = cursorY + row * (cellH + gap);
				drawCoverImage(ctx, img, cx, cy, cellW, cellH, 16);

				if (i === 3 && extraImagesCount > 0) {
					ctx.save();
					ctx.beginPath();
					ctx.roundRect(cx, cy, cellW, cellH, 16);
					ctx.fillStyle = 'rgba(0,0,0,0.48)';
					ctx.fill();
					ctx.fillStyle = '#ffffff';
					ctx.font = `700 38px ${FONT_SANS}`;
					ctx.textAlign = 'center';
					ctx.textBaseline = 'middle';
					ctx.fillText(`+${extraImagesCount}`, cx + cellW / 2, cy + cellH / 2);
					ctx.restore();
				}
			}
		}
		cursorY += mediaH;
	}

	// Non-image attachment badges
	if (nonImageResources.length > 0) {
		cursorY += filePillsGap;
		let pillX = contentX;
		ctx.save();
		ctx.font = `500 22px ${FONT_SANS}`;
		for (const res of nonImageResources.slice(0, 3)) {
			const label = res.filename.length > 26 ? `${res.filename.slice(0, 23)}…` : res.filename;
			const pw = ctx.measureText(label).width + 32;
			if (pillX + pw > contentX + contentW) break;
			ctx.beginPath();
			ctx.roundRect(pillX, cursorY, pw, 40, 20);
			ctx.fillStyle = theme.surfaceSubtle;
			ctx.fill();
			ctx.lineWidth = 1.5;
			ctx.strokeStyle = theme.divider;
			ctx.stroke();
			ctx.fillStyle = theme.textSecondary;
			ctx.fillText(label, pillX + 16, cursorY + 27);
			pillX += pw + 12;
		}
		ctx.restore();
		cursorY += filePillsH;
	}

	// 9. Draw Location & Linked Relations Metadata (exact MemoCard lines 223-239)
	if (metaH > 0) {
		cursorY += metaGap;
		ctx.save();
		ctx.font = `500 23px ${FONT_SANS}`;
		ctx.fillStyle = theme.textMuted;

		if (view.memo.payload.location) {
			const loc = view.memo.payload.location;
			const locLabel =
				loc.placeholder?.trim() || `${loc.latitude.toFixed(4)}°, ${loc.longitude.toFixed(4)}°`;
			drawMapPinIcon(ctx, contentX, cursorY + 22, theme.textMuted);
			ctx.fillText(locLabel, contentX + 30, cursorY + 22);
			cursorY += 36;
		}
		if (hasRelations) {
			drawLinkIcon(ctx, contentX, cursorY + 22, theme.textMuted);
			ctx.fillText(`${referenceRelations.length} linked`, contentX + 30, cursorY + 22);
			cursorY += 36;
		}
		ctx.restore();
	}

	// 10. Draw 1:1 Clean Footer (Divider + Reactions/Comments Left + Minimal Xmemos Emblem Right)
	const footerY = ry + cardH - cardPadBottom - footerH;
	ctx.save();
	ctx.fillStyle = theme.divider;
	ctx.fillRect(contentX, footerY - 12, contentW, 2);

	let chipX = contentX;
	const chipY = footerY + 6;
	const chipH = 38;
	ctx.font = `500 22px ${FONT_SANS}`;

	for (const item of groupedReactions.slice(0, 6)) {
		const text = `${item.emoji} ${item.count}`;
		const cw = ctx.measureText(text).width + 28;
		if (chipX + cw > contentX + contentW - 180) break;
		ctx.beginPath();
		ctx.roundRect(chipX, chipY, cw, chipH, chipH / 2);
		ctx.fillStyle = theme.chipBg;
		ctx.fill();
		ctx.lineWidth = 1.5;
		ctx.strokeStyle = theme.chipBorder;
		ctx.stroke();
		ctx.fillStyle = theme.textPrimary;
		ctx.fillText(text, chipX + 14, chipY + 26);
		chipX += cw + 10;
	}

	if (view.commentCount > 0 && chipX < contentX + contentW - 190) {
		drawCommentIcon(ctx, chipX + 4, chipY + 27, theme.textMuted);
		ctx.fillStyle = theme.textMuted;
		ctx.fillText(String(view.commentCount), chipX + 30, chipY + 26);
	}

	// Minimal right-aligned Xmemos brand badge
	ctx.font = `600 24px ${FONT_SANS}`;
	const brandText = 'Xmemos';
	const brandW = ctx.measureText(brandText).width;
	const iconSize = 32;
	const brandRightX = contentX + contentW;
	const textX = brandRightX - brandW;
	const iconX = textX - iconSize - 10;
	const iconY = footerY + 9;

	drawMiniAppIcon(ctx, iconX, iconY, iconSize);
	ctx.fillStyle = theme.textSecondary;
	ctx.fillText(brandText, textX, footerY + 33);
	ctx.restore();

	return new Promise<Blob>((resolve, reject) => {
		canvas.toBlob((blob) => {
			if (blob) resolve(blob);
			else reject(new Error('Failed to encode PNG share card'));
		}, 'image/png');
	});
}
