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
	import { onDestroy } from 'svelte';
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

	let routeIndex = $derived.by(() => {
		const idx = NAV_ITEMS.findIndex((item) => {
			if (item.path === '/') return currentPath === '/';
			return currentPath === item.path || currentPath.startsWith(`${item.path}/`);
		});
		return idx >= 0 ? idx : 0;
	});

	let trackEl = $state<HTMLDivElement | null>(null);
	let pillEl = $state<HTMLDivElement | null>(null);
	let dragging = $state(false);
	let pointerId: number | null = null;
	let startX = 0;
	let didScrub = false;
	/** Locks the pill on the target tab while SvelteKit navigation settles (zero snap-back/bounce) */
	let pendingNavIndex = $state<number | null>(null);
	/** Only updates when crossing into a new integer tab slot (zero per-pixel Svelte re-renders!) */
	let hoverIndex = $state<number | null>(null);

	let trackRectCache: DOMRect | null = null;
	let pendingClientX: number | null = null;
	let scrubRafId = 0;
	let lastNavigatedIndex = -1;

	$effect(() => {
		if (pendingNavIndex !== null && routeIndex === pendingNavIndex) {
			pendingNavIndex = null;
		}
	});

	let settledIndex = $derived(pendingNavIndex !== null ? pendingNavIndex : routeIndex);
	let highlightedIndex = $derived(hoverIndex !== null ? hoverIndex : settledIndex);

	// Sync pill GPU transform whenever settledIndex changes while not actively finger-dragging
	$effect(() => {
		const idx = settledIndex;
		if (!dragging && pillEl) {
			pillEl.style.transform = `translate3d(${idx * 100}%, 0, 0)`;
		}
	});

	function navigateToIndex(index: number, fromScrub = false): void {
		const clamped = Math.max(0, Math.min(TAB_COUNT - 1, index));
		const target = NAV_ITEMS[clamped];
		if (!target) return;
		lastNavigatedIndex = clamped;
		if (currentPath === target.path) {
			pendingNavIndex = null;
			if (!fromScrub && typeof window !== 'undefined') {
				window.scrollTo({ top: 0, behavior: 'smooth' });
			}
			return;
		}
		pendingNavIndex = clamped;
		void goto(target.path, { reset: false });
	}

	function pointerXToFloatIndex(clientX: number): number {
		const rect = trackRectCache ?? trackEl?.getBoundingClientRect();
		if (!rect || rect.width <= 0) return settledIndex;
		const innerWidth = Math.max(1, rect.width - 8);
		const slotWidth = innerWidth / TAB_COUNT;
		const raw = (clientX - (rect.left + 4) - slotWidth / 2) / slotWidth;
		return Math.max(0, Math.min(TAB_COUNT - 1, raw));
	}

	function flushScrubFrame(): void {
		scrubRafId = 0;
		if (!dragging || pendingClientX === null) return;
		const nextFloat = pointerXToFloatIndex(pendingClientX);

		// Direct GPU compositor transform update — zero Svelte re-renders during finger glide!
		if (pillEl) {
			pillEl.style.transform = `translate3d(${nextFloat * 100}%, 0, 0)`;
		}

		const candidate = Math.max(0, Math.min(TAB_COUNT - 1, Math.round(nextFloat)));
		if (candidate !== hoverIndex) {
			hoverIndex = candidate;
		}
	}

	function handlePointerDown(event: PointerEvent): void {
		if (event.button !== 0 && event.pointerType === 'mouse') return;
		pointerId = event.pointerId;
		startX = event.clientX;
		didScrub = false;
		lastNavigatedIndex = settledIndex;
		trackRectCache = trackEl?.getBoundingClientRect() ?? null;
	}

	function handlePointerMove(event: PointerEvent): void {
		if (pointerId !== event.pointerId || !trackEl) return;
		const delta = Math.abs(event.clientX - startX);
		if (!dragging && delta > 5) {
			dragging = true;
			didScrub = true;
			hoverIndex = settledIndex;
			try {
				trackEl.setPointerCapture(event.pointerId);
			} catch {
				// Ignore capture errors on synthetic events
			}
		}
		if (dragging) {
			pendingClientX = event.clientX;
			if (scrubRafId === 0) {
				scrubRafId = requestAnimationFrame(flushScrubFrame);
			}
		}
	}

	function finishScrub(event: PointerEvent, commit: boolean): void {
		if (pointerId !== event.pointerId) return;
		if (scrubRafId !== 0) {
			cancelAnimationFrame(scrubRafId);
			scrubRafId = 0;
		}
		const finalIndex = dragging
			? Math.max(0, Math.min(TAB_COUNT - 1, Math.round(pointerXToFloatIndex(event.clientX))))
			: null;
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
		pendingClientX = null;
		trackRectCache = null;
		dragging = false;
		hoverIndex = null;

		if (commit && finalIndex !== null) {
			pendingNavIndex = finalIndex;
			if (pillEl) {
				pillEl.style.transform = `translate3d(${finalIndex * 100}%, 0, 0)`;
			}
			if (finalIndex !== lastNavigatedIndex) {
				navigateToIndex(finalIndex, true);
			}
		} else if (pillEl) {
			pillEl.style.transform = `translate3d(${settledIndex * 100}%, 0, 0)`;
		}
	}

	function handleTabClick(index: number): void {
		if (didScrub) {
			didScrub = false;
			return;
		}
		navigateToIndex(index, false);
	}

	onDestroy(() => {
		if (scrubRafId !== 0) {
			cancelAnimationFrame(scrubRafId);
		}
	});
</script>

{#if visible}
	<!-- Authentic Konsta UI v5 iOS 26 Liquid Glass Tabbar with 60/120fps direct-GPU scrubbing -->
	<nav
		aria-label={i18n.t('nav.primaryNav')}
		class="xm-ios-tabbar-shell md:hidden"
		class:xm-ios-tabbar-hidden={keyboardState.open}
	>
		<div
			bind:this={trackEl}
			role="toolbar"
			tabindex="-1"
			aria-label={i18n.t('nav.primaryNav')}
			class="xm-ios-tabbar-track"
			onpointerdown={handlePointerDown}
			onpointermove={handlePointerMove}
			onpointerup={(e) => finishScrub(e, true)}
			onpointercancel={(e) => finishScrub(e, false)}
		>
			<!-- Direct-GPU transform: translate3d neutral Liquid Glass lens (zero blue fill, zero jank) -->
			<div
				bind:this={pillEl}
				class="xm-ios-tabbar-pill"
				class:is-dragging={dragging}
				style={`width: calc((100% - 8px) / ${TAB_COUNT});`}
				aria-hidden="true"
			></div>

			{#each NAV_ITEMS as item, idx (item.path)}
				{@const IconComp = item.icon}
				{@const active = highlightedIndex === idx}
				<button
					type="button"
					class="xm-ios-tabbar-item"
					class:is-active={active}
					aria-current={settledIndex === idx ? 'page' : undefined}
					onclick={() => handleTabClick(idx)}
				>
					<IconComp
						class="xm-ios-tabbar-icon h-[20px] w-[20px]"
						strokeWidth={active ? 2.2 : 1.8}
						aria-hidden="true"
					/>
					<span class="xm-ios-tabbar-label">{i18n.t(item.labelKey)}</span>
				</button>
			{/each}
		</div>
	</nav>
{/if}
