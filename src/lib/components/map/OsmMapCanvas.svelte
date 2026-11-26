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
	import { onDestroy, onMount, untrack } from 'svelte';
	import { LocateFixed, Minus, Plus } from 'lucide-svelte';
	import type * as LeafletNS from 'leaflet';

	import { i18n } from '#lib/i18n/index.svelte.ts';
	import { themeState } from '#lib/state/theme.svelte.ts';
	import type { MemoLocation, MemoView } from '#lib/types/memo.ts';
	import { attachMapPinchZoomGuard, createOfflineOsmTileLayer, loadLeaflet } from '#lib/utils/osm-tiles.ts';

	export interface MapCluster {
		id: string;
		latitude: number;
		longitude: number;
		memos: MemoView[];
		label: string;
	}

	interface Props {
		memos?: MemoView[];
		selectedClusterId?: string | null;
		droppedPin?: MemoLocation | null;
		focusMemoUid?: string | null;
		pickerMode?: boolean;
		compact?: boolean;
		onSelectCluster?: (cluster: MapCluster) => void;
		onDropPin?: (lat: number, lng: number) => void;
	}

	let {
		memos = [],
		selectedClusterId = null,
		droppedPin = null,
		focusMemoUid = null,
		pickerMode = false,
		compact = false,
		onSelectCluster,
		onDropPin
	}: Props = $props();

	let mapEl = $state<HTMLDivElement | null>(null);
	let L = $state<typeof LeafletNS | null>(null);
	let map = $state<LeafletNS.Map | null>(null);
	let markerLayer: LeafletNS.LayerGroup | null = null;
	let droppedLayer: LeafletNS.LayerGroup | null = null;
	let zoomLevel = $state(2);
	let hasFittedInitial = false;
	let resizeObserver: ResizeObserver | null = null;
	let detachPinchGuard: (() => void) | null = null;
	let destroyed = false;
	const invalidateTimers: ReturnType<typeof setTimeout>[] = [];

	function computeClusters(
		leaflet: typeof LeafletNS,
		mapInst: LeafletNS.Map,
		items: MemoView[]
	): MapCluster[] {
		const located = items.filter(
			(m) =>
				m.memo.payload.location &&
				Number.isFinite(m.memo.payload.location.latitude) &&
				Number.isFinite(m.memo.payload.location.longitude)
		);
		const clusters: {
			latSum: number;
			lngSum: number;
			px: number;
			py: number;
			memos: MemoView[];
		}[] = [];

		const CLUSTER_RADIUS_PX = 38;

		for (const item of located) {
			const loc = item.memo.payload.location!;
			const pt = mapInst.latLngToContainerPoint(leaflet.latLng(loc.latitude, loc.longitude));
			let target = clusters.find((c) => Math.hypot(c.px - pt.x, c.py - pt.y) <= CLUSTER_RADIUS_PX);
			if (!target) {
				target = { latSum: 0, lngSum: 0, px: pt.x, py: pt.y, memos: [] };
				clusters.push(target);
			}
			target.latSum += loc.latitude;
			target.lngSum += loc.longitude;
			target.memos.push(item);
		}

		return clusters.map((c) => {
			const count = c.memos.length;
			const lat = c.latSum / count;
			const lng = c.lngSum / count;
			const first = c.memos[0]!;
			const label =
				first.memo.payload.location?.placeholder?.trim() ||
				`${lat.toFixed(4)}, ${lng.toFixed(4)}`;
			const ids = c.memos.map((m) => m.memo.id).join('-');
			return {
				id: `c-${ids}`,
				latitude: lat,
				longitude: lng,
				memos: c.memos,
				label
			};
		});
	}

	function renderMarkers(): void {
		if (!L || !map || !markerLayer) return;
		const leaflet = L;
		const mapInst = map;
		markerLayer.clearLayers();

		const clusters = computeClusters(leaflet, mapInst, memos);
		for (const cluster of clusters) {
			const isSelected = selectedClusterId === cluster.id;
			const isMulti = cluster.memos.length > 1;
			const first = cluster.memos[0]!;
			const initial = (first.creator?.nickname || first.creator?.username || 'M')
				.slice(0, 1)
				.toUpperCase();

			const html = isMulti
				? `<div class="xm-map-cluster ${isSelected ? 'xm-map-marker-active' : ''}">${cluster.memos.length}</div>`
				: `<div class="xm-map-pin ${isSelected ? 'xm-map-marker-active' : ''}">${initial}</div>`;

			const icon = leaflet.divIcon({
				html,
				className: 'xm-map-icon-wrapper',
				iconSize: isMulti ? [32, 32] : [30, 30],
				iconAnchor: isMulti ? [16, 16] : [15, 15]
			});

			const marker = leaflet.marker([cluster.latitude, cluster.longitude], { icon });
			marker.on('click', (ev) => {
				leaflet.DomEvent.stopPropagation(ev);
				onSelectCluster?.(cluster);
			});
			marker.addTo(markerLayer);
		}
	}

	function renderDroppedPin(): void {
		if (!L || !map || !droppedLayer) return;
		const leaflet = L;
		droppedLayer.clearLayers();
		if (!droppedPin) return;

		const icon = leaflet.divIcon({
			html: `<div class="xm-map-dropped-pin"></div>`,
			className: 'xm-map-icon-wrapper',
			iconSize: [20, 20],
			iconAnchor: [10, 10]
		});

		leaflet.marker([droppedPin.latitude, droppedPin.longitude], { icon }).addTo(droppedLayer);
	}

	export function fitAllResults(): void {
		if (!L || !map) return;
		const points: [number, number][] = [];
		for (const m of memos) {
			const loc = m.memo.payload.location;
			if (loc && Number.isFinite(loc.latitude) && Number.isFinite(loc.longitude)) {
				points.push([loc.latitude, loc.longitude]);
			}
		}
		if (droppedPin) {
			points.push([droppedPin.latitude, droppedPin.longitude]);
		}
		if (points.length === 0) {
			map.setView([20, 10], 2, { animate: true });
		} else if (points.length === 1) {
			map.setView(points[0]!, pickerMode ? 12 : 10, { animate: true });
		} else {
			const bounds = L.latLngBounds(points);
			map.fitBounds(bounds, { padding: [48, 48], maxZoom: 13, animate: true });
		}
	}

	function zoomIn(): void {
		map?.zoomIn();
	}

	function zoomOut(): void {
		map?.zoomOut();
	}

	onMount(() => {
		destroyed = false;

		void (async () => {
			const leaflet = await loadLeaflet();
			if (destroyed || !mapEl) return;

			const initialCenter: [number, number] = droppedPin
				? [droppedPin.latitude, droppedPin.longitude]
				: [20, 10];
			const initialZoom = droppedPin ? 12 : 2;

			const mapInst = leaflet.map(mapEl, {
				center: initialCenter,
				zoom: initialZoom,
				minZoom: 2,
				maxZoom: 18,
				zoomSnap: 0.25,
				zoomDelta: 1,
				wheelPxPerZoomLevel: 90,
				zoomControl: false,
				attributionControl: true,
				worldCopyJump: true
			});

			mapInst.attributionControl?.setPrefix(false);

			const tileLayer = createOfflineOsmTileLayer(
				leaflet,
				() => themeState.isDark,
				'&copy; OpenStreetMap contributors'
			);
			tileLayer.addTo(mapInst);

			markerLayer = leaflet.layerGroup().addTo(mapInst);
			droppedLayer = leaflet.layerGroup().addTo(mapInst);
			detachPinchGuard = attachMapPinchZoomGuard(mapEl, mapInst);
			L = leaflet;
			map = mapInst;

			mapInst.on('zoomend', () => {
				zoomLevel = mapInst.getZoom();
				renderMarkers();
			});

			mapInst.on('click', (ev: LeafletNS.LeafletMouseEvent) => {
				onDropPin?.(
					Math.round(ev.latlng.lat * 1_000_000) / 1_000_000,
					Math.round(ev.latlng.lng * 1_000_000) / 1_000_000
				);
			});

			// Ensure Android WebView recalculates container bounds after .xm-page-enter & flex layout settle
			const safeInvalidate = () => {
				if (!destroyed && map === mapInst) {
					mapInst.invalidateSize({ animate: false });
				}
			};
			mapInst.whenReady(() => {
				requestAnimationFrame(safeInvalidate);
			});
			invalidateTimers.push(setTimeout(safeInvalidate, 160));
			invalidateTimers.push(setTimeout(safeInvalidate, 380));

			if (typeof ResizeObserver !== 'undefined') {
				resizeObserver = new ResizeObserver(() => {
					safeInvalidate();
				});
				resizeObserver.observe(mapEl);
			}
		})();
	});

	onDestroy(() => {
		destroyed = true;
		for (const timer of invalidateTimers) {
			clearTimeout(timer);
		}
		invalidateTimers.length = 0;
		detachPinchGuard?.();
		detachPinchGuard = null;
		resizeObserver?.disconnect();
		resizeObserver = null;
		map?.remove();
		map = null;
		markerLayer = null;
		droppedLayer = null;
	});

	$effect(() => {
		const _memos = memos;
		const _sel = selectedClusterId;
		const _zoom = zoomLevel;
		const _leaflet = L;
		const _map = map;
		const _focusUid = focusMemoUid;
		if (!_leaflet || !_map) return;
		void [_memos, _sel, _zoom];
		untrack(() => {
			renderMarkers();
			if (!hasFittedInitial && _memos.length > 0 && !_focusUid) {
				hasFittedInitial = true;
				fitAllResults();
			}
		});
	});

	$effect(() => {
		const _pin = droppedPin;
		const _leaflet = L;
		const _map = map;
		if (!_leaflet || !_map) return;
		untrack(() => {
			renderDroppedPin();
			if (pickerMode && _pin) {
				_map.setView([_pin.latitude, _pin.longitude], Math.max(_map.getZoom(), 11), {
					animate: true
				});
			}
		});
	});

	$effect(() => {
		const uid = focusMemoUid;
		const _leaflet = L;
		const _map = map;
		const _memos = memos;
		if (!uid || !_leaflet || !_map) return;
		untrack(() => {
			const target = _memos.find((m) => m.memo.uid === uid && m.memo.payload.location);
			if (!target?.memo.payload.location) return;
			const { latitude, longitude } = target.memo.payload.location;
			_map.setView([latitude, longitude], 12, { animate: true });
			hasFittedInitial = true;
		});
	});
</script>

<div
	class="relative isolate z-0 h-full w-full overflow-hidden rounded-2xl border border-[var(--xm-divider)] bg-[#e8eef5] dark:bg-[#181a1f]"
>
	<div
		bind:this={mapEl}
		class="xm-osm-map absolute inset-0 h-full w-full {themeState.isDark ? 'xm-osm-dark' : ''}"
		aria-label={i18n.t('map.canvasLabel')}
	></div>

	<!-- Official Memos Floating Bottom-Left Zoom & Fit Controls -->
	<div
		class="absolute bottom-3 left-3 z-[400] flex flex-col overflow-hidden rounded-xl border border-[var(--xm-divider)] bg-white/90 shadow-lg backdrop-blur-md dark:bg-[#1c1c1e]/90"
	>
		<button
			type="button"
			aria-label={i18n.t('map.zoomIn')}
			title={i18n.t('map.zoomIn')}
			class="flex {compact ? 'h-8 w-8' : 'h-9 w-9'} items-center justify-center xm-text-primary transition-colors hover:bg-[var(--xm-surface-subtle)]"
			onclick={zoomIn}
		>
			<Plus class="h-4 w-4" aria-hidden="true" />
		</button>
		<div class="h-px bg-[var(--xm-divider)]" aria-hidden="true"></div>
		<button
			type="button"
			aria-label={i18n.t('map.zoomOut')}
			title={i18n.t('map.zoomOut')}
			class="flex {compact ? 'h-8 w-8' : 'h-9 w-9'} items-center justify-center xm-text-primary transition-colors hover:bg-[var(--xm-surface-subtle)]"
			onclick={zoomOut}
		>
			<Minus class="h-4 w-4" aria-hidden="true" />
		</button>
		<div class="h-px bg-[var(--xm-divider)]" aria-hidden="true"></div>
		<button
			type="button"
			aria-label={i18n.t('map.showAll')}
			title={i18n.t('map.showAll')}
			class="flex {compact ? 'h-8 w-8' : 'h-9 w-9'} items-center justify-center xm-text-primary transition-colors hover:bg-[var(--xm-surface-subtle)]"
			onclick={fitAllResults}
		>
			<LocateFixed class="h-4 w-4" aria-hidden="true" />
		</button>
	</div>
</div>
