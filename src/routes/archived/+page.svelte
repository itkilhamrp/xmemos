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
	import { untrack } from 'svelte';
	import { goto } from '$app/navigation';

	import { memoApi } from '#lib/api/memo.ts';
	import type { MemoView } from '#lib/types/memo.ts';
	import { authState } from '#lib/state/auth.svelte.ts';
	import { confirmDeleteState } from '#lib/state/confirm-delete.svelte.ts';
	import { filterState } from '#lib/state/filters.svelte.ts';
	import { i18n } from '#lib/i18n/index.svelte.ts';
	import { memosState } from '#lib/state/memos.svelte.ts';
	import { toastState } from '#lib/state/toast.svelte.ts';
	import AppShell from '#lib/components/layout/AppShell.svelte';
	import MemoCard from '#lib/components/memo/MemoCard.svelte';
	import MemoEditor from '#lib/components/memo/MemoEditor.svelte';
	import InfiniteScrollSentinel from '#lib/components/ui/InfiniteScrollSentinel.svelte';
	import KonstaSkeleton from '#lib/components/ui/KonstaSkeleton.svelte';

	const PAGE_SIZE = 20;

	let items = $state<MemoView[]>([]);
	let loading = $state(true);
	let loadingMore = $state(false);
	let hasMore = $state(true);
	let offset = $state(0);
	let editorOpened = $state(false);
	let editingMemo = $state<MemoView | null>(null);
	let requestSeq = 0;

	async function load(reset = true): Promise<void> {
		if (!reset && (loading || loadingMore || !hasMore)) return;
		const seq = ++requestSeq;
		if (reset) {
			loading = true;
		} else {
			loadingMore = true;
		}
		try {
			const nextOffset = reset ? 0 : offset;
			const batch = await memoApi.list({
				creatorId: authState.currentUser?.id ?? null,
				rowStatus: 'ARCHIVED',
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

	function replaceArchivedMemo(updated: MemoView): void {
		if (updated.memo.rowStatus !== 'ARCHIVED') {
			items = items.filter((v) => v.memo.id !== updated.memo.id);
		} else {
			items = items.map((v) => (v.memo.id === updated.memo.id ? updated : v));
		}
	}

	function handleEdit(view: MemoView): void {
		editingMemo = view;
		editorOpened = true;
	}

	async function handleRestore(view: MemoView): Promise<void> {
		await memoApi.update({ id: view.memo.id, rowStatus: 'NORMAL' });
		items = items.filter((v) => v.memo.id !== view.memo.id);
		void memosState.loadSidebarMetadata(authState.currentUser?.id ?? null);
		toastState.show(i18n.t('archived.restored'), 'success');
	}

	function handleDelete(view: MemoView): void {
		void confirmDeleteState.run({
			title: i18n.t('archived.deleteTitle'),
			message: i18n.t('archived.deleteMsg'),
			confirmLabel: i18n.t('common.delete'),
			onConfirm: async () => {
				await memoApi.delete(view.memo.id);
				items = items.filter((v) => v.memo.id !== view.memo.id);
				toastState.show(i18n.t('archived.deleted'), 'info');
			}
		});
	}

	async function handleToggleTask(memoId: number, taskIndex: number): Promise<void> {
		const updated = await memoApi.toggleTask(memoId, taskIndex);
		replaceArchivedMemo(updated);
	}

	async function handleToggleReaction(view: MemoView, emoji: string): Promise<void> {
		const reactions = await memoApi.toggleReaction(view.memo.uid, emoji);
		replaceArchivedMemo({ ...view, reactions });
	}

	function handleSelectTag(tag: string): void {
		filterState.selectedTag = tag;
		void goto('/');
	}

	$effect(() => {
		const _uid = authState.currentUser?.id ?? null;
		void _uid;
		untrack(() => {
			void load(true);
		});
	});
</script>

<AppShell title={i18n.t('archived.title')} backHref="/" backLabel={i18n.t('nav.home')} showFab={false}>
	{#if loading && items.length === 0}
		<KonstaSkeleton variant="memo" count={3} label={i18n.t('common.loading')} />
	{:else if items.length === 0}
		<div class="glass-surface rounded-3xl p-8 text-center">
			<p class="font-semibold xm-text-primary">{i18n.t('archived.emptyTitle')}</p>
			<p class="mt-1.5 text-xs leading-relaxed xm-text-secondary">
				{i18n.t('archived.emptyDesc')}
			</p>
		</div>
	{:else}
		<div class="space-y-3">
			{#each items as view (view.memo.id)}
				<MemoCard
					{view}
					onEdit={handleEdit}
					onRestore={handleRestore}
					onDelete={handleDelete}
					onToggleTask={handleToggleTask}
					onToggleReaction={handleToggleReaction}
					onSelectTag={handleSelectTag}
				/>
			{/each}
		</div>

		<InfiniteScrollSentinel
			{hasMore}
			loading={loadingMore}
			itemCount={items.length}
			onLoadMore={() => load(false)}
		/>
	{/if}
</AppShell>

{#if editorOpened}
	<MemoEditor
		opened={editorOpened}
		{editingMemo}
		onSaved={(saved) => {
			replaceArchivedMemo(saved);
		}}
		onClose={() => {
			editorOpened = false;
			editingMemo = null;
		}}
	/>
{/if}
