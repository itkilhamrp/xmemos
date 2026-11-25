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
	import { Preloader } from 'konsta/svelte';

	import { userApi } from '#lib/api/user.ts';
	import { memoApi } from '#lib/api/memo.ts';
	import { i18n } from '#lib/i18n/index.svelte.ts';
	import { authState } from '#lib/state/auth.svelte.ts';
	import { confirmDeleteState } from '#lib/state/confirm-delete.svelte.ts';
	import { filterState } from '#lib/state/filters.svelte.ts';
	import { memosState } from '#lib/state/memos.svelte.ts';
	import { toastState } from '#lib/state/toast.svelte.ts';
	import type { UserProfileStats } from '#lib/types/user.ts';
	import type { HeatmapDay, MemoView } from '#lib/types/memo.ts';
	import AppShell from '#lib/components/layout/AppShell.svelte';
	import ActivityHeatmap from '#lib/components/calendar/ActivityHeatmap.svelte';
	import MemoCard from '#lib/components/memo/MemoCard.svelte';
	import MemoEditor from '#lib/components/memo/MemoEditor.svelte';

	let stats = $state<UserProfileStats | null>(null);
	let memos = $state<MemoView[]>([]);
	let heatmap = $state<HeatmapDay[]>([]);
	let loading = $state(true);
	let editorOpened = $state(false);
	let editingMemo = $state<MemoView | null>(null);

	function replaceProfileMemo(updated: MemoView): void {
		memos = memos.map((m) => (m.memo.id === updated.memo.id ? updated : m));
		memosState.replaceItem(updated);
	}

	async function loadProfile(username: string): Promise<void> {
		loading = true;
		try {
			stats = await userApi.getProfile(username);
			const [m, hm] = await Promise.all([
				memoApi.list({ creatorId: stats.user.id, rowStatus: 'NORMAL', limit: 100 }),
				memoApi.getHeatmap(stats.user.id, 0)
			]);
			memos = m;
			heatmap = hm;
		} finally {
			loading = false;
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
		replaceProfileMemo(updated);
	}

	async function handleArchive(view: MemoView): Promise<void> {
		await memoApi.update({ id: view.memo.id, rowStatus: 'ARCHIVED' });
		memos = memos.filter((m) => m.memo.id !== view.memo.id);
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
				memos = memos.filter((m) => m.memo.id !== view.memo.id);
				memosState.items = memosState.items.filter((m) => m.memo.id !== view.memo.id);
				void memosState.loadSidebarMetadata(authState.currentUser?.id ?? null);
				toastState.show(i18n.t('home.memoDeleted'), 'info');
			}
		});
	}

	$effect(() => {
		const uname = page.params.username;
		if (uname) {
			untrack(() => {
				void loadProfile(uname);
			});
		}
	});
</script>

<AppShell
	title={stats ? `@${stats.user.username}` : i18n.t('profile.title')}
	backHref="/"
	backLabel={i18n.t('nav.home')}
	showFab={false}
>
	{#if loading}
		<div class="flex justify-center py-12" role="status" aria-label="Loading profile">
			<Preloader />
		</div>
	{:else if !stats}
		<div class="glass-surface rounded-3xl p-8 text-center xm-text-secondary">{i18n.t('profile.notFound')}</div>
	{:else}
		<div class="space-y-4">
			<section
				class="glass-surface flex items-center gap-4 rounded-3xl p-5"
				aria-label="User profile summary"
			>
				<div
					class="flex h-16 w-16 shrink-0 items-center justify-center rounded-full border border-[var(--xm-accent-border)] bg-[var(--xm-accent-soft)] text-2xl font-bold text-[var(--xm-accent)]"
					aria-hidden="true"
				>
					{(stats.user.nickname || stats.user.username).slice(0, 1).toUpperCase()}
				</div>
				<div class="min-w-0 flex-1">
					<h1 class="truncate text-lg font-bold xm-text-primary">
						{stats.user.nickname || stats.user.username}
					</h1>
					<p class="text-xs xm-text-muted">@{stats.user.username} • {stats.user.role}</p>
					{#if stats.user.description}
						<p class="mt-1 text-sm xm-text-secondary">{stats.user.description}</p>
					{/if}
					<div class="mt-2.5 flex flex-wrap gap-4 text-xs font-semibold xm-text-primary tabular-nums">
						<span>{i18n.t('profile.memos', { count: stats.memoCount })}</span>
						<span>{i18n.t('profile.tags', { count: stats.tagCount })}</span>
						<span>{i18n.t('profile.activeDays', { count: stats.activeDays })}</span>
					</div>
				</div>
			</section>

			<ActivityHeatmap days={heatmap} weeks={18} />

			<div class="space-y-3">
				{#each memos as view (view.memo.id)}
					<MemoCard
						{view}
						onEdit={handleEdit}
						onPin={handlePin}
						onArchive={handleArchive}
						onDelete={handleDelete}
						onToggleTask={async (id, idx) => {
							const updated = await memoApi.toggleTask(id, idx);
							replaceProfileMemo(updated);
						}}
						onToggleReaction={async (v, emoji) => {
							const reactions = await memoApi.toggleReaction(v.memo.uid, emoji);
							replaceProfileMemo({ ...v, reactions });
						}}
						onSelectTag={(tag) => {
							filterState.selectedTag = tag;
							void goto('/');
						}}
					/>
				{/each}
			</div>
		</div>
	{/if}
</AppShell>

{#if editorOpened}
	<MemoEditor
		opened={editorOpened}
		{editingMemo}
		onSaved={(saved) => {
			replaceProfileMemo(saved);
			void memosState.loadSidebarMetadata(authState.currentUser?.id ?? null);
		}}
		onClose={() => {
			editorOpened = false;
			editingMemo = null;
		}}
	/>
{/if}
