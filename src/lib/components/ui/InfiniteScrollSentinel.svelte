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
	import { ArrowUp } from 'lucide-svelte';

	import { i18n } from '#lib/i18n/index.svelte.ts';
	import KonstaSkeleton from './KonstaSkeleton.svelte';

	interface Props {
		hasMore: boolean;
		loading: boolean;
		itemCount: number;
		variant?: 'memo' | 'media';
		rootMargin?: string;
		onLoadMore: () => void | Promise<void>;
	}

	let {
		hasMore,
		loading,
		itemCount,
		variant = 'memo',
		rootMargin = '320px 0px',
		onLoadMore
	}: Props = $props();

	let sentinelEl = $state<HTMLDivElement | null>(null);
	let showBackToTop = $state(false);

	function getScrollRoot(): HTMLElement | null {
		if (typeof document === 'undefined') return null;
		const mainEl = document.getElementById('main-content');
		if (mainEl) {
			const style = window.getComputedStyle(mainEl);
			if (style.overflowY === 'auto' || style.overflowY === 'scroll') {
				return mainEl;
			}
		}
		return null;
	}

	function evaluateBackToTop(): void {
		if (typeof window === 'undefined' || typeof document === 'undefined') {
			showBackToTop = false;
			return;
		}
		const mainEl = getScrollRoot();
		if (mainEl) {
			const hasOverflow = mainEl.scrollHeight > mainEl.clientHeight + 120;
			showBackToTop = hasOverflow && mainEl.scrollTop > 120;
			return;
		}
		const pageEl = document.querySelector<HTMLElement>('.k-page');
		if (pageEl && pageEl.scrollHeight > pageEl.clientHeight + 120) {
			showBackToTop = pageEl.scrollTop > 120;
			return;
		}
		const docEl = document.documentElement;
		const hasDocOverflow = docEl.scrollHeight > window.innerHeight + 120;
		showBackToTop = hasDocOverflow && window.scrollY > 120;
	}

	function scrollToTop(): void {
		if (typeof window === 'undefined') return;
		const mainEl = document.getElementById('main-content');
		if (mainEl && typeof mainEl.scrollTo === 'function') {
			mainEl.scrollTo({ top: 0, behavior: 'smooth' });
		}
		window.scrollTo({ top: 0, behavior: 'smooth' });
		const pageEl = document.querySelector('.k-page');
		if (pageEl && typeof pageEl.scrollTo === 'function') {
			pageEl.scrollTo({ top: 0, behavior: 'smooth' });
		}
	}

	// Infinite scroll observer
	$effect(() => {
		const el = sentinelEl;
		if (!el || !hasMore || typeof IntersectionObserver === 'undefined') return;

		const observer = new IntersectionObserver(
			(entries) => {
				if (entries.some((e) => e.isIntersecting) && hasMore && !loading) {
					void onLoadMore();
				}
			},
			{ root: getScrollRoot(), rootMargin, threshold: 0.01 }
		);

		observer.observe(el);
		return () => observer.disconnect();
	});

	// Smart dynamic Back-to-Top visibility: only appears when feed actually overflows AND user scrolled down
	$effect(() => {
		const _count = itemCount;
		const _more = hasMore;
		void [_count, _more];
		if (typeof window === 'undefined' || typeof document === 'undefined') return;

		const mainEl = document.getElementById('main-content');
		const pageEl = document.querySelector<HTMLElement>('.k-page');
		const onScrollOrResize = () => evaluateBackToTop();

		const rafId = requestAnimationFrame(evaluateBackToTop);
		window.addEventListener('scroll', onScrollOrResize, { passive: true });
		window.addEventListener('resize', onScrollOrResize, { passive: true });
		mainEl?.addEventListener('scroll', onScrollOrResize, { passive: true });
		pageEl?.addEventListener('scroll', onScrollOrResize, { passive: true });

		return () => {
			cancelAnimationFrame(rafId);
			window.removeEventListener('scroll', onScrollOrResize);
			window.removeEventListener('resize', onScrollOrResize);
			mainEl?.removeEventListener('scroll', onScrollOrResize);
			pageEl?.removeEventListener('scroll', onScrollOrResize);
		};
	});
</script>

{#if hasMore}
	<div bind:this={sentinelEl} class="pt-2" aria-live="polite">
		{#if loading}
			<KonstaSkeleton {variant} count={variant === 'media' ? 2 : 1} label={i18n.t('common.loading')} />
		{:else}
			<div class="h-4 w-full" aria-hidden="true"></div>
		{/if}
	</div>
{:else if itemCount > 0 && showBackToTop}
	<div class="flex w-full justify-center py-5">
		<button
			type="button"
			class="inline-flex items-center gap-1.5 rounded-full px-3.5 py-1.5 text-xs font-medium xm-text-muted transition-colors hover:bg-[var(--xm-surface-subtle)] hover:text-[var(--xm-text-primary)] active:scale-95"
			onclick={scrollToTop}
		>
			<span>{i18n.t('common.backToTop')}</span>
			<ArrowUp class="h-3.5 w-3.5" aria-hidden="true" />
		</button>
	</div>
{/if}
