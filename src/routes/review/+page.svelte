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
	import { tick, untrack } from 'svelte';
	import { goto } from '$app/navigation';
	import { Button, Segmented, SegmentedButton } from 'konsta/svelte';
	import { CalendarDays, Plus, Shuffle, X } from 'lucide-svelte';

	import { memoApi } from '#lib/api/memo.ts';
	import type { MemoView } from '#lib/types/memo.ts';
	import { i18n } from '#lib/i18n/index.svelte.ts';
	import { authState } from '#lib/state/auth.svelte.ts';
	import { confirmDeleteState } from '#lib/state/confirm-delete.svelte.ts';
	import { filterState } from '#lib/state/filters.svelte.ts';
	import { memosState } from '#lib/state/memos.svelte.ts';
	import { tabCacheState } from '#lib/state/tab-cache.svelte.ts';
	import { toastState } from '#lib/state/toast.svelte.ts';
	import { toLocalDateKey } from '#lib/utils/date.ts';
	import AppShell from '#lib/components/layout/AppShell.svelte';
	import FullMonthCalendar from '#lib/components/calendar/FullMonthCalendar.svelte';
	import MemoCard from '#lib/components/memo/MemoCard.svelte';
	import MemoEditor from '#lib/components/memo/MemoEditor.svelte';
	import KonstaSkeleton from '#lib/components/ui/KonstaSkeleton.svelte';

	const initialReviewCache = tabCacheState.getReview(authState.currentUser?.id ?? null);
	let allMemos = $state<MemoView[]>(initialReviewCache ?? []);
	let loading = $state(initialReviewCache === null);
	let activeTab = $state<'calendar' | 'random'>('calendar');
	let selectedDate = $state<string | null>(null);
	let randomPick = $state<MemoView | null>(null);
	let editorOpened = $state(false);
	let editingMemo = $state<MemoView | null>(null);
	let composeDate = $state<string | null>(null);
	let detailPanelEl = $state<HTMLElement | null>(null);
	let loadSeq = 0;

	let dateMemos = $derived.by(() => {
		if (!selectedDate) return [];
		return allMemos.filter((m) => {
			const day = toLocalDateKey(new Date(m.memo.createdTs * 1000));
			return day === selectedDate;
		});
	});

	let selectedDateHeading = $derived.by(() => {
		if (!selectedDate) return '';
		const m = /^(\d{4})-(\d{2})-(\d{2})$/.exec(selectedDate);
		if (!m) return selectedDate;
		const dt = new Date(Number(m[1]), Number(m[2]) - 1, Number(m[3]));
		const formatted = new Intl.DateTimeFormat(i18n.bcp47, {
			weekday: 'long',
			month: 'long',
			day: 'numeric',
			year: dt.getFullYear() === new Date().getFullYear() ? undefined : 'numeric'
		}).format(dt);
		return formatted.charAt(0).toUpperCase() + formatted.slice(1);
	});

	function sortReviewMemos(list: MemoView[]): MemoView[] {
		return [...list].sort((a, b) => {
			if (a.memo.pinned !== b.memo.pinned) return a.memo.pinned ? -1 : 1;
			if (a.memo.createdTs !== b.memo.createdTs) return b.memo.createdTs - a.memo.createdTs;
			return b.memo.id - a.memo.id;
		});
	}

	function pickRandom(): void {
		if (allMemos.length === 0) {
			randomPick = null;
			return;
		}
		const idx = Math.floor(Math.random() * allMemos.length);
		randomPick = allMemos[idx] ?? null;
	}

	function upsertMemo(saved: MemoView): void {
		const exists = allMemos.some((m) => m.memo.id === saved.memo.id);
		allMemos = sortReviewMemos(
			exists
				? allMemos.map((m) => (m.memo.id === saved.memo.id ? saved : m))
				: [saved, ...allMemos]
		);
		if (randomPick?.memo.id === saved.memo.id) {
			randomPick = saved;
		} else if (!randomPick) {
			randomPick = saved;
		}
		memosState.replaceItem(saved);
	}

	function removeMemo(memoId: number): void {
		allMemos = allMemos.filter((m) => m.memo.id !== memoId);
		memosState.items = memosState.items.filter((m) => m.memo.id !== memoId);
		if (randomPick?.memo.id === memoId) {
			pickRandom();
		}
		void memosState.loadSidebarMetadata(authState.currentUser?.id ?? null);
	}

	async function handleSelectDate(date: string | null): Promise<void> {
		selectedDate = date;
		if (date && typeof window !== 'undefined' && window.innerWidth < 1024) {
			await tick();
			detailPanelEl?.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
		}
	}

	function openNewMemoOnDate(dateKey: string): void {
		editingMemo = null;
		composeDate = dateKey;
		editorOpened = true;
	}

	function handleEdit(view: MemoView): void {
		editingMemo = view;
		composeDate = null;
		editorOpened = true;
	}

	async function handlePin(view: MemoView): Promise<void> {
		const updated = await memoApi.update({
			id: view.memo.id,
			pinned: !view.memo.pinned
		});
		upsertMemo(updated);
	}

	async function handleArchive(view: MemoView): Promise<void> {
		await memoApi.update({ id: view.memo.id, rowStatus: 'ARCHIVED' });
		removeMemo(view.memo.id);
		toastState.show(i18n.t('home.memoArchived'), 'info');
	}

	function handleDelete(view: MemoView): void {
		void confirmDeleteState.run({
			title: i18n.t('home.deleteMemoTitle'),
			message: i18n.t('home.deleteMemoMsg'),
			confirmLabel: i18n.t('common.delete'),
			onConfirm: async () => {
				await memoApi.delete(view.memo.id);
				removeMemo(view.memo.id);
				toastState.show(i18n.t('home.memoDeleted'), 'info');
			}
		});
	}

	async function handleToggleReaction(view: MemoView, emoji: string): Promise<void> {
		const reactions = await memoApi.toggleReaction(view.memo.uid, emoji);
		upsertMemo({ ...view, reactions });
	}

	async function handleToggleTask(id: number, idx: number): Promise<void> {
		const updated = await memoApi.toggleTask(id, idx);
		upsertMemo(updated);
	}

	function handleSelectTag(tag: string): void {
		filterState.selectedTag = tag;
		void goto('/');
	}

	$effect(() => {
		const uid = authState.currentUser?.id ?? null;
		untrack(() => {
			const seq = ++loadSeq;
			const cached = tabCacheState.getReview(uid);
			if (cached && allMemos.length === 0) {
				allMemos = cached;
				if (!randomPick) pickRandom();
				loading = false;
			} else if (allMemos.length === 0) {
				loading = true;
			}
			void memoApi
				.list({
					creatorId: uid,
					rowStatus: 'NORMAL',
					limit: 500
				})
				.then((memos) => {
					if (seq !== loadSeq) return;
					allMemos = memos;
					tabCacheState.setReview(uid, memos);
					if (!randomPick) pickRandom();
				})
				.finally(() => {
					if (seq === loadSeq) loading = false;
				});
		});
	});
</script>

<AppShell title={i18n.t('review.title')} showFab={false}>
	<div
		class="flex flex-col gap-3 {activeTab === 'calendar'
			? 'md:h-[calc(100dvh-5.25rem)] md:overflow-hidden'
			: ''}"
	>
		<!-- Konsta iOS Segmented Mode Switcher: full-width on mobile, compact pill on tablet/desktop -->
		<div class="w-full shrink-0 sm:max-w-[260px]">
			<Segmented rounded strong>
				<SegmentedButton
					active={activeTab === 'calendar'}
					onclick={() => (activeTab = 'calendar')}
				>
					<span class="inline-flex items-center gap-1.5">
						<CalendarDays class="h-3.5 w-3.5" aria-hidden="true" />
						{i18n.t('calendar.tabCalendar')}
					</span>
				</SegmentedButton>
				<SegmentedButton
					active={activeTab === 'random'}
					onclick={() => (activeTab = 'random')}
				>
					<span class="inline-flex items-center gap-1.5">
						<Shuffle class="h-3.5 w-3.5" aria-hidden="true" />
						{i18n.t('calendar.tabRandom')}
					</span>
				</SegmentedButton>
			</Segmented>
		</div>

		{#if loading && allMemos.length === 0}
			<KonstaSkeleton variant="memo" count={3} label={i18n.t('common.loading')} />
		{:else if activeTab === 'calendar'}
			<!-- Viewport-Locked Official Memos 2026 Split Layout on Tablet/Desktop (zero outer scrollbar) -->
			<div class="flex flex-col gap-4 md:min-h-0 md:flex-1 lg:flex-row">
				<div class="min-w-0 flex-1 md:min-h-0">
					<FullMonthCalendar
						memos={allMemos}
						{selectedDate}
						onSelectDate={handleSelectDate}
					/>
				</div>

				{#if selectedDate}
					<aside
						bind:this={detailPanelEl}
						aria-label={selectedDateHeading}
						class="glass-surface flex w-full flex-col overflow-hidden rounded-2xl border border-[var(--xm-divider)] lg:h-full lg:w-[356px] lg:shrink-0 xl:w-[392px]"
					>
						<!-- Clean Single-Purpose Header: Date + Count (left), Close X only (right) -->
						<div
							class="flex shrink-0 items-center justify-between gap-3 border-b border-[var(--xm-divider)] px-4 py-3"
						>
							<div class="min-w-0">
								<h2 class="truncate text-sm font-bold xm-text-primary">
									{selectedDateHeading}
								</h2>
								<p class="text-[11px] xm-text-muted tabular-nums">
									{dateMemos.length}
									{dateMemos.length === 1
										? i18n.t('heatmap.memoSingular')
										: i18n.t('heatmap.memoPlural')}
								</p>
							</div>
							<button
								type="button"
								aria-label={i18n.t('calendar.closePanel')}
								class="flex h-8 w-8 shrink-0 items-center justify-center rounded-full xm-text-secondary transition-colors hover:bg-[var(--xm-surface-subtle)] hover:text-[var(--xm-text-primary)]"
								onclick={() => (selectedDate = null)}
							>
								<X class="h-4 w-4" aria-hidden="true" />
							</button>
						</div>

						<!-- Day Detail Panel Body (internal scroll only on desktop) -->
						<div class="min-h-0 flex-1 space-y-3 p-3.5 lg:overflow-y-auto">
							{#if dateMemos.length === 0}
								<div class="space-y-3 py-8 text-center">
									<p class="text-xs xm-text-muted">{i18n.t('calendar.noMemosDay')}</p>
									{#if authState.isAuthenticated}
										<Button
											small
											tonal
											rounded
											inline
											onclick={() => selectedDate && openNewMemoOnDate(selectedDate)}
										>
											<Plus class="mr-1 h-3.5 w-3.5" aria-hidden="true" />
											{i18n.t('review.newMemoOnDate')}
										</Button>
									{/if}
								</div>
							{:else}
								{#each dateMemos as view (view.memo.id)}
									<MemoCard
										{view}
										onEdit={handleEdit}
										onPin={handlePin}
										onArchive={handleArchive}
										onDelete={handleDelete}
										onToggleReaction={handleToggleReaction}
										onToggleTask={handleToggleTask}
										onSelectTag={handleSelectTag}
									/>
								{/each}
							{/if}
						</div>

						<!-- Single Action Footer when memos exist on the selected day -->
						{#if authState.isAuthenticated && dateMemos.length > 0}
							<div class="shrink-0 border-t border-[var(--xm-divider)] p-3">
								<Button
									small
									tonal
									rounded
									onclick={() => selectedDate && openNewMemoOnDate(selectedDate)}
								>
									<Plus class="mr-1 h-3.5 w-3.5" aria-hidden="true" />
									{i18n.t('review.newMemoOnDate')}
								</Button>
							</div>
						{/if}
					</aside>
				{/if}
			</div>
		{:else}
			<!-- Random Review View -->
			<section class="max-w-[680px] space-y-2.5" aria-label="Random review">
				<div class="flex items-center justify-between px-1">
					<h2 class="text-sm font-bold xm-text-primary">{i18n.t('review.randomTitle')}</h2>
					<Button small outline rounded inline onclick={pickRandom}>
						<Shuffle class="mr-1.5 h-3.5 w-3.5" aria-hidden="true" /> {i18n.t('review.shuffle')}
					</Button>
				</div>
				{#if randomPick}
					<MemoCard
						view={randomPick}
						onEdit={handleEdit}
						onPin={handlePin}
						onArchive={handleArchive}
						onDelete={handleDelete}
						onToggleReaction={handleToggleReaction}
						onToggleTask={handleToggleTask}
						onSelectTag={handleSelectTag}
					/>
				{:else}
					<div class="glass-surface rounded-2xl p-6 text-center text-xs xm-text-secondary">
						{i18n.t('review.emptyRandom')}
					</div>
				{/if}
			</section>
		{/if}
	</div>
</AppShell>

{#if editorOpened}
	<MemoEditor
		opened={editorOpened}
		{editingMemo}
		initialDate={editingMemo ? null : composeDate}
		onSaved={(saved) => {
			upsertMemo(saved);
			void memosState.loadSidebarMetadata(authState.currentUser?.id ?? null);
		}}
		onClose={() => {
			editorOpened = false;
			editingMemo = null;
			composeDate = null;
		}}
	/>
{/if}
