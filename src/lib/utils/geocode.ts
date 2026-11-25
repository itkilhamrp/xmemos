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

const GEOCODE_CACHE_KEY = 'xmemos.geocode.cache.v1';
const MAX_CACHE_ENTRIES = 250;

interface NominatimAddress {
	neighbourhood?: string;
	suburb?: string;
	village?: string;
	town?: string;
	city_district?: string;
	city?: string;
	municipality?: string;
	county?: string;
	state_district?: string;
	state?: string;
	region?: string;
	country?: string;
}

interface NominatimResponse {
	error?: string;
	name?: string;
	display_name?: string;
	address?: NominatimAddress;
}

interface PhotonFeatureProperties {
	name?: string;
	locality?: string;
	district?: string;
	city?: string;
	county?: string;
	state?: string;
	country?: string;
}

interface PhotonResponse {
	features?: { properties?: PhotonFeatureProperties }[];
}

const memoryCache = new Map<string, string>();
const inFlightRequests = new Map<string, Promise<string | null>>();
let storageLoaded = false;

function cacheKey(lat: number, lng: number): string {
	return `${lat.toFixed(3)},${lng.toFixed(3)}`;
}

function loadStorageCache(): void {
	if (storageLoaded || typeof window === 'undefined') return;
	storageLoaded = true;
	try {
		const raw = window.localStorage.getItem(GEOCODE_CACHE_KEY);
		if (!raw) return;
		const parsed = JSON.parse(raw) as Record<string, string>;
		for (const [k, v] of Object.entries(parsed)) {
			if (typeof v === 'string' && v.trim() && !isCoordinateOnlyLabel(v)) {
				memoryCache.set(k, v);
			}
		}
	} catch {
		// Ignore malformed localStorage cache
	}
}

function persistStorageCache(): void {
	if (typeof window === 'undefined') return;
	try {
		const entries = Array.from(memoryCache.entries()).slice(-MAX_CACHE_ENTRIES);
		window.localStorage.setItem(GEOCODE_CACHE_KEY, JSON.stringify(Object.fromEntries(entries)));
	} catch {
		// Ignore storage quota errors
	}
}

/** Returns true if a location label is empty or contains only numeric lat/lng coordinates. */
export function isCoordinateOnlyLabel(label: string | null | undefined): boolean {
	if (!label) return true;
	const trimmed = label.trim();
	if (!trimmed) return true;
	return /^-?\d+(?:\.\d+)?°?\s*,\s*-?\d+(?:\.\d+)?°?$/.test(trimmed);
}

/** Formats latitude & longitude in Official Memos degree notation: "37.995400°, -123.020400°". */
export function formatCoordinates(lat: number, lng: number): string {
	return `${lat.toFixed(6)}°, ${lng.toFixed(6)}°`;
}

function dedupeParts(candidates: (string | undefined)[]): string | null {
	const parts: string[] = [];
	for (const part of candidates) {
		const clean = part?.trim();
		if (clean && !parts.some((existing) => existing.toLowerCase() === clean.toLowerCase())) {
			parts.push(clean);
		}
	}
	return parts.length > 0 ? parts.join(', ') : null;
}

/** Builds a concise, human-readable regional label (e.g. "Point Reyes, California, United States"). */
export function formatNominatimPlace(data: NominatimResponse): string | null {
	if (data.error) return null;
	const addr = data.address;
	if (addr) {
		const primary =
			addr.village ||
			addr.town ||
			addr.suburb ||
			addr.city_district ||
			addr.city ||
			addr.municipality ||
			addr.county ||
			addr.neighbourhood ||
			data.name;
		const secondary = addr.state || addr.region || addr.state_district;
		const formatted = dedupeParts([primary, secondary, addr.country]);
		if (formatted) return formatted;
	}

	if (data.display_name) {
		const segments = data.display_name
			.split(',')
			.map((s) => s.trim())
			.filter(Boolean);
		if (segments.length > 0) {
			return segments.slice(0, 3).join(', ');
		}
	}

	return null;
}

/** Formats a Komoot Photon (OpenStreetMap FOSS) reverse-geocode response into a regional label. */
export function formatPhotonPlace(data: PhotonResponse): string | null {
	const props = data.features?.[0]?.properties;
	if (!props) return null;
	const primary = props.city || props.locality || props.district || props.county || props.name;
	return dedupeParts([primary, props.state, props.country]);
}

async function fetchJsonWithTimeout<T>(
	url: string,
	timeoutMs: number,
	headers: Record<string, string>
): Promise<T | null> {
	const controller = new AbortController();
	const timer = setTimeout(() => controller.abort(), timeoutMs);
	try {
		const resp = await fetch(url, { headers, signal: controller.signal });
		if (!resp.ok) return null;
		return (await resp.json()) as T;
	} catch {
		return null;
	} finally {
		clearTimeout(timer);
	}
}

async function queryNominatim(lat: number, lng: number, zoom: number, lang: string): Promise<string | null> {
	const url =
		`https://nominatim.openstreetmap.org/reverse?format=jsonv2` +
		`&lat=${encodeURIComponent(lat.toFixed(6))}` +
		`&lon=${encodeURIComponent(lng.toFixed(6))}` +
		`&zoom=${zoom}&addressdetails=1`;
	const data = await fetchJsonWithTimeout<NominatimResponse>(url, 3200, {
		Accept: 'application/json',
		'Accept-Language': lang
	});
	return data ? formatNominatimPlace(data) : null;
}

async function queryPhoton(lat: number, lng: number): Promise<string | null> {
	const url =
		`https://photon.komoot.io/reverse?lat=${encodeURIComponent(lat.toFixed(6))}` +
		`&lon=${encodeURIComponent(lng.toFixed(6))}&limit=1`;
	const data = await fetchJsonWithTimeout<PhotonResponse>(url, 3000, {
		Accept: 'application/json'
	});
	return data ? formatPhotonPlace(data) : null;
}

/** Returns a cached nearest region name synchronously if available. */
export function getCachedPlaceName(lat: number, lng: number): string | null {
	loadStorageCache();
	return memoryCache.get(cacheKey(lat, lng)) ?? null;
}

/**
 * Resolves the nearest place/region name via multi-tier FOSS OpenStreetMap reverse geocoding:
 * 1. Instant localStorage/memory cache
 * 2. OpenStreetMap Nominatim (zoom=10 town/region, falling back to zoom=6 province/country for remote areas)
 * 3. Komoot Photon (OpenStreetMap FOSS fallback if Nominatim rate-limits or returns empty)
 * Always resolves cleanly without throwing and degrades gracefully offline.
 */
export async function reverseGeocodeNearestPlace(
	lat: number,
	lng: number,
	lang = 'en'
): Promise<string | null> {
	if (!Number.isFinite(lat) || !Number.isFinite(lng)) return null;
	loadStorageCache();
	const key = cacheKey(lat, lng);
	const cached = memoryCache.get(key);
	if (cached) return cached;

	if (typeof fetch === 'undefined' || (typeof navigator !== 'undefined' && !navigator.onLine)) {
		return null;
	}

	const existing = inFlightRequests.get(key);
	if (existing) return existing;

	const task = (async (): Promise<string | null> => {
		let place = await queryNominatim(lat, lng, 10, lang);
		if (!place) {
			place = await queryPhoton(lat, lng);
		}
		if (!place) {
			place = await queryNominatim(lat, lng, 6, lang);
		}
		if (place && !isCoordinateOnlyLabel(place)) {
			memoryCache.set(key, place);
			persistStorageCache();
			return place;
		}
		return null;
	})().finally(() => {
		inFlightRequests.delete(key);
	});

	inFlightRequests.set(key, task);
	return task;
}
