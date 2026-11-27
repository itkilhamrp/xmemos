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

import type * as LeafletNS from 'leaflet';

export const OSM_TILE_URL = 'https://tile.openstreetmap.org/{z}/{x}/{y}.png';
export const OSM_TILE_CACHE_NAME = 'xmemos-osm-tiles-v1';
// 1x1 transparent PNG so uncached/offline OSM tiles reveal the 60fps offline vector world map underneath
const TRANSPARENT_TILE_DATA_URL =
	'data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7';

/**
 * Dynamically imports Leaflet + its stylesheet and safely unwraps ESM/CJS default exports.
 */
export async function loadLeaflet(): Promise<typeof LeafletNS> {
	const mod = (await import('leaflet')) as typeof LeafletNS & { default?: typeof LeafletNS };
	return mod.default && typeof mod.default.map === 'function' ? mod.default : mod;
}

/**
 * Lightweight, clearly legible offline-first SVG grid tile rendered only when the device is
 * truly offline (or tile loading fails) and the requested OpenStreetMap tile is not yet cached.
 */
export function createOfflineFallbackDataUrl(isDark: boolean): string {
	const bg = isDark ? '%231e232d' : '%23e8eef5';
	const grid = isDark ? '%2330394a' : '%23cbd5e1';
	const major = isDark ? '%233e4a61' : '%23b8c5d6';
	return (
		`data:image/svg+xml;utf8,` +
		`<svg xmlns="http://www.w3.org/2000/svg" width="256" height="256">` +
		`<rect width="256" height="256" fill="${bg}"/>` +
		`<path d="M0 64H256M0 128H256M0 192H256M64 0V256M128 0V256M192 0V256" fill="none" stroke="${grid}" stroke-width="1"/>` +
		`<path d="M0 0H256V256" fill="none" stroke="${major}" stroke-width="1"/>` +
		`</svg>`
	);
}

/**
 * Creates an Android WebView-safe, offline-first OpenStreetMap TileLayer:
 * 1. Serves instantly from Cache API (`xmemos-osm-tiles-v1`) when available.
 * 2. Fetches online tiles with CORS and persists them to Cache API for offline reuse.
 * 3. Falls back to direct `<img>` loading if Cache API is unavailable or CORS fetch is rejected
 *    (critical on Android WebView `http://tauri.localhost` where `caches` or CORS may vary).
 * 4. Renders a legible SVG coordinate grid only when both cache and live image loading fail.
 */
export function createOfflineOsmTileLayer(
	leaflet: typeof LeafletNS,
	getIsDark: () => boolean,
	attribution = '&copy; OpenStreetMap contributors'
): LeafletNS.TileLayer {
	const OfflineOsmTileLayer = leaflet.TileLayer.extend({
		createTile(coords: LeafletNS.Coords, done: LeafletNS.DoneCallback) {
			const img = document.createElement('img');
			img.alt = '';
			img.decoding = 'async';
			img.setAttribute('role', 'presentation');

			const url = (this as unknown as LeafletNS.TileLayer).getTileUrl(coords);
			const fallback = TRANSPARENT_TILE_DATA_URL;
			let finished = false;

			const finish = (err?: Error) => {
				if (finished) return;
				finished = true;
				done(err, img);
			};

			const finishFallback = () => {
				img.onload = () => finish();
				img.onerror = () => finish();
				img.src = fallback;
			};

			const loadDirectImage = () => {
				if (typeof navigator !== 'undefined' && navigator.onLine === false) {
					finishFallback();
					return;
				}
				img.referrerPolicy = 'no-referrer-when-downgrade';
				img.onload = () => finish();
				img.onerror = finishFallback;
				img.src = url;
			};

			const loadBlob = (blob: Blob, onBlobError: () => void) => {
				const objUrl = URL.createObjectURL(blob);
				img.onload = () => {
					URL.revokeObjectURL(objUrl);
					finish();
				};
				img.onerror = () => {
					URL.revokeObjectURL(objUrl);
					onBlobError();
				};
				img.src = objUrl;
			};

			if (typeof caches !== 'undefined' && typeof caches.open === 'function') {
				void caches
					.open(OSM_TILE_CACHE_NAME)
					.then(async (cache) => {
						const cached = await cache.match(url);
						if (cached) {
							const blob = await cached.blob();
							loadBlob(blob, loadDirectImage);
							return;
						}
						if (typeof navigator !== 'undefined' && navigator.onLine === false) {
							finishFallback();
							return;
						}
						try {
							const resp = await fetch(url, {
								mode: 'cors',
								referrerPolicy: 'no-referrer-when-downgrade'
							});
							if (!resp.ok) {
								loadDirectImage();
								return;
							}
							void cache.put(url, resp.clone()).catch(() => {});
							const blob = await resp.blob();
							loadBlob(blob, loadDirectImage);
						} catch {
							// Android WebView / CORS fallback: still try direct <img> loading before offline grid
							loadDirectImage();
						}
					})
					.catch(() => {
						loadDirectImage();
					});
			} else {
				loadDirectImage();
			}

			return img;
		}
	});

	return new (OfflineOsmTileLayer as unknown as new (
		url: string,
		opts: LeafletNS.TileLayerOptions
	) => LeafletNS.TileLayer)(OSM_TILE_URL, {
		maxZoom: 18,
		zIndex: 2,
		attribution
	});
}

interface WebKitGestureEvent extends Event {
	scale?: number;
}

/**
 * Intercepts Desktop touchpad 2-finger pinch-to-zoom events (`wheel` with `ctrlKey: true` on
 * WebView2/Chromium/WebKitGTK, and `gesturestart`/`gesturechange`/`gestureend` on macOS WKWebView
 * and WebKitGTK) so pinching on the map smoothly zooms ONLY the Leaflet map and never zooms
 * the surrounding Xmemos desktop application UI.
 */
export function attachMapPinchZoomGuard(
	container: HTMLElement,
	mapInst: LeafletNS.Map
): () => void {
	let gestureBaseZoom = mapInst.getZoom();
	let pendingDeltaZoom = 0;
	let lastAnchorEvent: WheelEvent | null = null;
	let pinchRafId = 0;

	const clampZoom = (z: number): number => {
		const minZ = mapInst.getMinZoom();
		const maxZ = mapInst.getMaxZoom();
		return Math.min(maxZ, Math.max(minZ, z));
	};

	const flushPinchFrame = () => {
		pinchRafId = 0;
		if (!lastAnchorEvent || pendingDeltaZoom === 0) return;
		const delta = pendingDeltaZoom;
		const ev = lastAnchorEvent;
		pendingDeltaZoom = 0;
		const nextZoom = clampZoom(mapInst.getZoom() + delta);
		if (Math.abs(nextZoom - mapInst.getZoom()) < 0.001) return;
		const anchor = mapInst.mouseEventToLatLng(ev);
		mapInst.setZoomAround(anchor, nextZoom, { animate: false });
	};

	const onWheel = (event: WheelEvent) => {
		if (!event.ctrlKey) return;
		event.preventDefault();
		event.stopPropagation();

		// Normalize touchpad pinch deltaMode (pixel/line/page) & coalesce into 60/120fps rAF
		const scale = event.deltaMode === 1 ? 16 : event.deltaMode === 2 ? 100 : 1;
		const normalized = Math.max(-60, Math.min(60, event.deltaY * scale));
		const delta = -normalized * 0.015;
		if (!Number.isFinite(delta) || delta === 0) return;

		pendingDeltaZoom += delta;
		lastAnchorEvent = event;
		if (pinchRafId === 0) {
			pinchRafId = requestAnimationFrame(flushPinchFrame);
		}
	};

	const onGestureStart = (event: Event) => {
		event.preventDefault();
		event.stopPropagation();
		gestureBaseZoom = mapInst.getZoom();
	};

	const onGestureChange = (event: Event) => {
		event.preventDefault();
		event.stopPropagation();
		const scale = (event as WebKitGestureEvent).scale;
		if (typeof scale !== 'number' || !Number.isFinite(scale) || scale <= 0) return;
		const nextZoom = clampZoom(gestureBaseZoom + Math.log2(scale));
		mapInst.setZoom(nextZoom, { animate: false });
	};

	const onGestureEnd = (event: Event) => {
		event.preventDefault();
		event.stopPropagation();
	};

	container.addEventListener('wheel', onWheel, { passive: false, capture: true });
	container.addEventListener('gesturestart', onGestureStart, { passive: false });
	container.addEventListener('gesturechange', onGestureChange, { passive: false });
	container.addEventListener('gestureend', onGestureEnd, { passive: false });

	return () => {
		if (pinchRafId !== 0) cancelAnimationFrame(pinchRafId);
		container.removeEventListener('wheel', onWheel, { capture: true });
		container.removeEventListener('gesturestart', onGestureStart);
		container.removeEventListener('gesturechange', onGestureChange);
		container.removeEventListener('gestureend', onGestureEnd);
	};
}
