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
	import { onDestroy, onMount } from 'svelte';
	import { LocateFixed, Minus, Plus } from 'lucide-svelte';
	import type * as LeafletNS from 'leaflet';

	import { i18n } from '#lib/i18n/index.svelte.ts';
	import { themeState } from '#lib/state/theme.svelte.ts';
	import type { MemoLocation, MemoView } from '#lib/types/memo.ts';

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

	const TILE_CACHE_NAME = 'xmemos-osm-tiles-v1';

	/**
	 * Lightweight offline-first SVG fallback tile rendered when device is completely offline
	 * and a requested OpenStreetMap tile has not been cached locally yet.
	 */
	function createOfflineFallbackDataUrl(isDark: boolean): string {
		const bg = isDark ? '%23181a1f' : '%23e9edf2';
		const stroke = isDark ? '%232a2e38' : '%23d4dce6';
		return `data:image/svg+xml;utf8,<svg xmlns="http://www.w3.org/2000/svg" width="256" height="256"><rect width="256" height="256" fill="${bg}"/><path d="M0 0h256v256" fill="none" stroke="${stroke}" stroke-width="1"/></svg>`;
	}

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

	onMount(async () => {
		if (!mapEl) return;
		await import('leaflet/dist/leaflet.css');
		const leaflet = await import('leaflet');
		L = leaflet;

		const initialCenter: [number, number] = droppedPin
			? [droppedPin.latitude, droppedPin.longitude]
			: [20, 10];
		const initialZoom = droppedPin ? 12 : 2;

		const mapInst = leaflet.map(mapEl, {
			center: initialCenter,
			zoom: initialZoom,
			minZoom: 2,
			maxZoom: 18,
			zoomControl: false,
			attributionControl: true,
			worldCopyJump: true
		});

		// Offline-first OpenStreetMap / OsmAnd-compatible TileLayer with Cache API persistence
		const OfflineOsmTileLayer = leaflet.TileLayer.extend({
			createTile(coords: LeafletNS.Coords, done: LeafletNS.DoneCallback) {
				const img = document.createElement('img');
				img.alt = '';
				img.setAttribute('role', 'presentation');
				const url = (this as unknown as LeafletNS.TileLayer).getTileUrl(coords);
				const fallback = createOfflineFallbackDataUrl(themeState.isDark);

				const finishFallback = () => {
					img.src = fallback;
					done(undefined, img);
				};

				if (typeof caches !== 'undefined') {
					void caches
						.open(TILE_CACHE_NAME)
						.then(async (cache) => {
							const cached = await cache.match(url);
							if (cached) {
								const blob = await cached.blob();
								const objUrl = URL.createObjectURL(blob);
								img.onload = () => {
									URL.revokeObjectURL(objUrl);
									done(undefined, img);
								};
								img.onerror = finishFallback;
								img.src = objUrl;
								return;
							}
							if (!navigator.onLine) {
								finishFallback();
								return;
							}
							try {
								const resp = await fetch(url, { mode: 'cors' });
								if (!resp.ok) {
									finishFallback();
									return;
								}
								void cache.put(url, resp.clone()).catch(() => {});
								const blob = await resp.blob();
								const objUrl = URL.createObjectURL(blob);
								img.onload = () => {
									URL.revokeObjectURL(objUrl);
									done(undefined, img);
								};
								img.onerror = finishFallback;
								img.src = objUrl;
							} catch {
								finishFallback();
							}
						})
						.catch(() => {
							img.onload = () => done(undefined, img);
							img.onerror = finishFallback;
							img.src = url;
						});
				} else {
					img.onload = () => done(undefined, img);
					img.onerror = finishFallback;
					img.src = url;
				}

				return img;
			}
		});

		mapInst.attributionControl?.setPrefix(false);

		const tileLayer = new (OfflineOsmTileLayer as unknown as new (
			url: string,
			opts: LeafletNS.TileLayerOptions
		) => LeafletNS.TileLayer)('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
			maxZoom: 18,
			attribution: '&copy; OpenStreetMap contributors'
		});
		tileLayer.addTo(mapInst);

		markerLayer = leaflet.layerGroup().addTo(mapInst);
		droppedLayer = leaflet.layerGroup().addTo(mapInst);
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

		if (typeof ResizeObserver !== 'undefined') {
			resizeObserver = new ResizeObserver(() => {
				mapInst.invalidateSize({ animate: false });
			});
			resizeObserver.observe(mapEl);
		}
	});

	onDestroy(() => {
		resizeObserver?.disconnect();
		map?.remove();
		map = null;
	});

	$effect(() => {
		const _memos = memos;
		const _sel = selectedClusterId;
		const _zoom = zoomLevel;
		void [_memos, _sel, _zoom];
		if (!L || !map) return;
		renderMarkers();
		if (!hasFittedInitial && _memos.length > 0 && !focusMemoUid) {
			hasFittedInitial = true;
			fitAllResults();
		}
	});

	$effect(() => {
		const _pin = droppedPin;
		void _pin;
		renderDroppedPin();
		if (pickerMode && _pin && map) {
			map.setView([_pin.latitude, _pin.longitude], Math.max(map.getZoom(), 11), { animate: true });
		}
	});

	$effect(() => {
		const uid = focusMemoUid;
		if (!uid || !L || !map) return;
		const target = memos.find((m) => m.memo.uid === uid && m.memo.payload.location);
		if (!target?.memo.payload.location) return;
		const { latitude, longitude } = target.memo.payload.location;
		map.setView([latitude, longitude], 12, { animate: true });
		hasFittedInitial = true;
	});
</script>

<div
	class="relative isolate z-0 h-full w-full overflow-hidden rounded-2xl border border-[var(--xm-divider)] bg-[#e9edf2] dark:bg-[#181a1f]"
>
	<div
		bind:this={mapEl}
		class="xm-osm-map h-full w-full {themeState.isDark ? 'xm-osm-dark' : ''}"
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
