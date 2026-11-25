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
	import { MapPin } from 'lucide-svelte';

	import type { MemoLocation } from '#lib/types/memo.ts';
	import { i18n } from '#lib/i18n/index.svelte.ts';
	import { locationPreviewState } from '#lib/state/location-preview.svelte.ts';
	import {
		formatCoordinates,
		getCachedPlaceName,
		isCoordinateOnlyLabel,
		reverseGeocodeNearestPlace
	} from '#lib/utils/geocode.ts';

	interface Props {
		location: MemoLocation;
		memoUid: string;
	}

	let { location, memoUid }: Props = $props();

	let triggerEl = $state<HTMLButtonElement | null>(null);
	let resolvedLabel = $state<string | null>(null);

	let opened = $derived(locationPreviewState.active?.memoUid === memoUid);

	let displayLabel = $derived.by(() => {
		const raw = location.placeholder?.trim() ?? '';
		if (raw && !isCoordinateOnlyLabel(raw)) return raw;
		if (resolvedLabel) return resolvedLabel;
		const cached = getCachedPlaceName(location.latitude, location.longitude);
		if (cached) return cached;
		return raw || formatCoordinates(location.latitude, location.longitude);
	});

	// Lazily resolve nearest region name when saved memo has empty or coordinate-only placeholder
	$effect(() => {
		const lat = location.latitude;
		const lng = location.longitude;
		const raw = location.placeholder?.trim() ?? '';
		if (raw && !isCoordinateOnlyLabel(raw)) {
			resolvedLabel = null;
			return;
		}
		const cached = getCachedPlaceName(lat, lng);
		if (cached) {
			resolvedLabel = cached;
			return;
		}
		let cancelled = false;
		void reverseGeocodeNearestPlace(lat, lng, i18n.bcp47).then((name) => {
			if (!cancelled && name) {
				resolvedLabel = name;
				locationPreviewState.updateLabel(memoUid, name);
			}
		});
		return () => {
			cancelled = true;
		};
	});

	function handleClick(event: MouseEvent): void {
		event.stopPropagation();
		if (!triggerEl) return;
		locationPreviewState.toggle(memoUid, location, displayLabel, triggerEl.getBoundingClientRect());
	}
</script>

<button
	bind:this={triggerEl}
	type="button"
	aria-expanded={opened}
	aria-haspopup="dialog"
	class="flex max-w-full items-center gap-1.5 rounded-lg px-1.5 py-1 -mx-1.5 text-left transition-colors cursor-pointer {opened
		? 'bg-[var(--xm-surface-subtle)] text-[var(--xm-text-primary)] ring-1 ring-[var(--xm-divider)]'
		: 'xm-text-muted hover:bg-[var(--xm-surface-subtle)] hover:text-[var(--xm-text-primary)]'}"
	onclick={handleClick}
>
	<MapPin class="h-3.5 w-3.5 shrink-0" aria-hidden="true" />
	<span class="truncate">{displayLabel}</span>
</button>
