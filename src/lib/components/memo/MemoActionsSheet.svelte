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
	import { goto } from '$app/navigation';
	import {
		Archive,
		BookmarkPlus,
		Copy,
		ExternalLink,
		FileText,
		Image as ImageIcon,
		Pencil,
		Pin,
		PinOff,
		RotateCcw,
		StickyNote,
		Trash2
	} from 'lucide-svelte';

	import { i18n } from '#lib/i18n/index.svelte.ts';
	import { memoActionsState } from '#lib/state/memo-actions.svelte.ts';
	import { themeState } from '#lib/state/theme.svelte.ts';
	import { toastState } from '#lib/state/toast.svelte.ts';
	import { stripMarkdownToPlainText } from '#lib/utils/markdown.ts';
	import { renderMemoShareCardPng } from '#lib/utils/share-card.ts';
	import { saveBlobWithNativeDialog } from '#lib/utils/download.ts';
	import { buildMemoShortcutMeta, loadStickyPrefs, shortcutApi } from '#lib/api/shortcut.ts';
	import { isDesktopTauri } from '#lib/utils/platform.ts';

	const DESKTOP_MENU_WIDTH = 216;
	const VIEWPORT_MARGIN = 12;

	let menuEl = $state<HTMLDivElement | null>(null);
	let menuHeight = $state(280);

	$effect(() => {
		if (memoActionsState.mode === 'desktop' && memoActionsState.opened && menuEl) {
			menuHeight = menuEl.offsetHeight || 280;
		}
	});

	let desktopPos = $derived.by(() => {
		const anchor = memoActionsState.active?.anchorRect;
		if (!anchor || typeof window === 'undefined') {
			return { left: VIEWPORT_MARGIN, top: 64, flipUp: false };
		}
		const vw = window.innerWidth;
		const vh = window.innerHeight;
		const rawLeft = anchor.right - DESKTOP_MENU_WIDTH;
		const left = Math.max(VIEWPORT_MARGIN, Math.min(rawLeft, vw - DESKTOP_MENU_WIDTH - VIEWPORT_MARGIN));

		const spaceBelow = vh - anchor.bottom - VIEWPORT_MARGIN;
		const spaceAbove = anchor.top - VIEWPORT_MARGIN;
		const flipUp = spaceBelow < menuHeight + 8 && spaceAbove > spaceBelow;
		const top = flipUp
			? Math.max(VIEWPORT_MARGIN, anchor.top - menuHeight - 6)
			: Math.min(anchor.bottom + 6, Math.max(VIEWPORT_MARGIN, vh - menuHeight - VIEWPORT_MARGIN));

		return { left, top, flipUp };
	});

	async function copyText(raw: boolean): Promise<void> {
		const req = memoActionsState.active;
		if (!req) return;
		const text = raw ? req.view.memo.content : stripMarkdownToPlainText(req.view.memo.content);
		await navigator.clipboard.writeText(text);
		toastState.show(i18n.t(raw ? 'memo.copiedMarkdown' : 'memo.copiedPlain'), 'success');
		memoActionsState.close();
	}

	async function exportShareCard(): Promise<void> {
		const req = memoActionsState.active;
		if (!req) return;
		try {
			const blob = await renderMemoShareCardPng(req.view, themeState.isDark);
			const filename = `xmemos-${req.view.memo.uid.slice(0, 8)}.png`;
			const saved = await saveBlobWithNativeDialog(blob, filename, 'image/png');
			if (saved !== null) {
				toastState.show(i18n.t('memo.exportedCard'), 'success');
			}
		} catch (err) {
			toastState.show(
				i18n.t('memo.exportFailed', { error: err instanceof Error ? err.message : String(err) }),
				'error'
			);
		}
		memoActionsState.close();
	}

	async function openAsStickyNote(): Promise<void> {
		const req = memoActionsState.active;
		if (!req) return;
		try {
			const { title } = buildMemoShortcutMeta(req.view.memo.content);
			await shortcutApi.openStickyMemoWindow(req.view.memo.uid, title);
		} catch (err) {
			toastState.show(
				i18n.t('memo.shortcutFailed', {
					error: err instanceof Error ? err.message : String(err)
				}),
				'error'
			);
		}
		memoActionsState.close();
	}

	async function pinMemoShortcut(): Promise<void> {
		const req = memoActionsState.active;
		if (!req) return;
		const desktop = isDesktopTauri();
		try {
			const { title, preview } = buildMemoShortcutMeta(req.view.memo.content);
			const stickyPrefs = loadStickyPrefs(req.view.memo.uid);
			const res = await shortcutApi.createMemoShortcut({
				memoUid: req.view.memo.uid,
				title,
				preview,
				pinned: req.view.memo.pinned,
				color: stickyPrefs.color
			});
			if (res.created) {
				toastState.show(
					i18n.t(desktop ? 'memo.shortcutCreatedDesktop' : 'memo.shortcutCreatedAndroid'),
					'success'
				);
			} else if (res.unsupported) {
				toastState.show(res.message || i18n.t('memo.shortcutUnsupported'), 'info');
			} else if (res.message !== 'cancelled') {
				toastState.show(
					i18n.t('memo.shortcutFailed', { error: res.message || 'Unknown error' }),
					'error'
				);
			}
		} catch (err) {
			toastState.show(
				i18n.t('memo.shortcutFailed', {
					error: err instanceof Error ? err.message : String(err)
				}),
				'error'
			);
		}
		memoActionsState.close();
	}

	function handleWindowKeydown(event: KeyboardEvent): void {
		if (!memoActionsState.opened) return;
		if (event.key === 'Escape') {
			event.preventDefault();
			memoActionsState.close();
		}
	}
</script>

<svelte:window
	onkeydown={handleWindowKeydown}
	onresize={() => {
		if (memoActionsState.mode === 'desktop' && memoActionsState.opened) {
			memoActionsState.close();
		}
	}}
/>

{#if memoActionsState.active && memoActionsState.opened}
	{@const req = memoActionsState.active}
	{#if memoActionsState.mode === 'desktop'}
		<!-- Desktop (>= 1024px): Native macOS anchored floating menu -->
		<button
			type="button"
			aria-label={i18n.t('common.close')}
			class="fixed inset-0 z-[79] cursor-default bg-transparent"
			onclick={() => memoActionsState.close()}
		></button>
		<div
			bind:this={menuEl}
			role="menu"
			aria-label={i18n.t('memo.options')}
			class="xm-macos-menu fixed z-[80] w-[216px] rounded-[12px] p-1.5"
			style={`left: ${desktopPos.left}px; top: ${desktopPos.top}px; transform-origin: ${desktopPos.flipUp ? 'bottom right' : 'top right'};`}
		>
			<button
				type="button"
				role="menuitem"
				class="xm-macos-menu-item"
				onclick={() => {
					const uid = req.view.memo.uid;
					memoActionsState.close();
					void goto(`/memo/${uid}`);
				}}
			>
				<ExternalLink class="h-3.5 w-3.5 shrink-0 opacity-80" aria-hidden="true" />
				<span class="truncate">{i18n.t('memo.openDetails')}</span>
			</button>

			{#if req.canManage && req.onPin}
				<button
					type="button"
					role="menuitem"
					class="xm-macos-menu-item"
					onclick={() => {
						const fn = req.onPin;
						const v = req.view;
						memoActionsState.close();
						fn?.(v);
					}}
				>
					{#if req.view.memo.pinned}
						<PinOff class="h-3.5 w-3.5 shrink-0 opacity-80" aria-hidden="true" />
					{:else}
						<Pin class="h-3.5 w-3.5 shrink-0 opacity-80" aria-hidden="true" />
					{/if}
					<span class="truncate">{i18n.t(req.view.memo.pinned ? 'memo.unpin' : 'memo.pin')}</span>
				</button>
			{/if}

			{#if req.canManage && req.onEdit}
				<button
					type="button"
					role="menuitem"
					class="xm-macos-menu-item"
					onclick={() => {
						const fn = req.onEdit;
						const v = req.view;
						memoActionsState.close();
						fn?.(v);
					}}
				>
					<Pencil class="h-3.5 w-3.5 shrink-0 opacity-80" aria-hidden="true" />
					<span class="truncate">{i18n.t('memo.edit')}</span>
				</button>
			{/if}

			<div class="xm-macos-menu-sep" role="separator"></div>

			<button
				type="button"
				role="menuitem"
				class="xm-macos-menu-item"
				onclick={() => copyText(true)}
			>
				<Copy class="h-3.5 w-3.5 shrink-0 opacity-80" aria-hidden="true" />
				<span class="truncate">{i18n.t('memo.copyMarkdown')}</span>
			</button>

			<button
				type="button"
				role="menuitem"
				class="xm-macos-menu-item"
				onclick={() => copyText(false)}
			>
				<FileText class="h-3.5 w-3.5 shrink-0 opacity-80" aria-hidden="true" />
				<span class="truncate">{i18n.t('memo.copyPlainText')}</span>
			</button>

			<button
				type="button"
				role="menuitem"
				class="xm-macos-menu-item"
				onclick={exportShareCard}
			>
				<ImageIcon class="h-3.5 w-3.5 shrink-0 opacity-80" aria-hidden="true" />
				<span class="truncate">{i18n.t('memo.exportCard')}</span>
			</button>

			{#if isDesktopTauri()}
				<button
					type="button"
					role="menuitem"
					class="xm-macos-menu-item"
					onclick={openAsStickyNote}
				>
					<StickyNote class="h-3.5 w-3.5 shrink-0 opacity-80" aria-hidden="true" />
					<span class="truncate">{i18n.t('sticky.open')}</span>
				</button>
			{/if}

			<button
				type="button"
				role="menuitem"
				class="xm-macos-menu-item"
				onclick={pinMemoShortcut}
			>
				<BookmarkPlus class="h-3.5 w-3.5 shrink-0 opacity-80" aria-hidden="true" />
				<span class="truncate">
					{i18n.t(isDesktopTauri() ? 'memo.addToDesktop' : 'memo.pinToHome')}
				</span>
			</button>

			{#if req.canManage && (req.onRestore || req.onArchive || req.onDelete)}
				{@const hasRestoreOrArchive = Boolean(req.onRestore || req.onArchive)}
				{@const hasDelete = Boolean(req.onDelete)}
				<div class="xm-macos-menu-sep" role="separator"></div>

				{#if req.canManage && req.onRestore}
					<button
						type="button"
						role="menuitem"
						class="xm-macos-menu-item"
						onclick={() => {
							const fn = req.onRestore;
							const v = req.view;
							memoActionsState.close();
							fn?.(v);
						}}
					>
						<RotateCcw class="h-3.5 w-3.5 shrink-0 opacity-80" aria-hidden="true" />
						<span class="truncate">{i18n.t('common.restore')}</span>
					</button>
				{/if}

				{#if req.canManage && req.onArchive}
					<button
						type="button"
						role="menuitem"
						class="xm-macos-menu-item"
						onclick={() => {
							const fn = req.onArchive;
							const v = req.view;
							memoActionsState.close();
							fn?.(v);
						}}
					>
						<Archive class="h-3.5 w-3.5 shrink-0 opacity-80" aria-hidden="true" />
						<span class="truncate">{i18n.t('memo.archive')}</span>
					</button>
				{/if}

				{#if hasRestoreOrArchive && hasDelete}
					<div class="xm-macos-menu-sep" role="separator"></div>
				{/if}

				{#if req.canManage && req.onDelete}
					<button
						type="button"
						role="menuitem"
						class="xm-macos-menu-item xm-macos-menu-item-danger"
						onclick={() => {
							const fn = req.onDelete;
							const v = req.view;
							memoActionsState.close();
							fn?.(v);
						}}
					>
						<Trash2 class="h-3.5 w-3.5 shrink-0" aria-hidden="true" />
						<span class="truncate">{i18n.t('memo.delete')}</span>
					</button>
				{/if}
			{/if}
		</div>
	{:else}
		<!-- Mobile & Tablet (< 1024px): Ultra-lightweight 60/120fps GPU-composited iOS Action Sheet -->
		<div
			class="xm-action-sheet-root fixed right-0 left-0 z-[75] flex items-end justify-center px-2.5 pb-[max(0.75rem,env(safe-area-inset-bottom))]"
			style="top: var(--xm-vvt, 0px); height: var(--xm-vvh, 100dvh);"
			role="dialog"
			aria-modal="true"
			aria-label={i18n.t('memo.options')}
		>
			<button
				type="button"
				aria-label={i18n.t('common.close')}
				class="xm-action-sheet-scrim absolute inset-0"
				onclick={() => memoActionsState.close()}
			></button>

			<div class="xm-action-sheet-stack relative z-10 w-full max-w-md space-y-2">
				<div class="xm-action-sheet-card max-h-[72dvh] overflow-y-auto overscroll-contain">
					<div class="px-4 py-2.5 text-center text-[12px] font-semibold tracking-wide xm-text-muted">
						{i18n.t('memo.options')}
					</div>

					<button
						type="button"
						class="xm-action-sheet-btn"
						onclick={() => {
							const uid = req.view.memo.uid;
							memoActionsState.close();
							void goto(`/memo/${uid}`);
						}}
					>
						<ExternalLink class="h-[18px] w-[18px] shrink-0 opacity-80" aria-hidden="true" />
						<span>{i18n.t('memo.openDetails')}</span>
					</button>

					{#if req.canManage && req.onPin}
						<button
							type="button"
							class="xm-action-sheet-btn"
							onclick={() => {
								const fn = req.onPin;
								const v = req.view;
								memoActionsState.close();
								fn?.(v);
							}}
						>
							{#if req.view.memo.pinned}
								<PinOff class="h-[18px] w-[18px] shrink-0 opacity-80" aria-hidden="true" />
							{:else}
								<Pin class="h-[18px] w-[18px] shrink-0 opacity-80" aria-hidden="true" />
							{/if}
							<span>{i18n.t(req.view.memo.pinned ? 'memo.unpin' : 'memo.pin')}</span>
						</button>
					{/if}

					{#if req.canManage && req.onEdit}
						<button
							type="button"
							class="xm-action-sheet-btn"
							onclick={() => {
								const fn = req.onEdit;
								const v = req.view;
								memoActionsState.close();
								fn?.(v);
							}}
						>
							<Pencil class="h-[18px] w-[18px] shrink-0 opacity-80" aria-hidden="true" />
							<span>{i18n.t('memo.edit')}</span>
						</button>
					{/if}

					<button type="button" class="xm-action-sheet-btn" onclick={() => copyText(true)}>
						<Copy class="h-[18px] w-[18px] shrink-0 opacity-80" aria-hidden="true" />
						<span>{i18n.t('memo.copyMarkdown')}</span>
					</button>

					<button type="button" class="xm-action-sheet-btn" onclick={() => copyText(false)}>
						<FileText class="h-[18px] w-[18px] shrink-0 opacity-80" aria-hidden="true" />
						<span>{i18n.t('memo.copyPlainText')}</span>
					</button>

					<button type="button" class="xm-action-sheet-btn" onclick={exportShareCard}>
						<ImageIcon class="h-[18px] w-[18px] shrink-0 opacity-80" aria-hidden="true" />
						<span>{i18n.t('memo.exportCard')}</span>
					</button>

					<button type="button" class="xm-action-sheet-btn" onclick={pinMemoShortcut}>
						<BookmarkPlus class="h-[18px] w-[18px] shrink-0 opacity-80" aria-hidden="true" />
						<span>{i18n.t(isDesktopTauri() ? 'memo.addToDesktop' : 'memo.pinToHome')}</span>
					</button>

					{#if req.canManage && req.onRestore}
						<button
							type="button"
							class="xm-action-sheet-btn"
							onclick={() => {
								const fn = req.onRestore;
								const v = req.view;
								memoActionsState.close();
								fn?.(v);
							}}
						>
							<RotateCcw class="h-[18px] w-[18px] shrink-0 opacity-80" aria-hidden="true" />
							<span>{i18n.t('common.restore')}</span>
						</button>
					{/if}

					{#if req.canManage && req.onArchive}
						<button
							type="button"
							class="xm-action-sheet-btn"
							onclick={() => {
								const fn = req.onArchive;
								const v = req.view;
								memoActionsState.close();
								fn?.(v);
							}}
						>
							<Archive class="h-[18px] w-[18px] shrink-0 opacity-80" aria-hidden="true" />
							<span>{i18n.t('memo.archive')}</span>
						</button>
					{/if}

					{#if req.canManage && req.onDelete}
						<button
							type="button"
							class="xm-action-sheet-btn xm-action-sheet-btn-danger"
							onclick={() => {
								const fn = req.onDelete;
								const v = req.view;
								memoActionsState.close();
								fn?.(v);
							}}
						>
							<Trash2 class="h-[18px] w-[18px] shrink-0" aria-hidden="true" />
							<span>{i18n.t('memo.delete')}</span>
						</button>
					{/if}
				</div>

				<button
					type="button"
					class="xm-action-sheet-cancel"
					onclick={() => memoActionsState.close()}
				>
					{i18n.t('common.cancel')}
				</button>
			</div>
		</div>
	{/if}
{/if}
