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

import { describe, expect, it } from 'vitest';
import {
	createOfflineFallbackDataUrl,
	OSM_TILE_CACHE_NAME,
	OSM_TILE_URL
} from '#lib/utils/osm-tiles.ts';

describe('osm-tiles', () => {
	it('uses canonical HTTPS OpenStreetMap tile URL and stable cache name', () => {
		expect(OSM_TILE_URL).toBe('https://tile.openstreetmap.org/{z}/{x}/{y}.png');
		expect(OSM_TILE_CACHE_NAME).toBe('xmemos-osm-tiles-v1');
	});

	it('generates legible light and dark offline SVG grid fallback data URLs', () => {
		const light = createOfflineFallbackDataUrl(false);
		const dark = createOfflineFallbackDataUrl(true);

		expect(light.startsWith('data:image/svg+xml;utf8,')).toBe(true);
		expect(dark.startsWith('data:image/svg+xml;utf8,')).toBe(true);
		expect(light).toContain('<svg');
		expect(dark).toContain('<svg');
		expect(light).not.toBe(dark);
		expect(dark).toContain('M0 64H256');
	});
});
