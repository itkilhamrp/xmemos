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
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { CalendarDays, Compass, FolderOpen, Home, Settings } from 'lucide-svelte';

	import { i18n, type TranslationKey } from '#lib/i18n/index.svelte.ts';
	import { keyboardState } from '#lib/utils/keyboard.svelte.ts';

	const NAV_ITEMS: readonly { path: string; labelKey: TranslationKey; icon: typeof Home }[] = [
		{ path: '/', labelKey: 'nav.home', icon: Home },
		{ path: '/explore', labelKey: 'nav.explore', icon: Compass },
		{ path: '/review', labelKey: 'nav.review', icon: CalendarDays },
		{ path: '/resources', labelKey: 'nav.media', icon: FolderOpen },
		{ path: '/settings', labelKey: 'nav.settings', icon: Settings }
	];

	const TAB_COUNT = NAV_ITEMS.length;

	let currentPath = $derived(page.url.pathname);
	let visible = $derived(currentPath !== '/auth');

	let activeIndex = $derived.by(() => {
		const idx = NAV_ITEMS.findIndex((item) => {
			if (item.path === '/') return currentPath === '/';
			return currentPath === item.path || currentPath.startsWith(`${item.path}/`);
		});
		return idx >= 0 ? idx : 0;
	});

	let trackEl = $state<HTMLDivElement | null>(null);
	let dragging = $state(false);
	let pointerId = $state<number | null>(null);
	let startX = 0;
	let didScrub = false;
	/** Continuous floating tab index [0 .. TAB_COUNT - 1] while finger-scrubbing */
	let dragIndex = $state<number | null>(null);

	let pillPosition = $derived(dragIndex !== null ? dragIndex : activeIndex);
	let highlightedIndex = $derived(
		dragIndex !== null ? Math.max(0, Math.min(TAB_COUNT - 1, Math.round(dragIndex))) : activeIndex
	);

	function navigateToIndex(index: number): void {
		const clamped = Math.max(0, Math.min(TAB_COUNT - 1, index));
		const target = NAV_ITEMS[clamped];
		if (!target) return;
		if (currentPath === target.path) {
			if (typeof window !== 'undefined') {
				window.scrollTo({ top: 0, behavior: 'smooth' });
			}
			return;
		}
		void goto(target.path);
	}

	function pointerXToFloatIndex(clientX: number): number {
		if (!trackEl) return activeIndex;
		const rect = trackEl.getBoundingClientRect();
		if (rect.width <= 0) return activeIndex;
		const slotWidth = rect.width / TAB_COUNT;
		const raw = (clientX - rect.left - slotWidth / 2) / slotWidth;
		return Math.max(0, Math.min(TAB_COUNT - 1, raw));
	}

	function handlePointerDown(event: PointerEvent): void {
		if (event.button !== 0 && event.pointerType === 'mouse') return;
		pointerId = event.pointerId;
		startX = event.clientX;
		didScrub = false;
	}

	function handlePointerMove(event: PointerEvent): void {
		if (pointerId !== event.pointerId || !trackEl) return;
		const delta = Math.abs(event.clientX - startX);
		if (!dragging && delta > 7) {
			dragging = true;
			didScrub = true;
			try {
				trackEl.setPointerCapture(event.pointerId);
			} catch {
				// Ignore capture errors on synthetic events
			}
		}
		if (dragging) {
			dragIndex = pointerXToFloatIndex(event.clientX);
		}
	}

	function finishScrub(event: PointerEvent, commit: boolean): void {
		if (pointerId !== event.pointerId) return;
		const finalIndex = dragging ? Math.round(pointerXToFloatIndex(event.clientX)) : null;
		if (trackEl) {
			try {
				if (trackEl.hasPointerCapture(event.pointerId)) {
					trackEl.releasePointerCapture(event.pointerId);
				}
			} catch {
				// Ignore
			}
		}
		pointerId = null;
		dragging = false;
		dragIndex = null;
		if (commit && finalIndex !== null) {
			navigateToIndex(finalIndex);
		}
	}

	function handleTabClick(index: number): void {
		if (didScrub) {
			didScrub = false;
			return;
		}
		navigateToIndex(index);
	}
</script>

{#if visible}
	<!-- Silky 60/120fps GPU-composited iOS 26 floating pill Tabbar with 1:1 finger scrubbing -->
	<nav
		aria-label={i18n.t('nav.primaryNav')}
		class="xm-ios-tabbar-shell md:hidden"
		class:xm-ios-tabbar-hidden={keyboardState.open}
	>
		<div
			bind:this={trackEl}
			class="xm-ios-tabbar-track"
			onpointerdown={handlePointerDown}
			onpointermove={handlePointerMove}
			onpointerup={(e) => finishScrub(e, true)}
			onpointercancel={(e) => finishScrub(e, false)}
		>
			<!-- Pure transform: translate3d sliding glass indicator pill (zero layout reflow) -->
			<div
				class="xm-ios-tabbar-pill"
				class:is-dragging={dragging}
				style={`width: calc((100% - 8px) / ${TAB_COUNT}); transform: translate3d(calc(${pillPosition} * 100%), 0, 0)${dragging ? ' scale(1.045)' : ''};`}
				aria-hidden="true"
			></div>

			{#each NAV_ITEMS as item, idx (item.path)}
				{@const IconComp = item.icon}
				{@const active = highlightedIndex === idx}
				<button
					type="button"
					class="xm-ios-tabbar-item"
					class:is-active={active}
					aria-current={activeIndex === idx ? 'page' : undefined}
					onclick={() => handleTabClick(idx)}
				>
					<IconComp class="xm-ios-tabbar-icon h-[20px] w-[20px]" strokeWidth={active ? 2.25 : 1.85} aria-hidden="true" />
					<span class="xm-ios-tabbar-label">{i18n.t(item.labelKey)}</span>
				</button>
			{/each}
		</div>
	</nav>
{/if}
