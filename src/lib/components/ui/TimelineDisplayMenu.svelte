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
	import { ChevronRight, SlidersHorizontal } from 'lucide-svelte';

	import { i18n } from '#lib/i18n/index.svelte.ts';
	import { feedDisplayState, type FeedColumnLayout } from '#lib/state/feed-display.svelte.ts';

	interface Props {
		sidebarMode?: boolean;
	}

	let { sidebarMode = false }: Props = $props();

	let triggerEl = $state<HTMLButtonElement | null>(null);

	const LAYOUT_LABELS: Record<
		FeedColumnLayout,
		'feed.layoutList' | 'feed.layout2' | 'feed.layout3' | 'feed.layoutAuto'
	> = {
		list: 'feed.layoutList',
		'2': 'feed.layout2',
		'3': 'feed.layout3',
		auto: 'feed.layoutAuto'
	};

	let opened = $derived(feedDisplayState.popoverAnchor !== null);

	let isCustomized = $derived(
		feedDisplayState.hasNonDefaultSort ||
			feedDisplayState.layout !== 'list' ||
			feedDisplayState.fullWidth
	);

	let activeLayoutLabel = $derived(i18n.t(LAYOUT_LABELS[feedDisplayState.layout]));

	function handleToggle(event: MouseEvent): void {
		event.stopPropagation();
		if (!triggerEl) return;
		feedDisplayState.togglePopover(
			triggerEl.getBoundingClientRect(),
			sidebarMode ? 'right' : 'below'
		);
	}
</script>

{#if sidebarMode}
	<button
		bind:this={triggerEl}
		type="button"
		aria-haspopup="dialog"
		aria-expanded={opened}
		aria-label={i18n.t('feed.displayOptions')}
		class="xm-macos-nav-item justify-between border {opened || isCustomized
			? 'border-[var(--xm-accent-border)] bg-[var(--xm-accent-soft)] xm-text-accent'
			: 'border-[var(--xm-divider)] bg-[var(--xm-surface-subtle)]'}"
		onclick={handleToggle}
	>
		<span class="flex min-w-0 items-center gap-2">
			<SlidersHorizontal class="h-3.5 w-3.5 shrink-0" aria-hidden="true" />
			<span class="truncate">{i18n.t('feed.displayOptions')}</span>
		</span>
		<span class="flex shrink-0 items-center gap-1 text-[11px] opacity-80">
			<span>{activeLayoutLabel}</span>
			<ChevronRight class="h-3.5 w-3.5" aria-hidden="true" />
		</span>
	</button>
{:else}
	<button
		bind:this={triggerEl}
		type="button"
		aria-haspopup="dialog"
		aria-expanded={opened}
		aria-label={i18n.t('feed.displayOptions')}
		class="inline-flex h-8 items-center gap-1.5 rounded-full border px-3 text-xs font-semibold transition-colors cursor-pointer {opened ||
		isCustomized
			? 'border-[var(--xm-accent-border)] bg-[var(--xm-accent-soft)] xm-text-accent'
			: 'border-[var(--xm-divider)] bg-[var(--xm-surface-subtle)] xm-text-secondary hover:bg-[var(--xm-surface-hover)] hover:text-[var(--xm-text-primary)]'}"
		onclick={handleToggle}
	>
		<SlidersHorizontal class="h-3.5 w-3.5 shrink-0" aria-hidden="true" />
		<span>{i18n.t('feed.displayOptions')}</span>
	</button>
{/if}
