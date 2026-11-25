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
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { Button, Preloader } from 'konsta/svelte';
	import { Check, Trash2 } from 'lucide-svelte';

	import { inboxApi } from '#lib/api/inbox.ts';
	import type { InboxView } from '#lib/types/inbox.ts';
	import { i18n } from '#lib/i18n/index.svelte.ts';
	import { confirmDeleteState } from '#lib/state/confirm-delete.svelte.ts';
	import { toastState } from '#lib/state/toast.svelte.ts';
	import { formatRelativeTime } from '#lib/utils/date.ts';
	import AppShell from '#lib/components/layout/AppShell.svelte';

	let items = $state<InboxView[]>([]);
	let loading = $state(true);

	async function load(): Promise<void> {
		loading = true;
		try {
			items = await inboxApi.list();
		} catch (err) {
			toastState.show(err instanceof Error ? err.message : String(err), 'error');
		} finally {
			loading = false;
		}
	}

	function parseInboxSummary(raw: string): { text: string; memoUid?: string } {
		try {
			const obj = JSON.parse(raw) as { type?: string; memoUid?: string; reactionType?: string };
			if (obj.type === 'MEMO_COMMENT') {
				return { text: i18n.t('inbox.commented'), memoUid: obj.memoUid };
			}
			if (obj.type === 'MEMO_REACTION') {
				return { text: i18n.t('inbox.reacted', { emoji: obj.reactionType ?? '' }), memoUid: obj.memoUid };
			}
		} catch {
			// Fall back to plain message
		}
		return { text: raw };
	}

	onMount(() => {
		void load();
	});
</script>

<AppShell title={i18n.t('inbox.title')} backHref="/" backLabel={i18n.t('nav.home')} showFab={false}>
	{#if loading}
		<div class="flex justify-center py-12" role="status" aria-label="Loading notifications">
			<Preloader />
		</div>
	{:else if items.length === 0}
		<div class="glass-surface rounded-3xl p-8 text-center">
			<p class="font-semibold xm-text-primary">{i18n.t('inbox.emptyTitle')}</p>
			<p class="mt-1.5 text-xs leading-relaxed xm-text-secondary">
				{i18n.t('inbox.emptyDesc')}
			</p>
		</div>
	{:else}
		<div class="space-y-2.5">
			{#each items as entry (entry.item.id)}
				{@const summary = parseInboxSummary(entry.item.message)}
				<div class="glass-surface flex items-center justify-between gap-3 rounded-2xl p-4">
					<div class="min-w-0">
						<div class="text-sm xm-text-primary">
							<strong>@{entry.sender?.username ?? 'user'}</strong>
							{summary.text}
						</div>
						<div class="mt-0.5 text-xs xm-text-muted tabular-nums">
							{formatRelativeTime(entry.item.createdTs)} • {entry.item.status}
						</div>
					</div>
					<div class="flex shrink-0 items-center gap-1.5">
						{#if summary.memoUid}
							<Button small outline rounded inline onclick={() => goto(`/memo/${summary.memoUid}`)}>
								{i18n.t('common.open')}
							</Button>
						{/if}
						{#if entry.item.status === 'UNREAD'}
							<button
								type="button"
								aria-label="Mark notification as read"
								class="flex h-10 w-10 items-center justify-center rounded-full xm-text-success transition-colors hover:bg-[var(--xm-success-soft)]"
								onclick={async () => {
									await inboxApi.updateStatus(entry.item.id, 'ARCHIVED');
									await load();
								}}
							>
								<Check class="h-4 w-4" aria-hidden="true" />
							</button>
						{/if}
						<button
							type="button"
							aria-label="Delete notification"
							class="flex h-10 w-10 items-center justify-center rounded-full xm-text-danger transition-colors hover:bg-[var(--xm-danger-soft)]"
							onclick={() => {
								void confirmDeleteState.run({
									title: i18n.t('inbox.deleteTitle'),
									message: i18n.t('inbox.deleteMsg'),
									confirmLabel: i18n.t('common.delete'),
									onConfirm: async () => {
										await inboxApi.delete(entry.item.id);
										items = items.filter((i) => i.item.id !== entry.item.id);
									}
								});
							}}
						>
							<Trash2 class="h-4 w-4" aria-hidden="true" />
						</button>
					</div>
				</div>
			{/each}
		</div>
	{/if}
</AppShell>
