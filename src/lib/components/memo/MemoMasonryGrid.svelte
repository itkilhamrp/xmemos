<!--
  Copyright (C) 2026 Xmemos Contributors

  This file is part of Xmemos.

  Xmemos is free software: you can redistribute it and/or modify
  it under the terms of the GNU Affero General Public License as published by
  the Free Software Foundation, either version 3 of the License, or
  (at your option) any later version.

  Xmemos is distributed in the hope that it will be useful,
  but WITHOUT ANY WARRANTY; without even the implied warranty of
  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
  GNU Affero General Public License for more details.

  You should have received a copy of the GNU Affero General Public License
  along with Xmemos. If not, see https://www.gnu.org/licenses/.

  SPDX-License-Identifier: AGPL-3.0-or-later
-->

<script lang="ts">
	import type { Snippet } from 'svelte';
	import type { MemoView } from '#lib/types/memo.ts';
	import type { FeedColumnLayout } from '#lib/state/feed-display.svelte.ts';

	interface Props {
		items: MemoView[];
		layout: FeedColumnLayout;
		card: Snippet<[MemoView]>;
	}

	let { items, layout, card }: Props = $props();

	let gridEl = $state<HTMLDivElement | null>(null);
	let containerWidth = $state(0);
	let viewportWidth = $state(typeof window !== 'undefined' ? window.innerWidth : 1024);

	$effect(() => {
		const el = gridEl;
		if (!el || typeof ResizeObserver === 'undefined') return;
		containerWidth = el.clientWidth;
		const ro = new ResizeObserver((entries) => {
			for (const entry of entries) {
				containerWidth = entry.contentRect.width;
			}
		});
		ro.observe(el);
		return () => ro.disconnect();
	});

	/**
	 * Smart container-width-aware column calculation:
	 * - Mobile (< 768px): supports 1-column List or crisp 2-column Masonry (when container >= 300px)
	 * - Desktop/Tablet: guarantees comfortable minimum card width (~275px+) so cards never crush
	 */
	let isMobileCompactTwoCol = $derived(layout !== 'list' && viewportWidth < 768);

	let columnCount = $derived.by((): number => {
		if (layout === 'list') return 1;
		const w = containerWidth > 0 ? containerWidth : viewportWidth;
		if (viewportWidth < 768) {
			return w >= 300 ? 2 : 1;
		}
		if (layout === '2') return w >= 540 ? 2 : 1;
		if (layout === '3') return w >= 840 ? 3 : w >= 540 ? 2 : 1;
		// 'auto': smart adaptive columns keeping >= 285px per card
		return Math.max(1, Math.min(4, Math.floor((w + 14) / 295)));
	});

	let columns = $derived.by((): MemoView[][] => {
		if (columnCount <= 1) return [items];
		const cols: MemoView[][] = Array.from({ length: columnCount }, () => []);
		for (let i = 0; i < items.length; i++) {
			cols[i % columnCount]!.push(items[i]!);
		}
		return cols;
	});
</script>

<svelte:window bind:innerWidth={viewportWidth} />

<div bind:this={gridEl} class="w-full min-w-0">
	{#if columnCount <= 1}
		<div class="space-y-3">
			{#each items as view (view.memo.id)}
				<div class="xm-memo-card">
					{@render card(view)}
				</div>
			{/each}
		</div>
	{:else}
		<div
			class="flex items-start {isMobileCompactTwoCol ? 'xm-masonry-compact gap-2.5' : 'gap-3.5'}"
		>
			{#each columns as col, colIdx (colIdx)}
				<div class="flex min-w-0 flex-1 flex-col {isMobileCompactTwoCol ? 'gap-2.5' : 'gap-3.5'}">
					{#each col as view (view.memo.id)}
						<div class="xm-memo-card min-w-0">
							{@render card(view)}
						</div>
					{/each}
				</div>
			{/each}
		</div>
	{/if}
</div>
