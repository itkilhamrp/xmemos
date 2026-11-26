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
	import { onDestroy, tick, untrack } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { shortcutApi } from '#lib/api/shortcut.ts';
	import type * as LeafletNS from 'leaflet';
	import { ExternalLink, Minus, Plus } from 'lucide-svelte';

	import { i18n } from '#lib/i18n/index.svelte.ts';
	import { locationPreviewState } from '#lib/state/location-preview.svelte.ts';
	import { themeState } from '#lib/state/theme.svelte.ts';
	import { formatCoordinates } from '#lib/utils/geocode.ts';
	import { attachMapPinchZoomGuard, createOfflineOsmTileLayer, loadLeaflet } from '#lib/utils/osm-tiles.ts';

	let popoverEl = $state<HTMLDivElement | null>(null);
	let mapEl = $state<HTMLDivElement | null>(null);
	let map: LeafletNS.Map | null = null;
	let mapSeq = 0;
	let invalidateTimer: ReturnType<typeof setTimeout> | null = null;
	let detachPinchGuard: (() => void) | null = null;

	let active = $derived(locationPreviewState.active);

	let popoverStyle = $derived.by(() => {
		if (!active || typeof window === 'undefined') return '';
		const rect = active.anchorRect;
		const popWidth = Math.min(window.innerWidth - 24, 312);
		const popHeight = Math.min(258, Math.max(180, window.innerHeight - 24));
		const margin = 12;

		let left = rect.left;
		if (left + popWidth > window.innerWidth - margin) {
			left = window.innerWidth - popWidth - margin;
		}
		if (left < margin) left = margin;

		const spaceBelow = window.innerHeight - rect.bottom - margin;
		const top =
			spaceBelow >= popHeight
				? rect.bottom + 8
				: Math.max(margin, rect.top - popHeight - 8);

		return `left:${Math.round(left)}px;top:${Math.round(top)}px;width:${Math.round(popWidth)}px;`;
	});

	function destroyMap(): void {
		mapSeq += 1;
		if (invalidateTimer) {
			clearTimeout(invalidateTimer);
			invalidateTimer = null;
		}
		detachPinchGuard?.();
		detachPinchGuard = null;
		if (map) {
			map.remove();
			map = null;
		}
	}

	async function mountMiniMap(lat: number, lng: number): Promise<void> {
		destroyMap();
		const seq = ++mapSeq;
		await tick();
		if (!mapEl || seq !== mapSeq) return;

		const leaflet = await loadLeaflet();
		if (!mapEl || seq !== mapSeq) return;

		const mapInst = leaflet.map(mapEl, {
			center: [lat, lng],
			zoom: 12,
			minZoom: 2,
			maxZoom: 18,
			zoomSnap: 0.25,
			zoomDelta: 1,
			wheelPxPerZoomLevel: 90,
			zoomControl: false,
			attributionControl: true
		});

		mapInst.attributionControl?.setPrefix(false);

		const tileLayer = createOfflineOsmTileLayer(
			leaflet,
			() => themeState.isDark,
			'&copy; OpenStreetMap'
		);
		tileLayer.addTo(mapInst);

		const pinIcon = leaflet.divIcon({
			html: `<div class="xm-map-preview-dot"></div>`,
			className: 'xm-map-icon-wrapper',
			iconSize: [18, 18],
			iconAnchor: [9, 9]
		});
		leaflet.marker([lat, lng], { icon: pinIcon }).addTo(mapInst);

		detachPinchGuard = attachMapPinchZoomGuard(mapEl, mapInst);
		map = mapInst;
		invalidateTimer = setTimeout(() => {
			if (map === mapInst) mapInst.invalidateSize({ animate: false });
		}, 90);
	}

	$effect(() => {
		const cur = active;
		untrack(() => {
			if (!cur) {
				destroyMap();
				return;
			}
			void mountMiniMap(cur.location.latitude, cur.location.longitude);
		});
	});

	function handleWindowKeydown(event: KeyboardEvent): void {
		if (active && event.key === 'Escape') {
			event.preventDefault();
			locationPreviewState.close();
		}
	}

	function openFullMap(): void {
		if (!active) return;
		const uid = active.memoUid;
		locationPreviewState.close();
		const target = `/map?memo=${encodeURIComponent(uid)}`;
		if (page.url.pathname.startsWith('/sticky/')) {
			void shortcutApi.focusMainRoute(target);
		} else {
			void goto(target);
		}
	}

	onDestroy(() => {
		destroyMap();
	});
</script>

<svelte:window
	onkeydown={handleWindowKeydown}
	onresize={() => active && locationPreviewState.close()}
/>

{#if active}
	<!-- Transparent click-outside backdrop at root level -->
	<div
		class="fixed inset-0 z-[79]"
		aria-hidden="true"
		onclick={() => locationPreviewState.close()}
	></div>

	<div
		bind:this={popoverEl}
		role="dialog"
		aria-label={active.displayLabel}
		style={popoverStyle}
		class="glass-elevated fixed z-[80] overflow-hidden rounded-2xl p-2.5 shadow-2xl"
	>
		<!-- Interactive Mini-Map Canvas (Official Memos 2026 style) -->
		<div
			class="relative isolate z-0 h-[176px] w-full overflow-hidden rounded-xl border border-[var(--xm-divider)] bg-[#e8eef5] dark:bg-[#181a1f]"
		>
			<div
				bind:this={mapEl}
				class="xm-osm-map absolute inset-0 h-full w-full {themeState.isDark ? 'xm-osm-dark' : ''}"
			></div>

			<!-- Floating Top-Right Controls: Open on Full Map + Zoom In + Zoom Out -->
			<div class="absolute top-2 right-2 z-[400] flex flex-col gap-1.5">
				<button
					type="button"
					title={i18n.t('map.openFullMap')}
					aria-label={i18n.t('map.openFullMap')}
					class="flex h-7 w-7 items-center justify-center rounded-lg border border-black/15 bg-black/75 text-white shadow-md backdrop-blur-sm transition hover:bg-black/90 cursor-pointer"
					onclick={openFullMap}
				>
					<ExternalLink class="h-3.5 w-3.5" aria-hidden="true" />
				</button>
				<div
					class="flex flex-col overflow-hidden rounded-lg border border-black/15 bg-black/75 text-white shadow-md backdrop-blur-sm"
				>
					<button
						type="button"
						title={i18n.t('map.zoomIn')}
						aria-label={i18n.t('map.zoomIn')}
						class="flex h-7 w-7 items-center justify-center transition hover:bg-white/15 cursor-pointer"
						onclick={() => map?.zoomIn()}
					>
						<Plus class="h-3.5 w-3.5" aria-hidden="true" />
					</button>
					<div class="h-px bg-white/20" aria-hidden="true"></div>
					<button
						type="button"
						title={i18n.t('map.zoomOut')}
						aria-label={i18n.t('map.zoomOut')}
						class="flex h-7 w-7 items-center justify-center transition hover:bg-white/15 cursor-pointer"
						onclick={() => map?.zoomOut()}
					>
						<Minus class="h-3.5 w-3.5" aria-hidden="true" />
					</button>
				</div>
			</div>
		</div>

		<!-- Clean Official Memos Footer: Place Name + Exact Degree Coordinates -->
		<div class="mt-2.5 px-0.5">
			<div class="truncate text-[13px] font-semibold leading-snug xm-text-primary">
				{active.displayLabel}
			</div>
			<div class="truncate text-[11px] xm-text-muted tabular-nums">
				{formatCoordinates(active.location.latitude, active.location.longitude)}
			</div>
		</div>
	</div>
{/if}
