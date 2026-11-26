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
	import { invoke, isTauri } from '@tauri-apps/api/core';
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import { Button, Chip } from 'konsta/svelte';
	import { MapPin, Navigation, Plus, X } from 'lucide-svelte';

	import { memoApi } from '#lib/api/memo.ts';
	import type { MemoLocation, MemoView, TagCount } from '#lib/types/memo.ts';
	import { i18n } from '#lib/i18n/index.svelte.ts';
	import { authState } from '#lib/state/auth.svelte.ts';
	import { confirmDeleteState } from '#lib/state/confirm-delete.svelte.ts';
	import { filterState } from '#lib/state/filters.svelte.ts';
	import { memosState } from '#lib/state/memos.svelte.ts';
	import { toastState } from '#lib/state/toast.svelte.ts';
	import { reverseGeocodeNearestPlace } from '#lib/utils/geocode.ts';
	import { isDesktopTauri } from '#lib/utils/platform.ts';
	import AppShell from '#lib/components/layout/AppShell.svelte';
	import OsmMapCanvas, { type MapCluster } from '#lib/components/map/OsmMapCanvas.svelte';
	import MemoCard from '#lib/components/memo/MemoCard.svelte';
	import MemoEditor from '#lib/components/memo/MemoEditor.svelte';
	import TagTree from '#lib/components/tag/TagTree.svelte';
	import KonstaSkeleton from '#lib/components/ui/KonstaSkeleton.svelte';

	let locatedMemos = $state<MemoView[]>([]);
	let loading = $state(true);
	let selectedTag = $state<string | null>(null);
	let selectedCluster = $state<MapCluster | null>(null);
	let droppedPin = $state<MemoLocation | null>(null);
	let editorOpened = $state(false);
	let editingMemo = $state<MemoView | null>(null);
	let composeLocation = $state<MemoLocation | null>(null);
	let detailPanelEl = $state<HTMLElement | null>(null);
	let loadSeq = 0;

	let focusMemoUid = $derived(page.url.searchParams.get('memo'));

	// Map-scoped Tags: derived ONLY from memos with saved locations (separate from Home/Explore tags)
	let mapTags = $derived.by((): TagCount[] => {
		const counts = new Map<string, number>();
		for (const m of locatedMemos) {
			for (const t of m.memo.payload.tags) {
				counts.set(t, (counts.get(t) ?? 0) + 1);
			}
		}
		return Array.from(counts.entries())
			.map(([name, count]) => ({ name, count }))
			.sort((a, b) => b.count - a.count || a.name.localeCompare(b.name));
	});

	let visibleMemos = $derived.by((): MemoView[] => {
		if (!selectedTag) return locatedMemos;
		const prefix = `${selectedTag}/`;
		return locatedMemos.filter((m) =>
			m.memo.payload.tags.some((t) => t === selectedTag || t.startsWith(prefix))
		);
	});

	function buildOsmAndGeoUrl(lat: number, lng: number, label: string): string {
		const safeLabel = encodeURIComponent(label || `${lat.toFixed(5)}, ${lng.toFixed(5)}`);
		return `geo:${lat},${lng}?q=${lat},${lng}(${safeLabel})`;
	}

	function openInOsmAnd(loc: MemoLocation): void {
		const uri = buildOsmAndGeoUrl(loc.latitude, loc.longitude, loc.placeholder);
		const webFallback = `https://www.openstreetmap.org/?mlat=${loc.latitude.toFixed(6)}&mlon=${loc.longitude.toFixed(6)}#map=15/${loc.latitude.toFixed(6)}/${loc.longitude.toFixed(6)}`;
		if (isTauri()) {
			void invoke('plugin:opener|open_url', { url: uri }).catch(() => {
				if (isDesktopTauri()) {
					void invoke('plugin:opener|open_url', { url: webFallback }).catch(() => {
						window.open(webFallback, '_blank', 'noopener,noreferrer');
					});
				} else {
					window.open(uri, '_blank', 'noopener,noreferrer');
				}
			});
		} else {
			window.open(webFallback, '_blank', 'noopener,noreferrer');
		}
	}

	function upsertLocatedMemo(saved: MemoView): void {
		if (saved.memo.rowStatus === 'NORMAL' && saved.memo.payload.location) {
			const exists = locatedMemos.some((m) => m.memo.id === saved.memo.id);
			locatedMemos = exists
				? locatedMemos.map((m) => (m.memo.id === saved.memo.id ? saved : m))
				: [saved, ...locatedMemos];
		} else {
			locatedMemos = locatedMemos.filter((m) => m.memo.id !== saved.memo.id);
		}
		if (selectedCluster) {
			const updatedList = selectedCluster.memos
				.map((m) => (m.memo.id === saved.memo.id ? saved : m))
				.filter((m) => m.memo.rowStatus === 'NORMAL' && Boolean(m.memo.payload.location));
			if (updatedList.length > 0) {
				selectedCluster = { ...selectedCluster, memos: updatedList };
			} else {
				selectedCluster = null;
			}
		}
		memosState.replaceItem(saved);
	}

	function removeLocatedMemo(memoId: number): void {
		locatedMemos = locatedMemos.filter((m) => m.memo.id !== memoId);
		memosState.items = memosState.items.filter((m) => m.memo.id !== memoId);
		if (selectedCluster) {
			const remaining = selectedCluster.memos.filter((m) => m.memo.id !== memoId);
			selectedCluster = remaining.length > 0 ? { ...selectedCluster, memos: remaining } : null;
		}
		void memosState.loadSidebarMetadata(authState.currentUser?.id ?? null);
	}

	async function scrollDetailIntoViewOnMobile(): Promise<void> {
		if (typeof window !== 'undefined' && window.innerWidth < 1024) {
			await tick();
			detailPanelEl?.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
		}
	}

	function handleSelectCluster(cluster: MapCluster): void {
		droppedPin = null;
		selectedCluster = cluster;
		void scrollDetailIntoViewOnMobile();
	}

	let pinGeocodeSeq = 0;

	function handleDropPin(lat: number, lng: number): void {
		selectedCluster = null;
		const seq = ++pinGeocodeSeq;
		droppedPin = {
			placeholder: `${lat.toFixed(5)}, ${lng.toFixed(5)}`,
			latitude: lat,
			longitude: lng
		};
		void scrollDetailIntoViewOnMobile();
		void reverseGeocodeNearestPlace(lat, lng, i18n.bcp47).then((place) => {
			if (seq !== pinGeocodeSeq || !place || !droppedPin) return;
			droppedPin = { ...droppedPin, placeholder: place };
		});
	}

	function openWriteHere(loc: MemoLocation): void {
		editingMemo = null;
		composeLocation = { ...loc };
		editorOpened = true;
	}

	function handleEdit(view: MemoView): void {
		editingMemo = view;
		composeLocation = null;
		editorOpened = true;
	}

	async function handlePin(view: MemoView): Promise<void> {
		const updated = await memoApi.update({
			id: view.memo.id,
			pinned: !view.memo.pinned
		});
		upsertLocatedMemo(updated);
	}

	async function handleArchive(view: MemoView): Promise<void> {
		await memoApi.update({ id: view.memo.id, rowStatus: 'ARCHIVED' });
		removeLocatedMemo(view.memo.id);
		toastState.show(i18n.t('home.memoArchived'), 'info');
	}

	function handleDelete(view: MemoView): void {
		void confirmDeleteState.run({
			title: i18n.t('home.deleteMemoTitle'),
			message: i18n.t('home.deleteMemoMsg'),
			confirmLabel: i18n.t('common.delete'),
			onConfirm: async () => {
				await memoApi.delete(view.memo.id);
				removeLocatedMemo(view.memo.id);
				toastState.show(i18n.t('home.memoDeleted'), 'info');
			}
		});
	}

	$effect(() => {
		const uid = authState.currentUser?.id ?? null;
		untrack(() => {
			const seq = ++loadSeq;
			loading = true;
			void memoApi
				.list({
					rowStatus: 'NORMAL',
					visibilityList: uid ? undefined : ['PUBLIC'],
					limit: 2000
				})
				.then((all) => {
					if (seq !== loadSeq) return;
					locatedMemos = all.filter(
						(m) =>
							m.memo.payload.location &&
							Number.isFinite(m.memo.payload.location.latitude) &&
							Number.isFinite(m.memo.payload.location.longitude)
					);
					if (focusMemoUid) {
						const target = locatedMemos.find((m) => m.memo.uid === focusMemoUid);
						if (target?.memo.payload.location) {
							selectedCluster = {
								id: `c-${target.memo.id}`,
								latitude: target.memo.payload.location.latitude,
								longitude: target.memo.payload.location.longitude,
								memos: [target],
								label:
									target.memo.payload.location.placeholder ||
									`${target.memo.payload.location.latitude.toFixed(4)}, ${target.memo.payload.location.longitude.toFixed(4)}`
							};
						}
					}
				})
				.finally(() => {
					if (seq === loadSeq) loading = false;
				});
		});
	});
</script>

<AppShell title={i18n.t('map.title')} showFab={false}>
	{#snippet desktopLeftTags()}
		<TagTree
			tags={mapTags}
			{selectedTag}
			title={i18n.t('map.tagsTitle')}
			onSelectTag={(t) => {
				selectedTag = t;
				selectedCluster = null;
			}}
		/>
	{/snippet}

	<div class="flex flex-col gap-3 md:h-[calc(100dvh-5.25rem)] md:overflow-hidden">
		<!-- Compact Map Status & Active Tag Pill Bar -->
		<div class="flex shrink-0 flex-wrap items-center justify-between gap-2 px-0.5">
			<div class="flex flex-wrap items-center gap-2 text-xs xm-text-secondary">
				<span class="inline-flex items-center gap-1.5 font-semibold xm-text-primary tabular-nums">
					<MapPin class="h-3.5 w-3.5 xm-text-accent" aria-hidden="true" />
					{i18n.t('map.pinnedCount', { count: visibleMemos.length })}
				</span>
				<span class="hidden xm-text-muted sm:inline">•</span>
				<span class="hidden xm-text-muted sm:inline">{i18n.t('map.hintTapToPin')}</span>
				{#if selectedTag}
					<Chip deleteButton class="!text-xs" onDelete={() => (selectedTag = null)}>
						#{selectedTag}
					</Chip>
				{/if}
			</div>
		</div>

		{#if loading && locatedMemos.length === 0}
			<KonstaSkeleton variant="memo" count={2} label={i18n.t('common.loading')} />
		{:else}
			<div class="flex flex-col gap-3.5 md:min-h-0 md:flex-1 lg:flex-row">
				<!-- Mobile & Tablet/iPad (< 1024px): Map-Scoped Tags (moved to Desktop Left Rail on lg+) -->
				{#if mapTags.length > 0}
					<div class="order-3 shrink-0 lg:hidden">
						<TagTree
							tags={mapTags}
							{selectedTag}
							title={i18n.t('map.tagsTitle')}
							onSelectTag={(t) => {
								selectedTag = t;
								selectedCluster = null;
							}}
						/>
					</div>
				{/if}

				<!-- Interactive OsmAnd / OpenStreetMap Offline-First Canvas -->
				<div class="order-1 h-[min(60dvh,460px)] min-h-[340px] w-full min-w-0 shrink-0 sm:h-[480px] md:h-full md:min-h-0 md:flex-1 md:shrink">
					<OsmMapCanvas
						memos={visibleMemos}
						selectedClusterId={selectedCluster?.id ?? null}
						{droppedPin}
						{focusMemoUid}
						onSelectCluster={handleSelectCluster}
						onDropPin={handleDropPin}
					/>
				</div>

				<!-- Selected Location / Dropped Pin Detail Panel (Right on Desktop, Below on Mobile) -->
				{#if selectedCluster || droppedPin}
					{@const activeLoc = selectedCluster
						? {
								placeholder: selectedCluster.label,
								latitude: selectedCluster.latitude,
								longitude: selectedCluster.longitude
							}
						: droppedPin!}
					<aside
						bind:this={detailPanelEl}
						aria-label={i18n.t('map.selectedMemos')}
						class="glass-surface order-2 flex w-full flex-col overflow-hidden rounded-2xl border border-[var(--xm-divider)] lg:h-full lg:w-[356px] lg:shrink-0 xl:w-[388px]"
					>
						<div
							class="flex shrink-0 items-center justify-between gap-2 border-b border-[var(--xm-divider)] px-4 py-3"
						>
							<div class="min-w-0">
								<h2 class="truncate text-sm font-bold xm-text-primary">
									{selectedCluster ? selectedCluster.label : (droppedPin?.placeholder || i18n.t('map.newPinTitle'))}
								</h2>
								<p class="truncate text-[11px] xm-text-muted tabular-nums">
									{activeLoc.latitude.toFixed(5)}, {activeLoc.longitude.toFixed(5)}
								</p>
							</div>
							<button
								type="button"
								aria-label={i18n.t('calendar.closePanel')}
								class="flex h-8 w-8 shrink-0 items-center justify-center rounded-full xm-text-secondary transition-colors hover:bg-[var(--xm-surface-subtle)] hover:text-[var(--xm-text-primary)]"
								onclick={() => {
									selectedCluster = null;
									droppedPin = null;
								}}
							>
								<X class="h-4 w-4" aria-hidden="true" />
							</button>
						</div>

						<div class="min-h-0 flex-1 space-y-3 p-3.5 lg:overflow-y-auto">
							{#if selectedCluster}
								{#each selectedCluster.memos as view (view.memo.id)}
									<MemoCard
										{view}
										onEdit={handleEdit}
										onPin={handlePin}
										onArchive={handleArchive}
										onDelete={handleDelete}
										onToggleTask={async (id, idx) => {
											const updated = await memoApi.toggleTask(id, idx);
											upsertLocatedMemo(updated);
										}}
										onToggleReaction={async (v, emoji) => {
											const reactions = await memoApi.toggleReaction(v.memo.uid, emoji);
											upsertLocatedMemo({ ...v, reactions });
										}}
										onSelectTag={(tag) => {
											selectedTag = selectedTag === tag ? null : tag;
										}}
									/>
								{/each}
							{:else if droppedPin}
								<div class="space-y-3 py-6 text-center">
									<MapPin class="mx-auto h-7 w-7 xm-text-accent" aria-hidden="true" />
									<p class="text-xs leading-relaxed xm-text-secondary">
										{i18n.t('map.droppedPinDesc')}
									</p>
								</div>
							{/if}
						</div>

						<!-- Footer Actions: Write Here + Open in OsmAnd -->
						<div class="flex shrink-0 flex-wrap items-center gap-2 border-t border-[var(--xm-divider)] p-3">
							{#if authState.isAuthenticated}
								<div class="min-w-0 flex-1">
									<Button small rounded onclick={() => openWriteHere(activeLoc)}>
										<Plus class="mr-1 h-3.5 w-3.5" aria-hidden="true" />
										{i18n.t('map.writeHere')}
									</Button>
								</div>
							{/if}
							<Button
								small
								tonal
								rounded
								inline
								onclick={() => openInOsmAnd(activeLoc)}
							>
								<Navigation class="mr-1 h-3.5 w-3.5" aria-hidden="true" />
								{i18n.t('map.openOsmAnd')}
							</Button>
						</div>
					</aside>
				{/if}
			</div>
		{/if}
	</div>
</AppShell>

{#if editorOpened}
	<MemoEditor
		opened={editorOpened}
		{editingMemo}
		initialLocation={editingMemo ? null : composeLocation}
		onSaved={(saved) => {
			upsertLocatedMemo(saved);
			droppedPin = null;
			void memosState.loadSidebarMetadata(authState.currentUser?.id ?? null);
		}}
		onClose={() => {
			editorOpened = false;
			editingMemo = null;
			composeLocation = null;
		}}
	/>
{/if}
