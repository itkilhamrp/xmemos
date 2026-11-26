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
	import { page } from '$app/state';
	import { Button, Preloader } from 'konsta/svelte';
	import { MessageCirclePlus } from 'lucide-svelte';

	import { memoApi } from '#lib/api/memo.ts';
	import type { MemoView } from '#lib/types/memo.ts';
	import { i18n } from '#lib/i18n/index.svelte.ts';
	import { authState } from '#lib/state/auth.svelte.ts';
	import { confirmDeleteState } from '#lib/state/confirm-delete.svelte.ts';
	import { filterState } from '#lib/state/filters.svelte.ts';
	import { toastState } from '#lib/state/toast.svelte.ts';
	import AppShell from '#lib/components/layout/AppShell.svelte';
	import MemoCard from '#lib/components/memo/MemoCard.svelte';
	import MemoEditor from '#lib/components/memo/MemoEditor.svelte';

	let memoView = $state<MemoView | null>(null);
	let comments = $state<MemoView[]>([]);
	let loading = $state(true);
	let commentSheetOpened = $state(false);
	let editingMemo = $state<MemoView | null>(null);
	let editSheetOpened = $state(false);

	async function loadDetail(uid: string): Promise<void> {
		loading = true;
		try {
			memoView = await memoApi.getByUid(uid);
			comments = await memoApi.listComments(memoView.memo.id);
		} catch (err) {
			toastState.show(err instanceof Error ? err.message : String(err), 'error');
		} finally {
			loading = false;
		}
	}

	function jumpToTag(tag: string): void {
		filterState.selectedTag = tag;
		void goto('/');
	}

	$effect(() => {
		const uid = page.params.uid;
		if (uid) {
			untrack(() => {
				void loadDetail(uid);
			});
		}
	});
</script>

<AppShell title={i18n.t('memo.detailTitle')} backHref="/" backLabel={i18n.t('nav.home')} showFab={false}>
	{#if loading}
		<div class="flex justify-center py-12" role="status" aria-label="Loading memo details">
			<Preloader />
		</div>
	{:else if !memoView}
		<div class="glass-surface rounded-3xl p-8 text-center xm-text-secondary">{i18n.t('memo.notFound')}</div>
	{:else}
		<div class="space-y-4">
			<MemoCard
				view={memoView}
				onEdit={(v) => {
					editingMemo = v;
					editSheetOpened = true;
				}}
				onPin={async (v) => {
					memoView = await memoApi.update({ id: v.memo.id, pinned: !v.memo.pinned });
				}}
				onArchive={async (v) => {
					await memoApi.update({ id: v.memo.id, rowStatus: 'ARCHIVED' });
					toastState.show(i18n.t('home.memoArchived'), 'info');
					await goto('/');
				}}
				onDelete={(v) => {
					void confirmDeleteState.run({
						title: i18n.t('home.deleteMemoTitle'),
						message: i18n.t('home.deleteMemoMsg'),
						confirmLabel: i18n.t('common.delete'),
						onConfirm: async () => {
							await memoApi.delete(v.memo.id);
							toastState.show(i18n.t('home.memoDeleted'), 'info');
							await goto('/');
						}
					});
				}}
				onToggleTask={async (id, idx) => {
					memoView = await memoApi.toggleTask(id, idx);
				}}
				onToggleReaction={async (v, emoji) => {
					const reactions = await memoApi.toggleReaction(v.memo.uid, emoji);
					memoView = { ...v, reactions };
				}}
				onSelectTag={jumpToTag}
			/>

			<div class="flex items-center justify-between px-1">
				<h2 class="text-sm font-bold xm-text-primary">{i18n.t('memo.comments', { count: comments.length })}</h2>
				{#if authState.isAuthenticated}
					<Button small rounded inline onclick={() => (commentSheetOpened = true)}>
						<MessageCirclePlus class="mr-1.5 h-4 w-4" aria-hidden="true" /> {i18n.t('memo.reply')}
					</Button>
				{/if}
			</div>

			{#if comments.length === 0}
				<div class="glass-surface rounded-2xl p-5 text-center text-xs xm-text-secondary">
					{i18n.t('memo.noComments')}
				</div>
			{:else}
				<div class="space-y-2.5 border-l-2 border-[var(--xm-accent-border)] pl-3">
					{#each comments as c (c.memo.id)}
						<MemoCard
							view={c}
							onEdit={(v) => {
								editingMemo = v;
								editSheetOpened = true;
							}}
							onDelete={(v) => {
								void confirmDeleteState.run({
									title: i18n.t('memo.deleteCommentTitle'),
									message: i18n.t('memo.deleteCommentMsg'),
									confirmLabel: i18n.t('common.delete'),
									onConfirm: async () => {
										await memoApi.delete(v.memo.id);
										comments = comments.filter((item) => item.memo.id !== v.memo.id);
										toastState.show(i18n.t('memo.commentDeleted'), 'info');
									}
								});
							}}
							onToggleTask={async (id, idx) => {
								const updated = await memoApi.toggleTask(id, idx);
								comments = comments.map((item) => (item.memo.id === id ? updated : item));
							}}
							onToggleReaction={async (v, emoji) => {
								const reactions = await memoApi.toggleReaction(v.memo.uid, emoji);
								comments = comments.map((item) =>
									item.memo.id === v.memo.id ? { ...item, reactions } : item
								);
							}}
							onSelectTag={jumpToTag}
						/>
					{/each}
				</div>
			{/if}
		</div>
	{/if}
</AppShell>

{#if memoView}
	<MemoEditor
		opened={commentSheetOpened}
		parentMemoId={memoView.memo.id}
		onClose={() => (commentSheetOpened = false)}
		onSaved={() => memoView && void loadDetail(memoView.memo.uid)}
	/>

	<MemoEditor
		opened={editSheetOpened}
		{editingMemo}
		isComment={Boolean(editingMemo && editingMemo.memo.id !== memoView.memo.id)}
		onClose={() => {
			editSheetOpened = false;
			editingMemo = null;
		}}
		onSaved={() => memoView && void loadDetail(memoView.memo.uid)}
	/>
{/if}
