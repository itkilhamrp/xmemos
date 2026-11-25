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
	import { page } from '$app/state';
	import { isTauri } from '@tauri-apps/api/core';
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import {
		Check,
		ExternalLink,
		Globe,
		Link2,
		Lock,
		MessageCircle,
		Pencil,
		Pin,
		PinOff,
		Users,
		X
	} from 'lucide-svelte';

	import { memoApi } from '#lib/api/memo.ts';
	import {
		loadStickyPrefs,
		saveStickyPrefs,
		shortcutApi,
		type StickyColorId
	} from '#lib/api/shortcut.ts';
	import { i18n } from '#lib/i18n/index.svelte.ts';
	import { authState } from '#lib/state/auth.svelte.ts';
	import { toastState } from '#lib/state/toast.svelte.ts';
	import type { MemoView } from '#lib/types/memo.ts';
	import { formatFullTimestamp } from '#lib/utils/date.ts';
	import { handleStickyDragMouseDown } from '#lib/utils/platform.ts';
	import AppIcon from '#lib/components/ui/AppIcon.svelte';
	import MemoContent from '#lib/components/memo/MemoContent.svelte';
	import AttachmentGrid from '#lib/components/media/AttachmentGrid.svelte';
	import ImageLightbox from '#lib/components/media/ImageLightbox.svelte';
	import MemoLocationPreview from '#lib/components/map/MemoLocationPreview.svelte';
	import LocationPreviewPopover from '#lib/components/map/LocationPreviewPopover.svelte';

	const STICKY_THEMES: Record<
		StickyColorId,
		{
			label: string;
			swatch: string;
			accentBar: string;
			header: string;
			surface: string;
			border: string;
		}
	> = {
		slate: {
			label: 'Graphite Glass',
			swatch: '#0a84ff',
			accentBar: 'bg-[#007aff] dark:bg-[#0a84ff]',
			header: 'bg-[#f2f4f8]/95 dark:bg-[#242428]/95 border-black/[0.08] dark:border-white/[0.10]',
			surface: 'bg-[#f8f9fc] dark:bg-[#1c1c1e] text-[#0f172a] dark:text-[#f8fafc]',
			border: 'border-black/[0.08] dark:border-white/[0.10]'
		},
		yellow: {
			label: 'Amber Glow',
			swatch: '#f59e0b',
			accentBar: 'bg-[#f59e0b] dark:bg-[#fbbf24]',
			header: 'bg-[#fef3c7]/95 dark:bg-[#2b2619]/95 border-[#d97706]/20 dark:border-[#fbbf24]/18',
			surface: 'bg-[#fffbeb] dark:bg-[#1f1c14] text-[#0f172a] dark:text-[#f8fafc]',
			border: 'border-[#d97706]/20 dark:border-[#fbbf24]/18'
		},
		blue: {
			label: 'Oceanic Blue',
			swatch: '#38bdf8',
			accentBar: 'bg-[#0284c7] dark:bg-[#38bdf8]',
			header: 'bg-[#e0f2fe]/95 dark:bg-[#162638]/95 border-[#0284c7]/20 dark:border-[#38bdf8]/18',
			surface: 'bg-[#f0f9ff] dark:bg-[#111c29] text-[#0f172a] dark:text-[#f8fafc]',
			border: 'border-[#0284c7]/20 dark:border-[#38bdf8]/18'
		},
		green: {
			label: 'Emerald Mint',
			swatch: '#34d399',
			accentBar: 'bg-[#10b981] dark:bg-[#34d399]',
			header: 'bg-[#dcfce7]/95 dark:bg-[#152d21]/95 border-[#059669]/20 dark:border-[#34d399]/18',
			surface: 'bg-[#f0fdf4] dark:bg-[#112119] text-[#0f172a] dark:text-[#f8fafc]',
			border: 'border-[#059669]/20 dark:border-[#34d399]/18'
		},
		pink: {
			label: 'Rose Quartz',
			swatch: '#f472b6',
			accentBar: 'bg-[#ec4899] dark:bg-[#f472b6]',
			header: 'bg-[#fce7f3]/95 dark:bg-[#301928]/95 border-[#db2777]/20 dark:border-[#f472b6]/18',
			surface: 'bg-[#fdf2f8] dark:bg-[#23141e] text-[#0f172a] dark:text-[#f8fafc]',
			border: 'border-[#db2777]/20 dark:border-[#f472b6]/18'
		},
		purple: {
			label: 'Indigo Mist',
			swatch: '#a78bfa',
			accentBar: 'bg-[#8b5cf6] dark:bg-[#a78bfa]',
			header: 'bg-[#ede9fe]/95 dark:bg-[#221d36]/95 border-[#7c3aed]/20 dark:border-[#a78bfa]/18',
			surface: 'bg-[#f5f3ff] dark:bg-[#191628] text-[#0f172a] dark:text-[#f8fafc]',
			border: 'border-[#7c3aed]/20 dark:border-[#a78bfa]/18'
		}
	};

	const COLOR_ORDER: StickyColorId[] = ['slate', 'yellow', 'blue', 'green', 'pink', 'purple'];

	let uid = $derived(page.params.uid ?? '');
	let view = $state<MemoView | null>(null);
	let loading = $state(true);
	let editing = $state(false);
	let draftContent = $state('');
	let saving = $state(false);
	let color = $state<StickyColorId>('slate');
	let alwaysOnTop = $state(false);
	let togglingTop = $state(false);
	let commentsOpen = $state(false);
	let commentsLoading = $state(false);
	let comments = $state<MemoView[]>([]);

	let isOwner = $derived(
		authState.currentUser !== null && view !== null && authState.currentUser.id === view.memo.creatorId
	);
	let palette = $derived(STICKY_THEMES[color]);
	let referenceRelations = $derived(
		view ? view.relations.filter((r) => r.type === 'REFERENCE') : []
	);
	let groupedReactions = $derived.by(() => {
		if (!view) return [];
		const map = new Map<string, { emoji: string; count: number; reactedByMe: boolean }>();
		for (const r of view.reactions) {
			const entry = map.get(r.reactionType) ?? {
				emoji: r.reactionType,
				count: 0,
				reactedByMe: false
			};
			entry.count += 1;
			if (r.creatorId === authState.currentUser?.id) {
				entry.reactedByMe = true;
			}
			map.set(r.reactionType, entry);
		}
		return Array.from(map.values());
	});

	async function loadMemo(): Promise<void> {
		if (!uid) return;
		loading = true;
		try {
			const res = await memoApi.getByUid(uid);
			view = res;
			draftContent = res?.memo.content ?? '';
		} catch (err) {
			toastState.show(err instanceof Error ? err.message : String(err), 'error');
		} finally {
			loading = false;
		}
	}

	function persistPrefs(
		patch: Partial<{
			x: number;
			y: number;
			width: number;
			height: number;
			color: StickyColorId;
			alwaysOnTop: boolean;
		}>
	): void {
		if (!uid) return;
		const current = loadStickyPrefs(uid);
		saveStickyPrefs(uid, {
			...current,
			color,
			alwaysOnTop,
			...patch
		});
	}

	function selectColor(next: StickyColorId): void {
		color = next;
		persistPrefs({ color: next });
	}

	async function applyAlwaysOnTopNative(
		next: boolean
	): Promise<{ active: boolean; supported: boolean }> {
		if (!isTauri()) return { active: next, supported: true };
		const win = getCurrentWindow();
		try {
			const res = await shortcutApi.setStickyAlwaysOnTop(win.label, next);
			return { active: res.active, supported: res.supported };
		} catch {
			await win.setAlwaysOnTop(next);
			return { active: next, supported: true };
		}
	}

	async function toggleAlwaysOnTop(): Promise<void> {
		if (togglingTop) return;
		togglingTop = true;
		try {
			const next = !alwaysOnTop;
			const res = await applyAlwaysOnTopNative(next);
			if (!res.supported) {
				alwaysOnTop = false;
				persistPrefs({ alwaysOnTop: false });
				toastState.show(i18n.t('sticky.alwaysOnTopWaylandHint'), 'info');
				return;
			}
			alwaysOnTop = res.active;
			persistPrefs({ alwaysOnTop: res.active });
			toastState.show(
				i18n.t(res.active ? 'sticky.alwaysOnTopOn' : 'sticky.alwaysOnTopOff'),
				'info'
			);
		} finally {
			togglingTop = false;
		}
	}

	async function handleToggleTask(taskIndex: number): Promise<void> {
		if (!view || !isOwner) return;
		try {
			const updated = await memoApi.toggleTask(view.memo.id, taskIndex);
			view = updated;
			draftContent = updated.memo.content;
		} catch (err) {
			toastState.show(err instanceof Error ? err.message : String(err), 'error');
		}
	}

	async function handleToggleReaction(emoji: string): Promise<void> {
		if (!view || !authState.isAuthenticated) return;
		try {
			const reactions = await memoApi.toggleReaction(view.memo.uid, emoji);
			view = { ...view, reactions };
		} catch (err) {
			toastState.show(err instanceof Error ? err.message : String(err), 'error');
		}
	}

	async function saveQuickEdit(): Promise<void> {
		if (!view || !isOwner || saving) return;
		const trimmed = draftContent.trim();
		if (!trimmed) return;
		saving = true;
		try {
			const updated = await memoApi.update({
				id: view.memo.id,
				content: trimmed
			});
			view = updated;
			draftContent = updated.memo.content;
			editing = false;
			toastState.show(i18n.t('common.saved'), 'success');
		} catch (err) {
			toastState.show(err instanceof Error ? err.message : String(err), 'error');
		} finally {
			saving = false;
		}
	}

	async function toggleCommentsPanel(): Promise<void> {
		if (!view) return;
		const next = !commentsOpen;
		commentsOpen = next;
		if (next && comments.length === 0 && view.commentCount > 0) {
			commentsLoading = true;
			try {
				comments = await memoApi.listComments(view.memo.id);
			} catch (err) {
				toastState.show(err instanceof Error ? err.message : String(err), 'error');
			} finally {
				commentsLoading = false;
			}
		}
	}

	function handleSelectTag(tag: string): void {
		void shortcutApi.focusMainRoute(`/?tag=${encodeURIComponent(tag)}`);
	}

	async function openInMainApp(): Promise<void> {
		if (!uid) return;
		await shortcutApi.focusMainMemo(uid);
	}

	async function closeSticky(): Promise<void> {
		if (isTauri()) {
			await getCurrentWindow().close();
		}
	}

	onMount(() => {
		const saved = loadStickyPrefs(uid);
		color = saved.color;
		alwaysOnTop = saved.alwaysOnTop;
		void loadMemo();

		if (!isTauri()) return;

		const win = getCurrentWindow();

		// Zero-flash native reveal strictly after Svelte paints the first real frame,
		// then re-apply Always on Top after window mapping (required by some X11/Wayland WMs)
		requestAnimationFrame(() => {
			requestAnimationFrame(() => {
				void win
					.show()
					.then(() => win.setFocus())
					.then(() => {
						if (saved.alwaysOnTop) {
							return applyAlwaysOnTopNative(true).then((r) => { if (!r.supported) alwaysOnTop = false; });
						}
					})
					.catch(() => {});
			});
		});

		let unlistenMoved: (() => void) | null = null;
		let unlistenResized: (() => void) | null = null;
		let geometryTimer: ReturnType<typeof setTimeout> | null = null;

		const scheduleGeometrySave = () => {
			if (geometryTimer) clearTimeout(geometryTimer);
			geometryTimer = setTimeout(async () => {
				try {
					const [pos, size] = await Promise.all([win.outerPosition(), win.innerSize()]);
					const factor = window.devicePixelRatio || 1;
					persistPrefs({
						x: Math.round(pos.x / factor),
						y: Math.round(pos.y / factor),
						width: Math.round(size.width / factor),
						height: Math.round(size.height / factor)
					});
				} catch {
					// Ignore Wayland/compositor position query limitations
				}
			}, 260);
		};

		void (async () => {
			try {
				unlistenMoved = await win.onMoved(scheduleGeometrySave);
				unlistenResized = await win.onResized(() => {
					void win
						.isMaximized()
						.then((isMax) => {
							if (isMax) {
								void win.unmaximize().catch(() => {});
								return;
							}
							scheduleGeometrySave();
						})
						.catch(() => scheduleGeometrySave());
				});
			} catch {
				// Ignore listener setup failures
			}
		})();

		return () => {
			if (geometryTimer) clearTimeout(geometryTimer);
			unlistenMoved?.();
			unlistenResized?.();
		};
	});
</script>

<div
	class="relative flex h-dvh w-full flex-col overflow-hidden border transition-colors duration-150 {palette.surface} {palette.border}"
>
	<!-- Crisp 3px MS Sticky Notes Top Accent Strip -->
	<div class="h-[3px] w-full shrink-0 transition-colors duration-150 {palette.accentBar}"></div>

	<!-- Official Memos 26.11 Top-Right Corner Fold Pin Indicator -->
	{#if view?.memo.pinned}
		<span
			class="pointer-events-none absolute top-[3px] right-0 z-10 h-3.5 w-3.5 bg-amber-400"
			style="clip-path: polygon(100% 0, 0 0, 100% 100%);"
			title={i18n.t('memo.pinned')}
			aria-label={i18n.t('memo.pinned')}
		></span>
	{/if}

	<!-- Konsta UI macOS 2026 Liquid Glass Draggable Header -->
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<header
		data-tauri-drag-region
		onmousedown={handleStickyDragMouseDown}
		class="xm-drag-region flex h-10 shrink-0 items-center justify-between gap-2 border-b px-2.5 shadow-[inset_0_1px_0_rgba(255,255,255,0.45)] dark:shadow-[inset_0_1px_0_rgba(255,255,255,0.08)] select-none transition-colors duration-150 {palette.header}"
	>
		<!-- 6 Curated macOS 2026 Color Swatches -->
		<div class="flex items-center gap-1.5">
			{#each COLOR_ORDER as id (id)}
				<button
					type="button"
					title={STICKY_THEMES[id].label}
					aria-label={STICKY_THEMES[id].label}
					aria-pressed={color === id}
					onclick={() => selectColor(id)}
					class="h-3.5 w-3.5 rounded-full shadow-[inset_0_1px_1px_rgba(255,255,255,0.45),0_1px_2px_rgba(0,0,0,0.2)] transition-transform duration-150 hover:scale-115 cursor-pointer {color === id
						? 'scale-110 ring-2 ring-[#0f172a]/55 ring-offset-1 ring-offset-transparent dark:ring-white/85'
						: 'opacity-75 hover:opacity-100'}"
					style="background-color: {STICKY_THEMES[id].swatch};"
				></button>
			{/each}
		</div>

		<!-- macOS Toolbar Action Controls (Always on Top pill + Edit + Open + Close) -->
		<div class="flex items-center gap-0.5">
			<button
				type="button"
				title={i18n.t('sticky.alwaysOnTop')}
				aria-label={i18n.t('sticky.alwaysOnTop')}
				aria-pressed={alwaysOnTop}
				disabled={togglingTop}
				onclick={toggleAlwaysOnTop}
				class="inline-flex h-7 items-center gap-1 rounded-lg px-2 text-[11px] font-semibold transition-colors cursor-pointer {alwaysOnTop
					? 'bg-[var(--xm-accent-btn)] text-white shadow-[0_1px_2px_rgba(0,0,0,0.2)]'
					: 'xm-text-secondary hover:bg-black/[0.07] dark:hover:bg-white/[0.10]'}"
			>
				{#if alwaysOnTop}
					<Pin class="h-3 w-3 shrink-0" aria-hidden="true" />
				{:else}
					<PinOff class="h-3 w-3 shrink-0" aria-hidden="true" />
				{/if}
				<span class="hidden sm:inline">{i18n.t('sticky.pinTopShort')}</span>
			</button>

			{#if isOwner}
				{#if editing}
					<button
						type="button"
						title={i18n.t('common.save')}
						aria-label={i18n.t('common.save')}
						disabled={saving}
						onclick={saveQuickEdit}
						class="flex h-7 w-7 items-center justify-center rounded-lg bg-[#047857]/14 text-[#047857] dark:bg-[#34d399]/18 dark:text-[#34d399] transition-colors hover:opacity-90 cursor-pointer"
					>
						<Check class="h-4 w-4" aria-hidden="true" />
					</button>
				{:else}
					<button
						type="button"
						title={i18n.t('memo.edit')}
						aria-label={i18n.t('memo.edit')}
						onclick={() => (editing = true)}
						class="flex h-7 w-7 items-center justify-center rounded-lg xm-text-secondary transition-colors hover:bg-black/[0.07] dark:hover:bg-white/[0.10] cursor-pointer"
					>
						<Pencil class="h-3.5 w-3.5" aria-hidden="true" />
					</button>
				{/if}
			{/if}

			<button
				type="button"
				title={i18n.t('sticky.openInApp')}
				aria-label={i18n.t('sticky.openInApp')}
				onclick={openInMainApp}
				class="flex h-7 w-7 items-center justify-center rounded-lg xm-text-secondary transition-colors hover:bg-black/[0.07] dark:hover:bg-white/[0.10] cursor-pointer"
			>
				<ExternalLink class="h-3.5 w-3.5" aria-hidden="true" />
			</button>

			<button
				type="button"
				title={i18n.t('common.close')}
				aria-label={i18n.t('common.close')}
				onclick={closeSticky}
				class="flex h-7 w-7 items-center justify-center rounded-lg xm-text-secondary transition-colors hover:bg-[#d70015]/15 hover:text-[#d70015] dark:hover:text-[#ff453a] cursor-pointer"
			>
				<X class="h-4 w-4" aria-hidden="true" />
			</button>
		</div>
	</header>

	<!-- Full Rich Memo Body (Markdown + Attachments + Location + Relations + Reactions) -->
	<main class="min-h-0 flex-1 overflow-y-auto overscroll-contain px-4 py-3.5">
		{#if loading}
			<p class="text-xs xm-text-muted">{i18n.t('common.loading')}</p>
		{:else if !view}
			<p class="text-xs xm-text-muted">{i18n.t('memo.notFound')}</p>
		{:else if editing}
			<textarea
				bind:value={draftContent}
				onkeydown={(e) => {
					if ((e.metaKey || e.ctrlKey) && e.key === 'Enter') {
						e.preventDefault();
						void saveQuickEdit();
					} else if (e.key === 'Escape') {
						editing = false;
						draftContent = view?.memo.content ?? '';
					}
				}}
				class="h-full min-h-[160px] w-full resize-none bg-transparent text-[14.5px] leading-relaxed text-inherit outline-none"
			></textarea>
		{:else}
			<div class="space-y-2.5">
				<MemoContent
					content={view.memo.content}
					canToggleTask={isOwner}
					onToggleTask={handleToggleTask}
					onSelectTag={handleSelectTag}
				/>

				{#if view.resources.length > 0}
					<AttachmentGrid resources={view.resources} compact />
				{/if}

				{#if view.memo.payload.location || referenceRelations.length > 0}
					<div class="space-y-1 pt-0.5 text-xs xm-text-muted">
						{#if view.memo.payload.location}
							<MemoLocationPreview
								location={view.memo.payload.location}
								memoUid={view.memo.uid}
							/>
						{/if}
						{#if referenceRelations.length > 0}
							<button
								type="button"
								class="flex items-center gap-1.5 hover:text-[var(--xm-text-primary)] hover:underline cursor-pointer"
								onclick={openInMainApp}
							>
								<Link2 class="h-3.5 w-3.5 shrink-0" aria-hidden="true" />
								<span class="tabular-nums">{referenceRelations.length} linked</span>
							</button>
						{/if}
					</div>
				{/if}

				{#if groupedReactions.length > 0 || view.commentCount > 0}
					<div class="flex flex-wrap items-center gap-1.5 pt-1">
						{#each groupedReactions as item (item.emoji)}
							<button
								type="button"
								onclick={() => handleToggleReaction(item.emoji)}
								class="inline-flex items-center gap-1 rounded-full border px-2 py-0.5 text-xs tabular-nums transition-colors cursor-pointer {item.reactedByMe
									? 'border-[var(--xm-accent-border)] bg-[var(--xm-accent-soft)] xm-text-accent font-semibold'
									: 'border-[var(--xm-divider)] bg-[var(--xm-surface-subtle)] xm-text-secondary'}"
							>
								<span>{item.emoji}</span>
								<span>{item.count}</span>
							</button>
						{/each}
						{#if view.commentCount > 0}
							<button
								type="button"
								aria-expanded={commentsOpen}
								onclick={toggleCommentsPanel}
								class="ml-auto inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-xs transition-colors tabular-nums cursor-pointer {commentsOpen
									? 'bg-[var(--xm-accent-soft)] xm-text-accent font-semibold'
									: 'xm-text-muted hover:bg-[var(--xm-surface-subtle)]'}"
							>
								<MessageCircle class="h-3.5 w-3.5" aria-hidden="true" />
								<span>{view.commentCount}</span>
							</button>
						{/if}
					</div>
				{/if}

				{#if commentsOpen && view.commentCount > 0}
					<div class="space-y-2 border-t pt-2.5 {palette.border}">
						{#if commentsLoading}
							<p class="text-xs xm-text-muted">{i18n.t('common.loading')}</p>
						{:else}
							{#each comments as comment (comment.memo.id)}
								<div class="rounded-xl border p-2.5 text-xs bg-[var(--xm-surface-subtle)] {palette.border}">
									<div class="mb-1 flex items-center justify-between gap-2 text-[11px] xm-text-muted">
										<span class="truncate font-semibold xm-text-primary">
											{comment.creator?.nickname || comment.creator?.username || 'Unknown'}
										</span>
										<span class="shrink-0 tabular-nums">{formatFullTimestamp(comment.memo.createdTs)}</span>
									</div>
									<MemoContent content={comment.memo.content} onSelectTag={handleSelectTag} />
								</div>
							{/each}
							<button
								type="button"
								onclick={openInMainApp}
								class="text-[11px] font-semibold xm-text-accent hover:underline cursor-pointer"
							>
								{i18n.t('sticky.openInApp')} →
							</button>
						{/if}
					</div>
				{/if}
			</div>
		{/if}
	</main>

	<!-- Refined Konsta macOS Footer -->
	{#if view}
		<footer
			class="flex shrink-0 items-center justify-between gap-2 border-t px-3.5 py-2 text-[11px] xm-text-muted {palette.border}"
		>
			<div class="flex min-w-0 items-center gap-1.5">
				{#if view.memo.visibility === 'PUBLIC'}
					<Globe class="h-3 w-3 shrink-0" aria-hidden="true" />
				{:else if view.memo.visibility === 'PROTECTED'}
					<Users class="h-3 w-3 shrink-0" aria-hidden="true" />
				{:else}
					<Lock class="h-3 w-3 shrink-0" aria-hidden="true" />
				{/if}
				<span class="truncate tabular-nums">{formatFullTimestamp(view.memo.createdTs)}</span>
			</div>
			<span class="inline-flex shrink-0 items-center gap-1.5 font-semibold xm-text-secondary">
				<AppIcon class="h-3.5 w-3.5 shrink-0" decorative />
				<span>Xmemos</span>
			</span>
		</footer>
	{/if}
</div>

<!-- Scoped interactive overlays for Sticky Note image lightbox & location mini-map -->
<ImageLightbox />
<LocationPreviewPopover />
