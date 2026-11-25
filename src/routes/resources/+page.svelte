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
	import { BlockTitle, Chip, List, ListItem } from 'konsta/svelte';
	import { Download, FileText, Film, Music, Trash2 } from 'lucide-svelte';

	import { resourceApi } from '#lib/api/resource.ts';
	import { getMediaUrl } from '#lib/api/invoke.ts';
	import type { OrphanScanSummary, Resource } from '#lib/types/resource.ts';
	import { i18n, type TranslationKey } from '#lib/i18n/index.svelte.ts';
	import { confirmDeleteState } from '#lib/state/confirm-delete.svelte.ts';
	import { lightboxState } from '#lib/state/lightbox.svelte.ts';
	import { toastState } from '#lib/state/toast.svelte.ts';
	import { formatBytes } from '#lib/utils/date.ts';
	import { formatSavedLabel } from '#lib/utils/download.ts';
	import AppShell from '#lib/components/layout/AppShell.svelte';
	import InfiniteScrollSentinel from '#lib/components/ui/InfiniteScrollSentinel.svelte';
	import KonstaSkeleton from '#lib/components/ui/KonstaSkeleton.svelte';

	type MediaKind = 'ALL' | 'IMAGE' | 'VIDEO' | 'AUDIO' | 'DOC';

	const FILTER_TABS: { id: MediaKind; labelKey: TranslationKey }[] = [
		{ id: 'ALL', labelKey: 'resources.filter.all' },
		{ id: 'IMAGE', labelKey: 'resources.filter.photos' },
		{ id: 'VIDEO', labelKey: 'resources.filter.videos' },
		{ id: 'AUDIO', labelKey: 'resources.filter.audio' },
		{ id: 'DOC', labelKey: 'resources.filter.files' }
	];

	let resources = $state<Resource[]>([]);
	let orphanSummary = $state<OrphanScanSummary | null>(null);
	let loading = $state(true);
	let scanning = $state(false);
	const MEDIA_BATCH = 24;
	let filterKind = $state<MediaKind>('ALL');
	let visibleCount = $state(MEDIA_BATCH);

	let filtered = $derived(
		resources.filter((r) => {
			if (filterKind === 'ALL') return true;
			if (filterKind === 'IMAGE') return r.type.startsWith('image/');
			if (filterKind === 'VIDEO') return r.type.startsWith('video/');
			if (filterKind === 'AUDIO') return r.type.startsWith('audio/');
			return (
				!r.type.startsWith('image/') &&
				!r.type.startsWith('video/') &&
				!r.type.startsWith('audio/')
			);
		})
	);

	let visibleResources = $derived(filtered.slice(0, visibleCount));
	let hasMoreMedia = $derived(visibleCount < filtered.length);

	function loadMoreMedia(): void {
		visibleCount = Math.min(filtered.length, visibleCount + MEDIA_BATCH);
	}

	function countFor(kind: MediaKind): number {
		if (kind === 'ALL') return resources.length;
		if (kind === 'IMAGE') return resources.filter((r) => r.type.startsWith('image/')).length;
		if (kind === 'VIDEO') return resources.filter((r) => r.type.startsWith('video/')).length;
		if (kind === 'AUDIO') return resources.filter((r) => r.type.startsWith('audio/')).length;
		return resources.filter(
			(r) =>
				!r.type.startsWith('image/') &&
				!r.type.startsWith('video/') &&
				!r.type.startsWith('audio/')
		).length;
	}

	async function load(): Promise<void> {
		loading = true;
		try {
			const [list, scan] = await Promise.all([
				resourceApi.list(),
				resourceApi.scanOrphans().catch(() => null)
			]);
			resources = list;
			orphanSummary = scan;
		} catch (err) {
			toastState.show(err instanceof Error ? err.message : String(err), 'error');
		} finally {
			loading = false;
		}
	}

	async function downloadResource(res: Resource): Promise<void> {
		try {
			const savedPath = await resourceApi.exportToDownloads(res.uid);
			if (savedPath !== null) {
				toastState.show(
					i18n.t('lightbox.downloaded', {
						filename: formatSavedLabel(savedPath, res.filename)
					}),
					'success'
				);
			}
		} catch (err) {
			toastState.show(
				i18n.t('lightbox.downloadFailed', {
					error: err instanceof Error ? err.message : String(err)
				}),
				'error'
			);
		}
	}

	function remove(res: Resource): void {
		void confirmDeleteState.run({
			title: i18n.t('resources.deleteTitle'),
			message: i18n.t('resources.deleteMsg', { filename: res.filename }),
			confirmLabel: i18n.t('common.delete'),
			onConfirm: async () => {
				await resourceApi.delete(res.id);
				resources = resources.filter((r) => r.id !== res.id);
				orphanSummary = await resourceApi.scanOrphans().catch(() => orphanSummary);
				toastState.show(i18n.t('resources.deleted', { filename: res.filename }), 'info');
			}
		});
	}

	async function handleCleanOrphans(): Promise<void> {
		if (scanning) return;
		scanning = true;
		try {
			const summary = await resourceApi.scanOrphans();
			orphanSummary = summary;

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

			// Always require confirmation for bulk orphan cleanup regardless of Settings toggle
			await confirmDeleteState.run({
				title: i18n.t('resources.cleanDialogTitle'),
				message:
					i18n.t('resources.cleanDialogMsg'),
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
		} finally {
			scanning = false;
		}
	}

	onMount(() => {
		void load();
	});
</script>

<AppShell title={i18n.t('resources.title')} showFab={false}>
	<div class="space-y-4">
		<!-- Clean Title-Case Filter Chips -->
		<div class="flex flex-wrap items-center gap-1.5" role="group" aria-label="Filter media by type">
			{#each FILTER_TABS as tab (tab.id)}
				{@const count = countFor(tab.id)}
				<Chip
					outline={filterKind !== tab.id}
					role="button"
					aria-pressed={filterKind === tab.id}
					class="cursor-pointer !text-xs tabular-nums"
					onclick={() => {
						filterKind = tab.id;
						visibleCount = MEDIA_BATCH;
					}}
				>
					{i18n.t(tab.labelKey)}{#if tab.id !== 'ALL' && count > 0}&nbsp;({count}){/if}
				</Chip>
			{/each}
		</div>

		{#if loading}
			<KonstaSkeleton variant="media" count={6} label={i18n.t('common.loading')} />
		{:else if filtered.length === 0}
			<div class="glass-surface rounded-3xl p-8 text-center">
				<p class="font-semibold xm-text-primary">{i18n.t('resources.emptyTitle')}</p>
				<p class="mt-1.5 text-xs leading-relaxed xm-text-secondary">
					{i18n.t('resources.emptyDesc')}
				</p>
			</div>
		{:else}
			<div class="grid grid-cols-2 gap-3 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5">
				{#each visibleResources as res (res.id)}
					<div class="glass-surface flex flex-col overflow-hidden rounded-2xl">
						<div class="relative aspect-square w-full overflow-hidden bg-[var(--xm-surface-subtle)]">
							{#if res.type.startsWith('image/')}
								<button
									type="button"
									aria-label="Preview {res.filename}"
									class="h-full w-full"
									onclick={() =>
										lightboxState.open({
											uid: res.uid,
											src: getMediaUrl(res.uid, false),
											filename: res.filename,
											size: res.size
										})}
								>
									<img
										src={getMediaUrl(res.uid, true)}
										alt={res.filename}
										loading="lazy"
										decoding="async"
										class="h-full w-full object-cover transition-transform duration-200 hover:scale-[1.03]"
									/>
								</button>
							{:else}
								<div
									class="flex h-full w-full flex-col items-center justify-center gap-2 bg-[var(--xm-accent-soft)] p-3 text-center"
								>
									{#if res.type.startsWith('video/')}
										<Film class="h-8 w-8 xm-text-accent" aria-hidden="true" />
									{:else if res.type.startsWith('audio/')}
										<Music class="h-8 w-8 xm-text-accent" aria-hidden="true" />
									{:else}
										<FileText class="h-8 w-8 xm-text-accent" aria-hidden="true" />
									{/if}
									<span class="max-w-full truncate text-[11px] font-semibold xm-text-secondary">
										{res.type || 'file'}
									</span>
								</div>
							{/if}

							{#if res.memoId === null}
								<span
									class="pointer-events-none absolute top-2 left-2 rounded-full bg-black/65 px-2 py-0.5 text-[10px] font-medium text-white/90"
								>
									{i18n.t('resources.unlinked')}
								</span>
							{/if}
						</div>

						<div class="flex items-center justify-between gap-2 p-2.5">
							<div class="min-w-0">
								<div class="truncate text-xs font-semibold xm-text-primary">{res.filename}</div>
								<div class="text-[11px] xm-text-muted tabular-nums">{formatBytes(res.size)}</div>
							</div>
							<div class="flex shrink-0 items-center gap-0.5">
								<button
									type="button"
									aria-label="Download {res.filename}"
									title="Download {res.filename}"
									class="flex h-8 w-8 shrink-0 items-center justify-center rounded-full xm-text-accent transition-colors hover:bg-[var(--xm-accent-soft)]"
									onclick={() => downloadResource(res)}
								>
									<Download class="h-4 w-4" aria-hidden="true" />
								</button>
								<button
									type="button"
									aria-label="Delete {res.filename}"
									class="flex h-8 w-8 shrink-0 items-center justify-center rounded-full xm-text-danger transition-colors hover:bg-[var(--xm-danger-soft)]"
									onclick={() => remove(res)}
								>
									<Trash2 class="h-4 w-4" aria-hidden="true" />
								</button>
							</div>
						</div>
					</div>
				{/each}
			</div>
			<InfiniteScrollSentinel
				hasMore={hasMoreMedia}
				loading={false}
				itemCount={visibleResources.length}
				variant="media"
				onLoadMore={loadMoreMedia}
			/>
		{/if}

		<!-- Deliberately placed bottom Konsta UI iOS Inset Maintenance Section -->
		<div class="max-w-2xl pt-4">
			<BlockTitle class="!mt-1 !mb-2 !px-1">{i18n.t('resources.maintenance')}</BlockTitle>
			<List strong inset class="!my-0 !mx-0">
				<ListItem
					link
					chevron={false}
					title={i18n.t('resources.cleanOrphans')}
					subtitle={orphanSummary && orphanSummary.totalOrphanCount > 0
						? i18n.t('resources.orphanSubtitleFound', {
								count: orphanSummary.totalOrphanCount,
								label: i18n.t(orphanSummary.totalOrphanCount === 1 ? 'common.item' : 'common.items'),
								size: formatBytes(orphanSummary.reclaimableBytes)
							})
						: i18n.t('resources.orphanSubtitleEmpty')}
					onclick={handleCleanOrphans}
				>
					{#snippet after()}
						<span
							class="text-xs font-semibold {orphanSummary && orphanSummary.totalOrphanCount > 0
								? 'text-[#d70015] dark:text-[#ff453a]'
								: 'xm-text-muted'}"
						>
							{scanning
								? i18n.t('resources.scanning')
								: orphanSummary && orphanSummary.totalOrphanCount > 0
									? i18n.t('common.clean')
									: i18n.t('common.check')}
						</span>
					{/snippet}
				</ListItem>
			</List>
		</div>
	</div>
</AppShell>
