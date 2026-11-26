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
	import { Toggle } from 'konsta/svelte';
	import { Columns2, Columns3, Infinity as InfinityIcon, List as ListIcon } from 'lucide-svelte';

	import { i18n } from '#lib/i18n/index.svelte.ts';
	import {
		feedDisplayState,
		type FeedColumnLayout,
		type FeedOrderBy,
		type FeedOrderDirection
	} from '#lib/state/feed-display.svelte.ts';

	const LAYOUT_OPTIONS: {
		id: FeedColumnLayout;
		labelKey: 'feed.layoutList' | 'feed.layout2' | 'feed.layout3' | 'feed.layoutAuto';
		icon: typeof ListIcon;
	}[] = [
		{ id: 'list', labelKey: 'feed.layoutList', icon: ListIcon },
		{ id: '2', labelKey: 'feed.layout2', icon: Columns2 },
		{ id: '3', labelKey: 'feed.layout3', icon: Columns3 },
		{ id: 'auto', labelKey: 'feed.layoutAuto', icon: InfinityIcon }
	];

	let anchor = $derived(feedDisplayState.popoverAnchor);

	let popoverStyle = $derived.by(() => {
		if (!anchor || typeof window === 'undefined') return '';
		const { rect, placement } = anchor;
		const popWidth = Math.min(window.innerWidth - 24, 292);
		const popHeight = 256;
		const margin = 12;

		let left: number;
		let top: number;

		if (placement === 'right') {
			left = rect.right + 10;
			if (left + popWidth > window.innerWidth - margin) {
				left = Math.max(margin, window.innerWidth - popWidth - margin);
			}
			top = rect.top - 8;
			if (top + popHeight > window.innerHeight - margin) {
				top = Math.max(margin, window.innerHeight - popHeight - margin);
			}
			if (top < margin) top = margin;
		} else {
			left = rect.right - popWidth;
			if (left < margin) left = margin;
			if (left + popWidth > window.innerWidth - margin) {
				left = window.innerWidth - popWidth - margin;
			}
			const spaceBelow = window.innerHeight - rect.bottom - margin;
			top =
				spaceBelow >= popHeight
					? rect.bottom + 8
					: Math.max(margin, rect.top - popHeight - 8);
		}

		return `left:${Math.round(left)}px;top:${Math.round(top)}px;width:${Math.round(popWidth)}px;`;
	});

	function handleWindowKeydown(event: KeyboardEvent): void {
		if (anchor && event.key === 'Escape') {
			event.preventDefault();
			feedDisplayState.closePopover();
		}
	}
</script>

<svelte:window
	onkeydown={handleWindowKeydown}
	onresize={() => anchor && feedDisplayState.closePopover()}
/>

{#if anchor}
	<div
		class="fixed inset-0 z-[79]"
		aria-hidden="true"
		onclick={() => feedDisplayState.closePopover()}
	></div>

	<div
		role="dialog"
		aria-label={i18n.t('feed.displayOptions')}
		style={popoverStyle}
		class="glass-elevated fixed z-[80] space-y-3.5 rounded-2xl border border-[var(--xm-divider)] p-3.5 shadow-2xl"
	>
		<!-- Layout Columns: Mobile supports clean 1-Column List & 2-Column Grid; Tablet/Desktop adds 3 & Auto -->
		<div class="space-y-1.5">
			<div
				class="grid grid-cols-2 gap-1 rounded-xl bg-[var(--xm-surface-subtle)] p-1 md:grid-cols-4"
				role="radiogroup"
				aria-label={i18n.t('feed.layoutLabel')}
			>
				{#each LAYOUT_OPTIONS as opt (opt.id)}
					{@const IconComp = opt.icon}
					{@const desktopOnly = opt.id === '3' || opt.id === 'auto'}
					{@const active =
						feedDisplayState.layout === opt.id ||
						(opt.id === '2' && (feedDisplayState.layout === '3' || feedDisplayState.layout === 'auto') && typeof window !== 'undefined' && window.innerWidth < 768)}
					<button
						type="button"
						role="radio"
						aria-checked={active}
						class="{desktopOnly ? 'hidden md:inline-flex' : 'inline-flex'} h-8 items-center justify-center gap-1 rounded-lg px-1.5 text-[11px] font-semibold transition-all cursor-pointer {active
							? 'bg-white text-[var(--xm-accent)] shadow-sm dark:bg-[#2c2c2e] dark:text-white'
							: 'xm-text-secondary hover:text-[var(--xm-text-primary)]'}"
						onclick={() => feedDisplayState.setLayout(opt.id)}
					>
						<IconComp class="h-3.5 w-3.5 shrink-0" aria-hidden="true" />
						<span>{i18n.t(opt.labelKey)}</span>
					</button>
				{/each}
			</div>
		</div>

		<!-- Order by & Direction (available on all screen sizes) -->
		<div class="space-y-2.5">
			<div class="flex items-center justify-between gap-2">
				<label for="xm-root-order-by" class="text-xs font-medium xm-text-secondary">
					{i18n.t('feed.orderBy')}
				</label>
				<select
					id="xm-root-order-by"
					value={feedDisplayState.orderBy}
					onchange={(e) =>
						feedDisplayState.setOrderBy((e.currentTarget as HTMLSelectElement).value as FeedOrderBy)}
					class="xm-input !h-8 !w-[152px] !rounded-xl !px-2.5 !py-0 !text-xs font-medium"
				>
					<option value="created_ts">{i18n.t('feed.orderCreatedAt')}</option>
					<option value="updated_ts">{i18n.t('feed.orderUpdatedAt')}</option>
				</select>
			</div>

			<div class="flex items-center justify-between gap-2">
				<label for="xm-root-direction" class="text-xs font-medium xm-text-secondary">
					{i18n.t('feed.direction')}
				</label>
				<select
					id="xm-root-direction"
					value={feedDisplayState.orderDirection}
					onchange={(e) =>
						feedDisplayState.setOrderDirection(
							(e.currentTarget as HTMLSelectElement).value as FeedOrderDirection
						)}
					class="xm-input !h-8 !w-[152px] !rounded-xl !px-2.5 !py-0 !text-xs font-medium"
				>
					<option value="desc">{i18n.t('feed.directionNewest')}</option>
					<option value="asc">{i18n.t('feed.directionOldest')}</option>
				</select>
			</div>
		</div>

		<!-- Full Width Toggle: Tablet & Desktop only -->
		<div
			class="hidden items-center justify-between gap-3 border-t border-[var(--xm-divider)] pt-3 md:flex"
		>
			<div class="min-w-0">
				<p class="text-xs font-semibold xm-text-primary">{i18n.t('feed.fullWidth')}</p>
				<p class="mt-0.5 text-[11px] leading-snug xm-text-muted">
					{i18n.t('feed.fullWidthDesc')}
				</p>
			</div>
			<Toggle
				checked={feedDisplayState.fullWidth}
				onchange={() => feedDisplayState.setFullWidth(!feedDisplayState.fullWidth)}
			/>
		</div>
	</div>
{/if}
