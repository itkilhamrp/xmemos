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

	let routeIndex = $derived.by(() => {
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
	/** Locks the pill on the target tab while SvelteKit navigation settles (zero snap-back/bounce) */
	let pendingNavIndex = $state<number | null>(null);
	let lastRealtimeIndex = -1;
	let trackRectCache: DOMRect | null = null;
	let pendingClientX: number | null = null;
	let scrubRafId = 0;

	$effect(() => {
		if (pendingNavIndex !== null && routeIndex === pendingNavIndex) {
			pendingNavIndex = null;
		}
	});

	let settledIndex = $derived(pendingNavIndex !== null ? pendingNavIndex : routeIndex);
	let pillPosition = $derived(dragIndex !== null ? dragIndex : settledIndex);
	let highlightedIndex = $derived(
		dragIndex !== null
			? Math.max(0, Math.min(TAB_COUNT - 1, Math.round(dragIndex)))
			: settledIndex
	);

	function navigateToIndex(index: number, fromScrub = false): void {
		const clamped = Math.max(0, Math.min(TAB_COUNT - 1, index));
		const target = NAV_ITEMS[clamped];
		if (!target) return;
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
		dragIndex = nextFloat;

		// Hysteresis (0.36) prevents rapid oscillation when finger hovers right on a tab boundary
		const candidate = Math.max(0, Math.min(TAB_COUNT - 1, Math.round(nextFloat)));
		if (
			candidate !== lastRealtimeIndex &&
			Math.abs(nextFloat - candidate) <= 0.36
		) {
			lastRealtimeIndex = candidate;
			navigateToIndex(candidate, true);
		}
	}

	function handlePointerDown(event: PointerEvent): void {
		if (event.button !== 0 && event.pointerType === 'mouse') return;
		pointerId = event.pointerId;
		startX = event.clientX;
		didScrub = false;
		lastRealtimeIndex = settledIndex;
		trackRectCache = trackEl?.getBoundingClientRect() ?? null;
	}

	function handlePointerMove(event: PointerEvent): void {
		if (pointerId !== event.pointerId || !trackEl) return;
		const delta = Math.abs(event.clientX - startX);
		if (!dragging && delta > 6) {
			dragging = true;
			didScrub = true;
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
		if (commit && finalIndex !== null) {
			pendingNavIndex = finalIndex;
			dragIndex = null;
			if (finalIndex !== lastRealtimeIndex) {
				navigateToIndex(finalIndex, true);
			}
		} else {
			dragIndex = null;
		}
	}

	function handleTabClick(index: number): void {
		if (didScrub) {
			didScrub = false;
			return;
		}
		navigateToIndex(index, false);
	}
</script>

{#if visible}
	<!-- Authentic Konsta UI v5 iOS 26 Liquid Glass Tabbar with 60/120fps realtime scrubbing -->
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
			<!-- Pure transform: translate3d neutral Liquid Glass sliding lens (zero blue fill, zero bounce) -->
			<div
				class="xm-ios-tabbar-pill"
				class:is-dragging={dragging}
				style={`width: calc((100% - 8px) / ${TAB_COUNT}); transform: translate3d(calc(${pillPosition} * 100%), 0, 0);`}
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
