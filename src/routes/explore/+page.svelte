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
	import { Button, Chip } from 'konsta/svelte';
	import { CornerDownLeft, Search, X } from 'lucide-svelte';

	import { memoApi } from '#lib/api/memo.ts';
	import type { MemoView } from '#lib/types/memo.ts';
	import { i18n, type TranslationKey } from '#lib/i18n/index.svelte.ts';
	import { tick } from 'svelte';
	import { authState } from '#lib/state/auth.svelte.ts';
	import { desktopLayoutState } from '#lib/state/desktop-layout.svelte.ts';
	import { confirmDeleteState } from '#lib/state/confirm-delete.svelte.ts';
	import { feedDisplayState } from '#lib/state/feed-display.svelte.ts';
	import type { QuickFilterType } from '#lib/state/filters.svelte.ts';
	import { memosState } from '#lib/state/memos.svelte.ts';
	import { shortcutsState } from '#lib/state/shortcuts.svelte.ts';
	import { toastState } from '#lib/state/toast.svelte.ts';
	import { getLocalDayBounds } from '#lib/utils/date.ts';
	import AppShell from '#lib/components/layout/AppShell.svelte';
	import MemoCard from '#lib/components/memo/MemoCard.svelte';
	import MemoEditor from '#lib/components/memo/MemoEditor.svelte';
	import MemoMasonryGrid from '#lib/components/memo/MemoMasonryGrid.svelte';
	import InfiniteScrollSentinel from '#lib/components/ui/InfiniteScrollSentinel.svelte';
	import KonstaSkeleton from '#lib/components/ui/KonstaSkeleton.svelte';
	import TimelineDisplayMenu from '#lib/components/ui/TimelineDisplayMenu.svelte';

	const PAGE_SIZE = 20;

	const QUICK_FILTERS: { id: QuickFilterType; labelKey: TranslationKey }[] = [
		{ id: 'ALL', labelKey: 'home.filter.all' },
		{ id: 'TASKS', labelKey: 'home.filter.tasks' },
		{ id: 'MEDIA', labelKey: 'home.filter.media' },
		{ id: 'LINKS', labelKey: 'home.filter.links' },
		{ id: 'CODE', labelKey: 'home.filter.code' }
	];

	let items = $state<MemoView[]>([]);
	let loading = $state(true);
	let loadingMore = $state(false);
	let hasMore = $state(true);
	let offset = $state(0);
	let editorOpened = $state(false);
	let editingMemo = $state<MemoView | null>(null);
	let searchInputEl = $state<HTMLInputElement | null>(null);
	let desktopSearchInputEl = $state<HTMLInputElement | null>(null);

	// Independent Explore search & filter state (100% SQLite-enforced PUBLIC/PROTECTED scope)
	let query = $state('');
	let searchTerms = $state<string[]>([]);
	let quickFilter = $state<QuickFilterType>('ALL');
	let selectedTag = $state<string | null>(null);
	let selectedDate = $state<string | null>(null);
	let searchDebounce: ReturnType<typeof setTimeout> | null = null;
	let requestSeq = 0;

	let activeHighlightTerms = $derived.by((): string[] => {
		const list = [...searchTerms];
		const live = query.trim();
		if (live && !list.some((t) => t.toLowerCase() === live.toLowerCase())) {
			list.push(live);
		}
		return list;
	});

	let effectiveQuery = $derived(activeHighlightTerms.join(' '));

	let hasActiveFilters = $derived(
		Boolean(
			query.trim() ||
				searchTerms.length > 0 ||
				quickFilter !== 'ALL' ||
				selectedTag ||
				selectedDate
		)
	);

	function commitSearchTerm(): void {
		const clean = query.trim();
		if (!clean) return;
		if (searchDebounce) clearTimeout(searchDebounce);
		if (!searchTerms.some((t) => t.toLowerCase() === clean.toLowerCase())) {
			searchTerms = [...searchTerms, clean];
		}
		query = '';
		void loadExplore(true);
	}

	function removeSearchTerm(term: string): void {
		searchTerms = searchTerms.filter((t) => t !== term);
	}

	function resetFilters(): void {
		query = '';
		searchTerms = [];
		quickFilter = 'ALL';
		selectedTag = null;
		selectedDate = null;
		if (searchDebounce) clearTimeout(searchDebounce);
	}

	function handleSearchInput(e: Event): void {
		query = (e.currentTarget as HTMLInputElement).value;
		if (searchDebounce) clearTimeout(searchDebounce);
		searchDebounce = setTimeout(() => {
			void loadExplore(true);
		}, 120);
	}

	function handleSearchKeydown(e: KeyboardEvent): void {
		if (e.key === 'Enter') {
			e.preventDefault();
			commitSearchTerm();
		} else if (e.key === 'Escape' && query.length > 0) {
			e.preventDefault();
			clearSearchOnly();
		}
	}

	function clearSearchOnly(): void {
		query = '';
		if (searchDebounce) clearTimeout(searchDebounce);
		void loadExplore(true);
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

	function replaceExploreMemo(updated: MemoView): void {
		const allowedVis = authState.isAuthenticated ? ['PUBLIC', 'PROTECTED'] : ['PUBLIC'];
		if (updated.memo.rowStatus !== 'NORMAL' || !allowedVis.includes(updated.memo.visibility)) {
			items = items.filter((m) => m.memo.id !== updated.memo.id);
		} else if (items.some((m) => m.memo.id === updated.memo.id)) {
			items = items.map((m) => (m.memo.id === updated.memo.id ? updated : m));
		} else {
			items = [updated, ...items];
		}
		memosState.replaceItem(updated);
	}

	async function loadExplore(reset = true): Promise<void> {
		if (!reset && (loading || loadingMore || !hasMore)) return;
		const seq = ++requestSeq;
		if (reset) {
			loading = true;
		} else {
			loadingMore = true;
		}
		try {
			const nextOffset = reset ? 0 : offset;
			const { startTs, endTs } = selectedDate
				? getLocalDayBounds(selectedDate)
				: { startTs: null, endTs: null };

			const batch = await memoApi.list({
				rowStatus: 'NORMAL',
				visibilityList: authState.isAuthenticated ? ['PUBLIC', 'PROTECTED'] : ['PUBLIC'],
				tag: selectedTag,
				query: effectiveQuery || null,
				hasTaskList: quickFilter === 'TASKS' ? true : undefined,
				hasMedia: quickFilter === 'MEDIA' ? true : undefined,
				hasLink: quickFilter === 'LINKS' ? true : undefined,
				hasCode: quickFilter === 'CODE' ? true : undefined,
				startTs,
				endTs,
				orderBy: feedDisplayState.orderBy,
				orderDirection: feedDisplayState.orderDirection,
				limit: PAGE_SIZE,
				offset: nextOffset
			});
			if (seq !== requestSeq) return;
			if (reset) {
				items = batch;
			} else {
				const seen = new Set(items.map((m) => m.memo.id));
				items = [...items, ...batch.filter((m) => !seen.has(m.memo.id))];
			}
			offset = nextOffset + batch.length;
			hasMore = batch.length === PAGE_SIZE;
		} finally {
			if (seq === requestSeq) {
				loading = false;
				loadingMore = false;
			}
		}
	}

	function handleEdit(view: MemoView): void {
		editingMemo = view;
		editorOpened = true;
	}

	async function handlePin(view: MemoView): Promise<void> {
		const updated = await memoApi.update({
			id: view.memo.id,
			pinned: !view.memo.pinned
		});
		replaceExploreMemo(updated);
	}

	async function handleArchive(view: MemoView): Promise<void> {
		await memoApi.update({ id: view.memo.id, rowStatus: 'ARCHIVED' });
		items = items.filter((m) => m.memo.id !== view.memo.id);
		memosState.items = memosState.items.filter((m) => m.memo.id !== view.memo.id);
		void memosState.loadSidebarMetadata(authState.currentUser?.id ?? null);
		toastState.show(i18n.t('home.memoArchived'), 'info');
	}

	function handleDelete(view: MemoView): void {
		void confirmDeleteState.run({
			title: i18n.t('home.deleteMemoTitle'),
			message: i18n.t('home.deleteMemoMsg'),
			confirmLabel: i18n.t('common.delete'),
			onConfirm: async () => {
				await memoApi.delete(view.memo.id);
				items = items.filter((m) => m.memo.id !== view.memo.id);
				memosState.items = memosState.items.filter((m) => m.memo.id !== view.memo.id);
				void memosState.loadSidebarMetadata(authState.currentUser?.id ?? null);
				toastState.show(i18n.t('home.memoDeleted'), 'info');
			}
		});
	}

	$effect(() => {
		const _auth = authState.currentUser?.id;
		const _qf = quickFilter;
		const _tag = selectedTag;
		const _date = selectedDate;
		const _terms = searchTerms.join('\u0000');
		const _orderBy = feedDisplayState.orderBy;
		const _orderDir = feedDisplayState.orderDirection;
		void [_auth, _qf, _tag, _date, _terms, _orderBy, _orderDir];
		void loadExplore(true);
	});
</script>


<AppShell
	title={i18n.t('explore.title')}
	onCompose={() => {
		editingMemo = null;
		editorOpened = true;
	}}
	onSelectTag={(t) => {
		selectedTag = t;
	}}
	onSelectDate={(d) => {
		selectedDate = d;
	}}
>
	{#snippet desktopLeftSearch()}
		<div
			class="flex h-9 w-full items-center gap-1.5 rounded-xl border border-[var(--xm-divider)] bg-[var(--xm-input-bg)] px-2.5 transition-shadow focus-within:border-[var(--xm-accent)] focus-within:ring-2 focus-within:ring-[var(--xm-accent-soft)]"
		>
			<Search class="h-3.5 w-3.5 shrink-0 xm-text-muted" aria-hidden="true" />
			<label for="desktop-explore-search-input" class="sr-only">{i18n.t('home.searchLabel')}</label>
			<input
				bind:this={desktopSearchInputEl}
				id="desktop-explore-search-input"
				type="text"
				inputmode="search"
				enterkeyhint="search"
				role="searchbox"
				autocomplete="off"
				value={query}
				oninput={handleSearchInput}
				onkeydown={handleSearchKeydown}
				placeholder={i18n.t('home.searchPlaceholder')}
				class="xm-search-input min-w-0 flex-1 bg-transparent text-xs xm-text-primary placeholder:text-[var(--xm-text-muted)]"
			/>
			{#if query.trim().length > 0}
				<button
					type="button"
					title={i18n.t('search.applyEnter')}
					aria-label={i18n.t('search.applyEnter')}
					class="flex h-5 w-5 shrink-0 items-center justify-center rounded-full bg-[var(--xm-accent-soft)] xm-text-accent transition-opacity hover:opacity-90"
					onclick={commitSearchTerm}
				>
					<CornerDownLeft class="h-3 w-3" aria-hidden="true" />
				</button>
			{/if}
			{#if query.length > 0}
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

			<div class="flex flex-wrap items-center gap-1.5 px-1" role="group" aria-label="Quick explore filters">
				{#each QUICK_FILTERS as f (f.id)}
					<button
						type="button"
						aria-pressed={quickFilter === f.id}
						class="h-7 rounded-full border px-2.5 text-[11px] font-semibold transition-colors cursor-pointer {quickFilter === f.id
							? 'border-[var(--xm-accent-border)] bg-[var(--xm-accent-soft)] xm-text-accent'
							: 'border-[var(--xm-divider)] bg-transparent xm-text-secondary hover:bg-[var(--xm-macos-nav-hover)] hover:text-[var(--xm-text-primary)]'}"
						onclick={() => (quickFilter = f.id)}
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
				<label for="explore-search-input" class="sr-only">{i18n.t('home.searchLabel')}</label>
				<input
					bind:this={searchInputEl}
					id="explore-search-input"
					type="text"
					inputmode="search"
					enterkeyhint="search"
					role="searchbox"
					autocomplete="off"
					value={query}
					oninput={handleSearchInput}
					onkeydown={handleSearchKeydown}
					placeholder={i18n.t('home.searchPlaceholder')}
					class="xm-search-input min-w-0 flex-1 bg-transparent text-sm xm-text-primary placeholder:text-[var(--xm-text-muted)]"
				/>
				{#if query.trim().length > 0}
					<button
						type="button"
						title={i18n.t('search.applyEnter')}
						aria-label={i18n.t('search.applyEnter')}
						class="flex h-7 w-7 shrink-0 items-center justify-center rounded-full bg-[var(--xm-accent-soft)] xm-text-accent transition-opacity hover:opacity-90 active:scale-95"
						onclick={commitSearchTerm}
					>
						<CornerDownLeft class="h-3.5 w-3.5" aria-hidden="true" />
					</button>
				{/if}
				{#if query.length > 0}
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
		<div class="flex flex-wrap items-center gap-1.5 lg:hidden" role="group" aria-label="Explore memo filters">
			{#each QUICK_FILTERS as f (f.id)}
				<Chip
					outline={quickFilter !== f.id}
					role="button"
					aria-pressed={quickFilter === f.id}
					class="cursor-pointer !text-xs"
					onclick={() => (quickFilter = f.id)}
				>
					{i18n.t(f.labelKey)}
				</Chip>
			{/each}

			{#each searchTerms as term (term)}
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
						onclick={() => removeSearchTerm(term)}
					>
						<X class="h-3 w-3" aria-hidden="true" />
					</button>
				</span>
			{/each}

			{#if selectedTag}
				<Chip
					deleteButton
					class="!text-xs"
					onDelete={() => (selectedTag = null)}
				>
					#{selectedTag}
				</Chip>
			{/if}
			{#if selectedDate}
				<Chip
					deleteButton
					class="!text-xs"
					onDelete={() => (selectedDate = null)}
				>
					{selectedDate}
				</Chip>
			{/if}
		</div>

		<!-- Desktop (>= 1024px): Active Search / Tag / Date Result Pills at top of Memos feed (Official Memos style) -->
		{#if searchTerms.length > 0 || selectedTag || selectedDate}
			<div class="hidden flex-wrap items-center gap-1.5 lg:flex" role="group" aria-label="Active explore filters">
				{#each searchTerms as term (term)}
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
							onclick={() => removeSearchTerm(term)}
						>
							<X class="h-3 w-3" aria-hidden="true" />
						</button>
					</span>
				{/each}
				{#if selectedTag}
					<Chip deleteButton class="!text-xs" onDelete={() => (selectedTag = null)}>
						#{selectedTag}
					</Chip>
				{/if}
				{#if selectedDate}
					<Chip deleteButton class="!text-xs" onDelete={() => (selectedDate = null)}>
						{selectedDate}
					</Chip>
				{/if}
			</div>
		{/if}

		{#if loading && items.length === 0}
			<KonstaSkeleton variant="memo" count={3} label={i18n.t('common.loading')} />
		{:else if items.length === 0}
			<div class="glass-surface space-y-3 rounded-3xl p-8 text-center">
				<p class="font-semibold xm-text-primary">
					{i18n.t(hasActiveFilters ? 'home.emptyFilteredTitle' : 'explore.emptyTitle')}
				</p>
				<p class="text-xs leading-relaxed xm-text-secondary">
					{i18n.t(hasActiveFilters ? 'home.emptyFilteredDesc' : 'explore.emptyDesc')}
				</p>
				{#if hasActiveFilters}
					<div class="pt-1">
						<Button
							rounded
							tonal
							inline
							onclick={() => {
								resetFilters();
								void loadExplore(true);
							}}
						>
							{i18n.t('home.clearFilters')}
						</Button>
					</div>
				{/if}
			</div>
		{:else}
			<MemoMasonryGrid {items} layout={feedDisplayState.layout}>
				{#snippet card(view)}
					<MemoCard
						{view}
						highlightTerms={activeHighlightTerms}
						onEdit={handleEdit}
						onPin={handlePin}
						onArchive={handleArchive}
						onDelete={handleDelete}
						onToggleTask={async (id, idx) => {
							const updated = await memoApi.toggleTask(id, idx);
							replaceExploreMemo(updated);
						}}
						onToggleReaction={async (v, emoji) => {
							const reactions = await memoApi.toggleReaction(v.memo.uid, emoji);
							replaceExploreMemo({ ...v, reactions });
						}}
						onSelectTag={(tag) => {
							selectedTag = selectedTag === tag ? null : tag;
						}}
					/>
				{/snippet}
			</MemoMasonryGrid>

			<InfiniteScrollSentinel
				{hasMore}
				loading={loadingMore}
				itemCount={items.length}
				onLoadMore={() => loadExplore(false)}
			/>
		{/if}
	</div>
</AppShell>

{#if editorOpened}
	<MemoEditor
		opened={editorOpened}
		{editingMemo}
		onSaved={(saved) => {
			replaceExploreMemo(saved);
			void memosState.loadSidebarMetadata(authState.currentUser?.id ?? null);
		}}
		onClose={() => {
			editorOpened = false;
			editingMemo = null;
		}}
	/>
{/if}
