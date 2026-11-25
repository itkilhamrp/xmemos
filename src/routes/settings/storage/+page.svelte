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
	import { BlockTitle, Button, List, ListItem, Toggle } from 'konsta/svelte';
	import { Lock } from 'lucide-svelte';

	import { resourceApi } from '#lib/api/resource.ts';
	import { backupApi } from '#lib/api/backup.ts';
	import type { StorageBreakdown } from '#lib/types/resource.ts';
	import type { BackupHistoryEntry } from '#lib/types/backup.ts';
	import { i18n } from '#lib/i18n/index.svelte.ts';
	import { authState } from '#lib/state/auth.svelte.ts';
	import { confirmDeleteState } from '#lib/state/confirm-delete.svelte.ts';
	import { toastState } from '#lib/state/toast.svelte.ts';
	import { formatBytes } from '#lib/utils/date.ts';
	import { supportsNomedia } from '#lib/utils/platform.ts';
	import AppShell from '#lib/components/layout/AppShell.svelte';
	import BackupRestorePanel from '#lib/components/backup/BackupRestorePanel.svelte';

	let breakdown = $state<StorageBreakdown | null>(null);
	let history = $state<BackupHistoryEntry[]>([]);
	let nomedia = $state(false);

	async function load(): Promise<void> {
		if (!authState.isAuthenticated) return;
		const [b, h] = await Promise.all([
			resourceApi.storageBreakdown(),
			backupApi.listHistory()
		]);
		breakdown = b;
		history = h;
		nomedia = b.nomediaEnabled;
	}

	async function toggleNomedia(): Promise<void> {
		if (!authState.isAuthenticated) return;
		const next = !nomedia;
		try {
			const res = await resourceApi.setNomedia(next);
			nomedia = res.enabled;
			toastState.show(
				i18n.t(res.enabled ? 'storage.nomediaOn' : 'storage.nomediaOff'),
				'success'
			);
		} catch (err) {
			toastState.show(err instanceof Error ? err.message : String(err), 'error');
		}
	}

	async function externalizeBlobs(): Promise<void> {
		if (!authState.isAuthenticated) return;
		const count = await resourceApi.externalizeBlobs();
		toastState.show(i18n.t('storage.externalized', { count }), 'success');
		await load();
	}

	async function handleCleanOrphans(): Promise<void> {
		if (!authState.isAuthenticated) return;
		try {
			const summary = await resourceApi.scanOrphans();
			if (summary.totalOrphanCount === 0) {
				toastState.show(i18n.t('resources.storageClean'), 'info');
				return;
			}
			const details: { label: string; value: string }[] = [];
			if (summary.unlinkedResourceCount > 0) {
				details.push({
					label: i18n.t('resources.unlinkedAttachments'),
					value: `${summary.unlinkedResourceCount} ${i18n.t(summary.unlinkedResourceCount === 1 ? 'common.item' : 'common.items')}`
				});
			}
			if (summary.orphanDiskFileCount > 0) {
				details.push({
					label: i18n.t('resources.unreferencedDisk'),
					value: `${summary.orphanDiskFileCount} ${i18n.t(summary.orphanDiskFileCount === 1 ? 'common.file' : 'common.files')}`
				});
			}
			details.push({
				label: i18n.t('resources.spaceToFree'),
				value: formatBytes(summary.reclaimableBytes)
			});
			await confirmDeleteState.run({
				title: i18n.t('resources.cleanDialogTitle'),
				message: i18n.t('resources.cleanDialogMsg'),
				confirmLabel: i18n.t('common.clean'),
				forceConfirm: true,
				details,
				samples: summary.sampleFilenames,
				onConfirm: async () => {
					const res = await resourceApi.cleanOrphans();
					await load();
					toastState.show(
						i18n.t('resources.cleanedToast', { count: res.removedFiles, label: i18n.t(res.removedFiles === 1 ? 'common.item' : 'common.items'), size: formatBytes(res.freedBytes) }),
						'success'
					);
				}
			});
		} catch (err) {
			toastState.show(err instanceof Error ? err.message : String(err), 'error');
		}
	}

	$effect(() => {
		const authed = authState.isAuthenticated;
		untrack(() => {
			if (authed) {
				void load();
			} else {
				breakdown = null;
				history = [];
			}
		});
	});
</script>

<AppShell title={i18n.t('storage.title')} backHref="/settings" backLabel={i18n.t('nav.settings')} showFab={false}>
	{#if !authState.isAuthenticated}
		<div class="glass-surface mx-auto max-w-md space-y-4 rounded-3xl p-6 text-center">
			<div
				class="mx-auto flex h-12 w-12 items-center justify-center rounded-2xl bg-[var(--xm-accent-soft)] xm-text-accent"
				aria-hidden="true"
			>
				<Lock class="h-6 w-6" />
			</div>
			<div class="space-y-1.5">
				<h2 class="text-base font-bold xm-text-primary">{i18n.t('guest.storageTitle')}</h2>
				<p class="text-xs leading-relaxed xm-text-secondary">
					{i18n.t('guest.storageDesc')}
				</p>
			</div>
			<Button rounded onclick={() => goto('/auth')}>{i18n.t('auth.signIn')}</Button>
		</div>
	{:else}
		<div class="space-y-5">
			{#if breakdown}
				<div>
					<BlockTitle class="!mt-1 !mb-2 !px-1">{i18n.t('storage.breakdown')}</BlockTitle>
					<List strong inset class="!my-0 !mx-0">
						<ListItem title={i18n.t('storage.mediaRoot')} footer={breakdown.mediaRoot} />
						<ListItem title={i18n.t('storage.memosDb')} after={formatBytes(breakdown.memosDbBytes)} />
						<ListItem title={i18n.t('storage.localDb')} after={formatBytes(breakdown.localDbBytes)} />
						<ListItem title={i18n.t('storage.assets')} after={formatBytes(breakdown.assetsBytes)} />
						<ListItem title={i18n.t('storage.backups')} after={formatBytes(breakdown.backupsBytes)} />
						<ListItem title={i18n.t('storage.thumbs')} after={formatBytes(breakdown.thumbsBytes)} />
						{#if supportsNomedia()}
							<ListItem
								title={i18n.t('storage.nomediaTitle')}
								subtitle={i18n.t('storage.nomediaSub')}
							>
								{#snippet after()}
									<Toggle checked={nomedia} onchange={toggleNomedia} />
								{/snippet}
							</ListItem>
						{/if}
						<ListItem
							link
							chevron={false}
							title={i18n.t('resources.cleanOrphans')}
							subtitle={(breakdown?.orphanFileCount ?? 0) > 0
								? i18n.t('storage.orphanDetected', { count: breakdown.orphanFileCount })
								: i18n.t('storage.orphanNone')}
							onclick={handleCleanOrphans}
						>
							{#snippet after()}
								<span
									class="text-xs font-semibold {(breakdown?.orphanFileCount ?? 0) > 0
										? 'text-[#d70015] dark:text-[#ff453a]'
										: 'xm-text-muted'}"
								>
									{i18n.t((breakdown?.orphanFileCount ?? 0) > 0 ? 'common.clean' : 'common.check')}
								</span>
							{/snippet}
						</ListItem>
					</List>
				</div>

				{#if breakdown.blobResourceCount > 0}
					<div class="glass-surface flex flex-wrap items-center justify-between gap-3 rounded-2xl p-4">
						<div class="min-w-0 flex-1 text-xs leading-relaxed xm-text-secondary">
							<strong class="xm-text-primary">
								{i18n.t('storage.legacyBlobs', { count: breakdown.blobResourceCount })}
							</strong>
							{i18n.t('storage.legacyBlobsDesc')}
						</div>
						<Button small rounded inline onclick={externalizeBlobs}>{i18n.t('storage.externalize')}</Button>
					</div>
				{/if}
			{/if}

			<BackupRestorePanel {history} onChanged={() => void load()} />
		</div>
	{/if}
</AppShell>
