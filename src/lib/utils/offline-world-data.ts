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

/**
 * Compact, 100% FOSS offline world cartography dataset (public-domain Natural Earth-derived
 * simplified coastline rings in [longitude, latitude] degrees + major global & Indonesian cities).
 * Enables instant 60fps offline map rendering with zero internet access.
 */

export type LonLat = readonly [number, number];

export interface OfflineLandmass {
	readonly name: string;
	readonly ring: readonly LonLat[];
}

export interface OfflineCity {
	readonly name: string;
	readonly lat: number;
	readonly lng: number;
	/** Minimum zoom level at which this city label is rendered */
	readonly minZoom: number;
}

export const OFFLINE_LANDMASSES: readonly OfflineLandmass[] = [
	{
		name: 'North America',
		ring: [
			[-168, 66], [-162, 70], [-140, 70], [-125, 70], [-100, 68], [-82, 65],
			[-78, 54], [-64, 60], [-55, 52], [-66, 44], [-70, 41], [-75, 35],
			[-80, 25], [-83, 29], [-90, 29], [-97, 26], [-96, 19], [-88, 21],
			[-87, 16], [-83, 10], [-78, 8], [-82, 8], [-86, 11], [-92, 14],
			[-104, 19], [-110, 24], [-117, 32], [-122, 37], [-124, 48], [-135, 57],
			[-150, 61], [-162, 58], [-168, 66]
		]
	},
	{
		name: 'Greenland',
		ring: [
			[-52, 60], [-42, 60], [-32, 68], [-18, 76], [-26, 82], [-50, 82],
			[-70, 77], [-60, 70], [-52, 60]
		]
	},
	{
		name: 'Cuba & Hispaniola',
		ring: [
			[-85, 22], [-79, 23], [-74, 20], [-76, 19], [-82, 21], [-85, 22]
		]
	},
	{
		name: 'South America',
		ring: [
			[-78, 9], [-72, 12], [-62, 10], [-50, 1], [-35, -6], [-38, -14],
			[-42, -23], [-52, -33], [-58, -38], [-66, -46], [-68, -55], [-74, -52],
			[-73, -38], [-71, -18], [-81, -5], [-79, 2], [-78, 9]
		]
	},
	{
		name: 'Europe',
		ring: [
			[-9, 37], [-9, 43], [-2, 43], [-5, 48], [2, 51], [8, 54], [10, 57],
			[14, 54], [20, 55], [24, 60], [28, 60], [30, 69], [42, 67], [58, 69],
			[60, 55], [50, 46], [40, 44], [36, 46], [29, 41], [24, 38], [20, 40],
			[15, 45], [13, 41], [16, 38], [12, 42], [8, 44], [3, 42], [-5, 36], [-9, 37]
		]
	},
	{
		name: 'Scandinavia',
		ring: [
			[5, 59], [11, 59], [18, 60], [21, 65], [28, 70], [20, 70], [12, 66], [5, 61], [5, 59]
		]
	},
	{
		name: 'Great Britain',
		ring: [
			[-5, 50], [1, 51], [2, 53], [-1, 56], [-3, 58], [-6, 57], [-5, 53], [-5, 50]
		]
	},
	{
		name: 'Ireland',
		ring: [
			[-10, 52], [-6, 52], [-6, 55], [-9, 55], [-10, 52]
		]
	},
	{
		name: 'Iceland',
		ring: [
			[-24, 64], [-14, 64], [-14, 66], [-23, 66], [-24, 64]
		]
	},
	{
		name: 'Africa',
		ring: [
			[-17, 15], [-13, 28], [-6, 36], [10, 37], [15, 32], [25, 32],
			[33, 31], [40, 20], [43, 12], [51, 11], [41, -2], [40, -15],
			[35, -24], [28, -33], [18, -34], [15, -25], [12, -12], [9, 4],
			[2, 6], [-8, 5], [-15, 11], [-17, 15]
		]
	},
	{
		name: 'Madagascar',
		ring: [
			[44, -25], [47, -25], [50, -15], [49, -12], [44, -18], [44, -25]
		]
	},
	{
		name: 'Asia',
		ring: [
			[27, 41], [36, 36], [35, 31], [43, 13], [53, 16], [59, 22], [50, 29],
			[56, 26], [66, 25], [72, 20], [77, 8], [82, 16], [89, 22], [95, 16],
			[98, 12], [100, 6], [104, 1], [104, 10], [109, 13], [108, 21], [114, 22],
			[121, 25], [122, 31], [120, 36], [124, 40], [129, 35], [130, 42], [140, 48],
			[142, 55], [160, 60], [178, 66], [165, 70], [135, 72], [105, 76], [80, 73],
			[60, 69], [58, 55], [48, 45], [38, 44], [27, 41]
		]
	},
	{
		name: 'Sri Lanka',
		ring: [
			[80, 6], [82, 7], [81, 10], [79.8, 8], [80, 6]
		]
	},
	{
		name: 'Sumatra',
		ring: [
			[95.3, 5.5], [97.8, 4.2], [99.8, 2.5], [102.5, 0.5], [104.8, -2.5],
			[105.8, -5.8], [104.2, -5.6], [102.0, -3.5], [100.0, -0.8], [97.5, 2.2],
			[95.3, 5.5]
		]
	},
	{
		name: 'Java',
		ring: [
			[105.3, -6.7], [107.6, -6.1], [110.5, -6.8], [113.5, -7.7], [114.5, -8.3],
			[113.8, -8.7], [110.8, -8.1], [107.8, -7.7], [105.7, -7.0], [105.3, -6.7]
		]
	},
	{
		name: 'Bali & Nusa Tenggara',
		ring: [
			[114.5, -8.2], [115.6, -8.4], [116.6, -8.6], [118.8, -8.5], [123.8, -10.0],
			[122.0, -9.2], [117.5, -8.9], [115.0, -8.8], [114.5, -8.2]
		]
	},
	{
		name: 'Kalimantan (Borneo)',
		ring: [
			[109.0, 1.5], [112.0, 3.0], [116.8, 6.8], [118.8, 4.8], [117.8, 1.2],
			[116.2, -3.5], [113.2, -3.4], [110.2, -2.8], [109.0, 0.0], [109.0, 1.5]
		]
	},
	{
		name: 'Sulawesi',
		ring: [
			[119.0, -5.5], [120.5, -5.0], [121.8, -4.5], [122.8, -1.2], [125.0, 1.5],
			[123.5, 1.0], [120.8, 1.0], [119.5, -1.2], [118.8, -3.0], [119.0, -5.5]
		]
	},
	{
		name: 'New Guinea (Papua)',
		ring: [
			[131.0, -1.5], [134.5, -1.0], [138.0, -2.5], [141.0, -2.7], [146.5, -6.5],
			[150.5, -10.3], [146.5, -9.5], [141.0, -9.0], [138.0, -8.0], [134.5, -4.5],
			[131.5, -3.0], [131.0, -1.5]
		]
	},
	{
		name: 'Philippines',
		ring: [
			[120.0, 18.4], [122.4, 17.5], [124.2, 13.0], [126.2, 7.2], [123.5, 6.5],
			[121.0, 10.5], [119.8, 14.8], [120.0, 18.4]
		]
	},
	{
		name: 'Taiwan',
		ring: [
			[120.2, 22.0], [121.2, 22.7], [121.9, 25.1], [120.8, 24.8], [120.2, 22.0]
		]
	},
	{
		name: 'Japan',
		ring: [
			[130.5, 31.5], [133.5, 33.5], [137.0, 34.8], [140.8, 35.8], [141.8, 41.0],
			[145.5, 43.5], [141.5, 45.2], [139.5, 41.0], [136.5, 37.0], [131.0, 34.0],
			[130.5, 31.5]
		]
	},
	{
		name: 'Australia',
		ring: [
			[114, -22], [122, -18], [131, -12], [136, -12], [142, -11], [146, -19],
			[153, -27], [150, -37], [141, -38], [135, -34], [125, -33], [115, -34],
			[113, -26], [114, -22]
		]
	},
	{
		name: 'Tasmania',
		ring: [
			[144.8, -41.0], [148.2, -41.0], [147.8, -43.4], [145.2, -43.0], [144.8, -41.0]
		]
	},
	{
		name: 'New Zealand North',
		ring: [
			[173.0, -34.8], [178.4, -37.7], [175.4, -41.4], [173.8, -39.2], [173.0, -34.8]
		]
	},
	{
		name: 'New Zealand South',
		ring: [
			[172.8, -40.8], [174.0, -41.8], [170.5, -45.8], [167.0, -46.2], [168.5, -43.5],
			[172.8, -40.8]
		]
	},
	{
		name: 'Antarctica',
		ring: [
			[-180, -72], [-120, -73], [-60, -64], [-40, -74], [0, -70],
			[60, -68], [120, -67], [180, -72], [180, -85], [-180, -85], [-180, -72]
		]
	}
];

export const OFFLINE_CITIES: readonly OfflineCity[] = [
	// Indonesia & Southeast Asia
	{ name: 'Jakarta', lat: -6.2088, lng: 106.8456, minZoom: 3 },
	{ name: 'Bandung', lat: -6.9175, lng: 107.6191, minZoom: 6 },
	{ name: 'Surabaya', lat: -7.2575, lng: 112.7521, minZoom: 5 },
	{ name: 'Yogyakarta', lat: -7.7956, lng: 110.3695, minZoom: 6 },
	{ name: 'Semarang', lat: -6.9667, lng: 110.4167, minZoom: 6 },
	{ name: 'Medan', lat: 3.5952, lng: 98.6722, minZoom: 5 },
	{ name: 'Palembang', lat: -2.9761, lng: 104.7754, minZoom: 6 },
	{ name: 'Denpasar', lat: -8.6705, lng: 115.2126, minZoom: 6 },
	{ name: 'Makassar', lat: -5.1477, lng: 119.4327, minZoom: 5 },
	{ name: 'Balikpapan', lat: -1.2379, lng: 116.8529, minZoom: 6 },
	{ name: 'Manado', lat: 1.4748, lng: 124.8421, minZoom: 6 },
	{ name: 'Jayapura', lat: -2.5916, lng: 140.669, minZoom: 5 },
	{ name: 'Singapore', lat: 1.3521, lng: 103.8198, minZoom: 3 },
	{ name: 'Kuala Lumpur', lat: 3.139, lng: 101.6869, minZoom: 4 },
	{ name: 'Bangkok', lat: 13.7563, lng: 100.5018, minZoom: 4 },
	{ name: 'Ho Chi Minh City', lat: 10.8231, lng: 106.6297, minZoom: 4 },
	{ name: 'Manila', lat: 14.5995, lng: 120.9842, minZoom: 4 },
	// Asia-Pacific
	{ name: 'Tokyo', lat: 35.6762, lng: 139.6503, minZoom: 3 },
	{ name: 'Seoul', lat: 37.5665, lng: 126.978, minZoom: 4 },
	{ name: 'Beijing', lat: 39.9042, lng: 116.4074, minZoom: 3 },
	{ name: 'Shanghai', lat: 31.2304, lng: 121.4737, minZoom: 4 },
	{ name: 'Taipei', lat: 25.033, lng: 121.5654, minZoom: 5 },
	{ name: 'New Delhi', lat: 28.6139, lng: 77.209, minZoom: 3 },
	{ name: 'Mumbai', lat: 19.076, lng: 72.8777, minZoom: 4 },
	{ name: 'Dubai', lat: 25.2048, lng: 55.2708, minZoom: 4 },
	{ name: 'Riyadh', lat: 24.7136, lng: 46.6753, minZoom: 4 },
	{ name: 'Sydney', lat: -33.8688, lng: 151.2093, minZoom: 3 },
	{ name: 'Melbourne', lat: -37.8136, lng: 144.9631, minZoom: 4 },
	{ name: 'Auckland', lat: -36.8509, lng: 174.7645, minZoom: 4 },
	// Europe & Africa
	{ name: 'London', lat: 51.5072, lng: -0.1276, minZoom: 3 },
	{ name: 'Paris', lat: 48.8566, lng: 2.3522, minZoom: 4 },
	{ name: 'Berlin', lat: 52.52, lng: 13.405, minZoom: 4 },
	{ name: 'Madrid', lat: 40.4168, lng: -3.7038, minZoom: 4 },
	{ name: 'Rome', lat: 41.9028, lng: 12.4964, minZoom: 4 },
	{ name: 'Istanbul', lat: 41.0082, lng: 28.9784, minZoom: 4 },
	{ name: 'Moscow', lat: 55.7558, lng: 37.6173, minZoom: 3 },
	{ name: 'Cairo', lat: 30.0444, lng: 31.2357, minZoom: 3 },
	{ name: 'Lagos', lat: 6.5244, lng: 3.3792, minZoom: 4 },
	{ name: 'Nairobi', lat: -1.2921, lng: 36.8219, minZoom: 4 },
	{ name: 'Johannesburg', lat: -26.2041, lng: 28.0473, minZoom: 3 },
	// Americas
	{ name: 'New York', lat: 40.7128, lng: -74.006, minZoom: 3 },
	{ name: 'San Francisco', lat: 37.7749, lng: -122.4194, minZoom: 4 },
	{ name: 'Los Angeles', lat: 34.0522, lng: -118.2437, minZoom: 4 },
	{ name: 'Toronto', lat: 43.6532, lng: -79.3832, minZoom: 4 },
	{ name: 'Mexico City', lat: 19.4326, lng: -99.1332, minZoom: 3 },
	{ name: 'Bogota', lat: 4.711, lng: -74.0721, minZoom: 4 },
	{ name: 'Sao Paulo', lat: -23.5558, lng: -46.6396, minZoom: 3 },
	{ name: 'Buenos Aires', lat: -34.6037, lng: -58.3816, minZoom: 3 }
];
