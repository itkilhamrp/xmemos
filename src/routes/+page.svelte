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
	import { goto } from '$app/navigation';
	import { Button, Chip } from 'konsta/svelte';
	import { CornerDownLeft, Search, X } from 'lucide-svelte';

	import type { MemoView } from '#lib/types/memo.ts';
	import { i18n, type TranslationKey } from '#lib/i18n/index.svelte.ts';
	import { tick, untrack } from 'svelte';
	import { authState } from '#lib/state/auth.svelte.ts';
	import { desktopLayoutState } from '#lib/state/desktop-layout.svelte.ts';
	import { confirmDeleteState } from '#lib/state/confirm-delete.svelte.ts';
	import { feedDisplayState } from '#lib/state/feed-display.svelte.ts';
	import { filterState, type QuickFilterType } from '#lib/state/filters.svelte.ts';
	import { memosState } from '#lib/state/memos.svelte.ts';
	import { shortcutsState } from '#lib/state/shortcuts.svelte.ts';
	import { toastState } from '#lib/state/toast.svelte.ts';
	import AppShell from '#lib/components/layout/AppShell.svelte';
	import MemoCard from '#lib/components/memo/MemoCard.svelte';
	import MemoEditor from '#lib/components/memo/MemoEditor.svelte';
	import MemoMasonryGrid from '#lib/components/memo/MemoMasonryGrid.svelte';
	import InfiniteScrollSentinel from '#lib/components/ui/InfiniteScrollSentinel.svelte';
	import KonstaSkeleton from '#lib/components/ui/KonstaSkeleton.svelte';
	import TimelineDisplayMenu from '#lib/components/ui/TimelineDisplayMenu.svelte';

	let editorOpened = $state(false);
	let editingMemo = $state<MemoView | null>(null);
	let searchInputEl = $state<HTMLInputElement | null>(null);
	let desktopSearchInputEl = $state<HTMLInputElement | null>(null);
	let searchDebounce: ReturnType<typeof setTimeout> | null = null;

	const QUICK_FILTERS: { id: QuickFilterType; labelKey: TranslationKey }[] = [
		{ id: 'ALL', labelKey: 'home.filter.all' },
		{ id: 'TASKS', labelKey: 'home.filter.tasks' },
		{ id: 'MEDIA', labelKey: 'home.filter.media' },
		{ id: 'LINKS', labelKey: 'home.filter.links' },
		{ id: 'CODE', labelKey: 'home.filter.code' }
	];

	async function reload(reset = true): Promise<void> {
		const uid = authState.currentUser?.id ?? null;
		if (reset) {
			const isMobileViewport = typeof window !== 'undefined' && window.innerWidth < 1024;
			if (isMobileViewport) {
				// On Android/Mobile Frame 0, prioritize timeline SQLite query with zero sidebar contention
				await memosState.loadTimeline(uid, true);
				setTimeout(() => {
					void memosState.loadSidebarMetadata(uid);
				}, 120);
			} else {
				await Promise.all([
					memosState.loadTimeline(uid, true),
					memosState.loadSidebarMetadata(uid)
				]);
			}
		} else {
			await memosState.loadTimeline(uid, false);
		}
	}

	$effect(() => {
		if (!authState.initialized) return;
		if (!authState.hasHost) {
			untrack(() => {
				void goto('/auth', { replace: true });
			});
			return;
		}
		const uid = authState.currentUser?.id ?? null;
		const hasActiveFilters = filterState.hasActiveFilters;
		const _tag = filterState.selectedTag;
		const _date = filterState.selectedDate;
		const _qf = filterState.quickFilter;
		const _vis = filterState.visibility;
		const _terms = filterState.searchTerms.join('\u0000');
		const _orderBy = feedDisplayState.orderBy;
		const _orderDir = feedDisplayState.orderDirection;
		void [uid, hasActiveFilters, _tag, _date, _qf, _vis, _terms, _orderBy, _orderDir];
		untrack(() => {
			if (!hasActiveFilters) {
				memosState.hydrateSnapshot(uid);
			}
			void reload(true);
		});
	});

	function handleSearchInput(e: Event): void {
		filterState.query = (e.currentTarget as HTMLInputElement).value;
		if (searchDebounce) clearTimeout(searchDebounce);
		searchDebounce = setTimeout(() => {
			void reload(true);
		}, 120);
	}

	function handleSearchSubmit(): void {
		if (searchDebounce) clearTimeout(searchDebounce);
		if (filterState.commitQuery()) {
			void reload(true);
		}
	}

	function handleSearchKeydown(e: KeyboardEvent): void {
		if (e.key === 'Enter') {
			e.preventDefault();
			handleSearchSubmit();
		} else if (e.key === 'Escape' && filterState.query.length > 0) {
			e.preventDefault();
			clearSearchOnly();
		}
	}

	function clearSearchOnly(): void {
		filterState.query = '';
		if (searchDebounce) clearTimeout(searchDebounce);
		void reload(true);
	}

	function focusSearchInput(): void {
		if (typeof window !== 'undefined' && window.innerWidth >= 1024) {
			if (!desktopLayoutState.sidebarOpen) {
				desktopLayoutState.setSidebarOpen(true);
			}
			void tick().then(() => {
				desktopSearchInputEl?.focus();
				desktopSearchInputEl?.select();
			});
		} else {
			searchInputEl?.focus();
			searchInputEl?.select();
		}
	}

	$effect(() => {
		return shortcutsState.registerFocusSearch(focusSearchInput);
	});
</script>

<AppShell
	title="Xmemos"
	onCompose={() => {
		editingMemo = null;
		editorOpened = true;
	}}
	onSelectTag={(t) => {
		filterState.selectedTag = t;
	}}
	onSelectDate={(d) => {
		filterState.selectedDate = d;
	}}
>
	{#snippet desktopLeftSearch()}
		<div
			class="flex h-9 w-full items-center gap-1.5 rounded-xl border border-[var(--xm-divider)] bg-[var(--xm-input-bg)] px-2.5 transition-shadow focus-within:border-[var(--xm-accent)] focus-within:ring-2 focus-within:ring-[var(--xm-accent-soft)]"
		>
			<Search class="h-3.5 w-3.5 shrink-0 xm-text-muted" aria-hidden="true" />
			<label for="desktop-memo-search-input" class="sr-only">{i18n.t('home.searchLabel')}</label>
			<input
				bind:this={desktopSearchInputEl}
				id="desktop-memo-search-input"
				type="text"
				inputmode="search"
				enterkeyhint="search"
				role="searchbox"
				autocomplete="off"
				value={filterState.query}
				oninput={handleSearchInput}
				onkeydown={handleSearchKeydown}
				placeholder={i18n.t('home.searchPlaceholder')}
				class="xm-search-input min-w-0 flex-1 bg-transparent text-xs xm-text-primary placeholder:text-[var(--xm-text-muted)]"
			/>
			{#if filterState.query.trim().length > 0}
				<button
					type="button"
					title={i18n.t('search.applyEnter')}
					aria-label={i18n.t('search.applyEnter')}
					class="flex h-5 w-5 shrink-0 items-center justify-center rounded-full bg-[var(--xm-accent-soft)] xm-text-accent transition-opacity hover:opacity-90"
					onclick={handleSearchSubmit}
				>
					<CornerDownLeft class="h-3 w-3" aria-hidden="true" />
				</button>
			{/if}
			{#if filterState.query.length > 0}
				<button
					type="button"
					aria-label={i18n.t('home.clearSearch')}
					class="flex h-5 w-5 shrink-0 items-center justify-center rounded-full bg-[var(--xm-surface-hover)] xm-text-secondary transition-colors hover:bg-[var(--xm-accent-soft)] hover:text-[var(--xm-accent)]"
					onclick={clearSearchOnly}
				>
					<X class="h-3 w-3" aria-hidden="true" />
				</button>
			{/if}
		</div>
	{/snippet}

	{#snippet desktopLeftControls()}
		<div class="space-y-2.5">
			<div
				class="px-2.5 text-[11px] font-semibold tracking-[0.04em] uppercase xm-text-muted select-none"
			>
				{i18n.t('nav.sectionFilters')}
			</div>

			<div class="flex flex-wrap items-center gap-1.5 px-1" role="group" aria-label="Quick memo filters">
				{#each QUICK_FILTERS as f (f.id)}
					<button
						type="button"
						aria-pressed={filterState.quickFilter === f.id}
						class="h-7 rounded-full border px-2.5 text-[11px] font-semibold transition-colors cursor-pointer {filterState.quickFilter === f.id
							? 'border-[var(--xm-accent-border)] bg-[var(--xm-accent-soft)] xm-text-accent'
							: 'border-[var(--xm-divider)] bg-transparent xm-text-secondary hover:bg-[var(--xm-macos-nav-hover)] hover:text-[var(--xm-text-primary)]'}"
						onclick={() => (filterState.quickFilter = f.id)}
					>
						{i18n.t(f.labelKey)}
					</button>
				{/each}
			</div>

			<TimelineDisplayMenu sidebarMode />
		</div>
	{/snippet}

	<div class="space-y-3">
		<!-- Mobile & Tablet/iPad (< 1024px): Inline Search Bar + Display Controls (moved to Left Rail on Desktop) -->
		<div class="flex flex-wrap items-center gap-2 lg:hidden">
			<div
				class="glass-surface flex h-11 min-w-0 flex-1 items-center gap-2 rounded-full px-3.5 transition-shadow focus-within:border-[var(--xm-accent)] focus-within:ring-2 focus-within:ring-[var(--xm-accent-soft)] sm:max-w-[540px]"
			>
				<Search class="h-4 w-4 shrink-0 xm-text-muted" aria-hidden="true" />
				<label for="memo-search-input" class="sr-only">{i18n.t('home.searchLabel')}</label>
				<input
					bind:this={searchInputEl}
					id="memo-search-input"
					type="text"
					inputmode="search"
					enterkeyhint="search"
					role="searchbox"
					autocomplete="off"
					value={filterState.query}
					oninput={handleSearchInput}
					onkeydown={handleSearchKeydown}
					placeholder={i18n.t('home.searchPlaceholder')}
					class="xm-search-input min-w-0 flex-1 bg-transparent text-sm xm-text-primary placeholder:text-[var(--xm-text-muted)]"
				/>
				{#if filterState.query.trim().length > 0}
					<button
						type="button"
						title={i18n.t('search.applyEnter')}
						aria-label={i18n.t('search.applyEnter')}
						class="flex h-7 w-7 shrink-0 items-center justify-center rounded-full bg-[var(--xm-accent-soft)] xm-text-accent transition-opacity hover:opacity-90 active:scale-95"
						onclick={handleSearchSubmit}
					>
						<CornerDownLeft class="h-3.5 w-3.5" aria-hidden="true" />
					</button>
				{/if}
				{#if filterState.query.length > 0}
					<button
						type="button"
						aria-label={i18n.t('home.clearSearch')}
						class="flex h-7 w-7 shrink-0 items-center justify-center rounded-full bg-[var(--xm-surface-hover)] xm-text-secondary transition-colors hover:bg-[var(--xm-accent-soft)] hover:text-[var(--xm-accent)]"
						onclick={clearSearchOnly}
					>
						<X class="h-3.5 w-3.5" aria-hidden="true" />
					</button>
				{/if}
			</div>

			<div class="ml-auto shrink-0">
				<TimelineDisplayMenu />
			</div>
		</div>

		<!-- Mobile & Tablet/iPad (< 1024px): Quick Filter Chips + Active Search Pills -->
		<div class="flex flex-wrap items-center gap-1.5 lg:hidden" role="group" aria-label="Quick memo filters">
			{#each QUICK_FILTERS as f (f.id)}
				<Chip
					outline={filterState.quickFilter !== f.id}
					role="button"
					aria-pressed={filterState.quickFilter === f.id}
					class="cursor-pointer !text-xs"
					onclick={() => (filterState.quickFilter = f.id)}
				>
					{i18n.t(f.labelKey)}
				</Chip>
			{/each}

			{#each filterState.searchTerms as term (term)}
				<span
					class="inline-flex h-7 items-center gap-1.5 rounded-full border border-[var(--xm-accent-border)] bg-[var(--xm-accent-soft)] pr-1.5 pl-2.5 text-xs font-semibold xm-text-accent"
				>
					<Search class="h-3 w-3 shrink-0" aria-hidden="true" />
					<span class="max-w-[160px] truncate">{term}</span>
					<button
						type="button"
						title={i18n.t('search.removeFilter')}
						aria-label="{i18n.t('search.removeFilter')}: {term}"
						class="flex h-4 w-4 items-center justify-center rounded-full transition-colors hover:bg-[var(--xm-accent)]/20"
						onclick={() => filterState.removeSearchTerm(term)}
					>
						<X class="h-3 w-3" aria-hidden="true" />
					</button>
				</span>
			{/each}

			{#if filterState.selectedTag}
				<Chip
					deleteButton
					class="!text-xs"
					onDelete={() => (filterState.selectedTag = null)}
				>
					#{filterState.selectedTag}
				</Chip>
			{/if}
			{#if filterState.selectedDate}
				<Chip
					deleteButton
					class="!text-xs"
					onDelete={() => (filterState.selectedDate = null)}
				>
					{filterState.selectedDate}
				</Chip>
			{/if}
		</div>

		<!-- Desktop (>= 1024px): Active Search / Tag / Date Result Pills at top of Memos feed (Official Memos style) -->
		{#if filterState.searchTerms.length > 0 || filterState.selectedTag || filterState.selectedDate}
			<div class="hidden flex-wrap items-center gap-1.5 lg:flex" role="group" aria-label="Active filters">
				{#each filterState.searchTerms as term (term)}
					<span
						class="inline-flex h-7 items-center gap-1.5 rounded-full border border-[var(--xm-accent-border)] bg-[var(--xm-accent-soft)] pr-1.5 pl-2.5 text-xs font-semibold xm-text-accent"
					>
						<Search class="h-3 w-3 shrink-0" aria-hidden="true" />
						<span class="max-w-[200px] truncate">{term}</span>
						<button
							type="button"
							title={i18n.t('search.removeFilter')}
							aria-label="{i18n.t('search.removeFilter')}: {term}"
							class="flex h-4 w-4 items-center justify-center rounded-full transition-colors hover:bg-[var(--xm-accent)]/20 cursor-pointer"
							onclick={() => filterState.removeSearchTerm(term)}
						>
							<X class="h-3 w-3" aria-hidden="true" />
						</button>
					</span>
				{/each}
				{#if filterState.selectedTag}
					<Chip deleteButton class="!text-xs" onDelete={() => (filterState.selectedTag = null)}>
						#{filterState.selectedTag}
					</Chip>
				{/if}
				{#if filterState.selectedDate}
					<Chip deleteButton class="!text-xs" onDelete={() => (filterState.selectedDate = null)}>
						{filterState.selectedDate}
					</Chip>
				{/if}
			</div>
		{/if}

		<!-- Timeline Feed: Flexbox Masonry Grid (1-col on mobile; clean List/2/3/Auto on tablet & desktop) -->
		{#if memosState.loading && memosState.items.length === 0}
			<KonstaSkeleton variant="memo" count={3} label={i18n.t('common.loading')} />
		{:else if memosState.items.length === 0}
			<div class="glass-surface space-y-3 rounded-3xl p-8 text-center">
				<p class="text-base font-semibold xm-text-primary">
					{i18n.t(filterState.hasActiveFilters ? 'home.emptyFilteredTitle' : 'home.emptyTitle')}
				</p>
				<p class="text-xs leading-relaxed xm-text-secondary">
					{i18n.t(filterState.hasActiveFilters ? 'home.emptyFilteredDesc' : 'home.emptyDesc')}
				</p>
				<div class="pt-1">
					{#if filterState.hasActiveFilters}
						<Button
							rounded
							tonal
							inline
							onclick={() => {
								filterState.reset();
								void reload(true);
							}}
						>
							{i18n.t('home.clearFilters')}
						</Button>
					{:else if authState.isAuthenticated}
						<Button
							rounded
							inline
							onclick={() => {
								editingMemo = null;
								editorOpened = true;
							}}
						>
							{i18n.t('home.createMemo')}
						</Button>
					{/if}
				</div>
			</div>
		{:else}
			<MemoMasonryGrid items={memosState.items} layout={feedDisplayState.layout}>
				{#snippet card(view)}
					<MemoCard
						{view}
						highlightTerms={filterState.activeHighlightTerms}
						onEdit={(v) => {
							editingMemo = v;
							editorOpened = true;
						}}
						onPin={(v) => memosState.togglePin(v)}
						onArchive={async (v) => {
							await memosState.archiveMemo(v.memo.id);
							toastState.show(i18n.t('home.memoArchived'), 'info');
						}}
						onDelete={(v) => {
							void confirmDeleteState.run({
								title: i18n.t('home.deleteMemoTitle'),
								message: i18n.t('home.deleteMemoMsg'),
								confirmLabel: i18n.t('common.delete'),
								onConfirm: async () => {
									await memosState.deleteMemo(v.memo.id);
									toastState.show(i18n.t('home.memoDeleted'), 'info');
								}
							});
						}}
						onToggleTask={(id, idx) => memosState.toggleTask(id, idx)}
						onToggleReaction={(v, emoji) => memosState.toggleReaction(v, emoji)}
						onSelectTag={(t) => (filterState.selectedTag = t)}
					/>
				{/snippet}
			</MemoMasonryGrid>

			<InfiniteScrollSentinel
				hasMore={memosState.hasMore}
				loading={memosState.loadingMore}
				itemCount={memosState.items.length}
				onLoadMore={() => reload(false)}
			/>
		{/if}
	</div>
</AppShell>

{#if editorOpened}
	<MemoEditor
		opened={editorOpened}
		{editingMemo}
		initialDate={editingMemo ? null : filterState.selectedDate}
		onSaved={() => {
			void memosState.loadSidebarMetadata(authState.currentUser?.id ?? null);
		}}
		onClose={() => {
			editorOpened = false;
			editingMemo = null;
		}}
	/>
{/if}
