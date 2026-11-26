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
import { OFFLINE_CITIES, OFFLINE_LANDMASSES } from '#lib/utils/offline-world-data.ts';

const TILE_SIZE = 256;

function lonToWorldX(lon: number, z: number): number {
	return ((lon + 180) / 360) * TILE_SIZE * Math.pow(2, z);
}

function latToWorldY(lat: number, z: number): number {
	const clamped = Math.max(-85.0511, Math.min(85.0511, lat));
	const rad = (clamped * Math.PI) / 180;
	const n = Math.log(Math.tan(Math.PI / 4 + rad / 2));
	return ((1 - n / Math.PI) / 2) * TILE_SIZE * Math.pow(2, z);
}

function graticuleStepForZoom(z: number): number {
	if (z <= 2) return 30;
	if (z <= 4) return 15;
	if (z <= 6) return 5;
	if (z <= 9) return 1;
	if (z <= 12) return 0.25;
	return 0.05;
}

/**
 * High-performance 60fps HTML5 Canvas Leaflet GridLayer that renders a complete,
 * crisp offline vector world map (ocean, continents, islands, graticule, and major cities)
 * with ZERO internet dependency. Detailed online/cached OSM tiles layer transparently on top.
 */
export function createOfflineVectorTileLayer(
	leaflet: typeof LeafletNS,
	getIsDark: () => boolean
): LeafletNS.GridLayer {
	const OfflineVectorLayer = leaflet.GridLayer.extend({
		createTile(coords: LeafletNS.Coords) {
			const canvas = document.createElement('canvas');
			const dpr = typeof window !== 'undefined' ? Math.min(window.devicePixelRatio || 1, 2) : 1;
			canvas.width = Math.round(TILE_SIZE * dpr);
			canvas.height = Math.round(TILE_SIZE * dpr);
			canvas.style.width = `${TILE_SIZE}px`;
			canvas.style.height = `${TILE_SIZE}px`;

			const ctx = canvas.getContext('2d');
			if (!ctx) return canvas;
			ctx.scale(dpr, dpr);

			const isDark = getIsDark();
			const z = coords.z;
			const worldSpan = TILE_SIZE * Math.pow(2, z);
			const tileOriginX = coords.x * TILE_SIZE;
			const tileOriginY = coords.y * TILE_SIZE;

			// 1. Ocean fill
			ctx.fillStyle = isDark ? '#111a26' : '#d6e6f5';
			ctx.fillRect(0, 0, TILE_SIZE, TILE_SIZE);

			// 2. Wrap offset so panning across the antimeridian renders seamlessly
			const numTiles = Math.pow(2, z);
			const wrappedTileX = ((coords.x % numTiles) + numTiles) % numTiles;
			const wrapDeltaPx = (wrappedTileX - coords.x) * TILE_SIZE;

			// 3. Draw landmass polygons (supporting world wrap copies -1, 0, +1)
			ctx.fillStyle = isDark ? '#202b38' : '#eef2ea';
			ctx.strokeStyle = isDark ? '#3a4d63' : '#9fb4a5';
			ctx.lineWidth = z <= 4 ? 1 : 1.25;
			ctx.lineJoin = 'round';

			for (const worldShift of [-worldSpan, 0, worldSpan]) {
				const originX = tileOriginX + wrapDeltaPx - worldShift;
				for (const land of OFFLINE_LANDMASSES) {
					const ring = land.ring;
					if (ring.length < 3) continue;
					ctx.beginPath();
					for (let i = 0; i < ring.length; i++) {
						const [lon, lat] = ring[i]!;
						const x = lonToWorldX(lon, z) - originX;
						const y = latToWorldY(lat, z) - tileOriginY;
						if (i === 0) ctx.moveTo(x, y);
						else ctx.lineTo(x, y);
					}
					ctx.closePath();
					ctx.fill();
					ctx.stroke();
				}
			}

			// 4. Subtle lat/lng graticule lines for geographic orientation at any zoom
			const step = graticuleStepForZoom(z);
			ctx.strokeStyle = isDark ? 'rgba(148, 163, 184, 0.14)' : 'rgba(71, 85, 105, 0.14)';
			ctx.lineWidth = 0.75;

			const minLon = (wrappedTileX / numTiles) * 360 - 180;
			const maxLon = ((wrappedTileX + 1) / numTiles) * 360 - 180;
			const startLon = Math.floor(minLon / step) * step;
			for (let lon = startLon; lon <= maxLon + step; lon += step) {
				const x = lonToWorldX(lon, z) - (wrappedTileX * TILE_SIZE);
				if (x >= -1 && x <= TILE_SIZE + 1) {
					ctx.beginPath();
					ctx.moveTo(x, 0);
					ctx.lineTo(x, TILE_SIZE);
					ctx.stroke();
				}
			}

			// Approximate tile latitude bounds for graticule horizontals
			const yToLat = (wy: number) => {
				const n = Math.PI - (2 * Math.PI * wy) / worldSpan;
				return (180 / Math.PI) * Math.atan(0.5 * (Math.exp(n) - Math.exp(-n)));
			};
			const maxLat = yToLat(tileOriginY);
			const minLat = yToLat(tileOriginY + TILE_SIZE);
			const startLat = Math.floor(minLat / step) * step;
			for (let lat = startLat; lat <= maxLat + step; lat += step) {
				if (lat < -85 || lat > 85) continue;
				const y = latToWorldY(lat, z) - tileOriginY;
				if (y >= -1 && y <= TILE_SIZE + 1) {
					ctx.beginPath();
					ctx.moveTo(0, y);
					ctx.lineTo(TILE_SIZE, y);
					ctx.stroke();
				}
			}

			// 5. Render major world & Indonesian city dots + labels
			ctx.font = '600 10px -apple-system, BlinkMacSystemFont, system-ui, sans-serif';
			ctx.textBaseline = 'middle';
			const dotColor = isDark ? '#60a5fa' : '#0066cc';
			const textColor = isDark ? 'rgba(226, 232, 240, 0.88)' : 'rgba(15, 23, 42, 0.82)';
			const haloColor = isDark ? 'rgba(17, 26, 38, 0.85)' : 'rgba(238, 242, 234, 0.88)';

			for (const city of OFFLINE_CITIES) {
				if (z < city.minZoom) continue;
				const x = lonToWorldX(city.lng, z) - (wrappedTileX * TILE_SIZE);
				const y = latToWorldY(city.lat, z) - tileOriginY;
				if (x < -40 || x > TILE_SIZE + 40 || y < -16 || y > TILE_SIZE + 16) continue;

				ctx.fillStyle = dotColor;
				ctx.beginPath();
				ctx.arc(x, y, 2.4, 0, Math.PI * 2);
				ctx.fill();

				ctx.lineWidth = 2.5;
				ctx.strokeStyle = haloColor;
				ctx.strokeText(city.name, x + 5, y);
				ctx.fillStyle = textColor;
				ctx.fillText(city.name, x + 5, y);
			}

			return canvas;
		}
	});

	return new (OfflineVectorLayer as unknown as new (opts: LeafletNS.GridLayerOptions) => LeafletNS.GridLayer)({
		tileSize: TILE_SIZE,
		minZoom: 2,
		maxZoom: 18,
		zIndex: 1,
		updateWhenIdle: false,
		keepBuffer: 2
	});
}
