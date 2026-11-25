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
	import { Chip } from 'konsta/svelte';
	import {
		Globe,
		Link2,
		Lock,
		MessageCircle,
		MoreHorizontal,
		RotateCcw,
		SmilePlus,
		Users
	} from 'lucide-svelte';

	import type { MemoView } from '#lib/types/memo.ts';
	import { i18n } from '#lib/i18n/index.svelte.ts';
	import { authState } from '#lib/state/auth.svelte.ts';
	import { memoActionsState } from '#lib/state/memo-actions.svelte.ts';
	import { themeState } from '#lib/state/theme.svelte.ts';
	import { formatFullTimestamp, formatRelativeTime } from '#lib/utils/date.ts';
	import MemoContent from './MemoContent.svelte';
	import AttachmentGrid from '#lib/components/media/AttachmentGrid.svelte';
	import MemoLocationPreview from '#lib/components/map/MemoLocationPreview.svelte';

	interface Props {
		view: MemoView;
		highlightTerms?: string[];
		onEdit?: (view: MemoView) => void;
		onPin?: (view: MemoView) => void;
		onArchive?: (view: MemoView) => void;
		onRestore?: (view: MemoView) => void;
		onDelete?: (view: MemoView) => void;
		onToggleTask?: (memoId: number, taskIndex: number) => void;
		onToggleReaction?: (view: MemoView, emoji: string) => void;
		onSelectTag?: (tag: string) => void;
	}

	let {
		view,
		highlightTerms = [],
		onEdit,
		onPin,
		onArchive,
		onRestore,
		onDelete,
		onToggleTask,
		onToggleReaction,
		onSelectTag
	}: Props = $props();

	let reactionPickerOpened = $state(false);

	const QUICK_EMOJIS = ['👍', '❤️', '🔥', '🎉', '💡', '👀'];

	let isOwner = $derived(
		authState.currentUser !== null && authState.currentUser.id === view.memo.creatorId
	);
	let canManage = $derived(isOwner);
	let isMenuOpen = $derived(
		memoActionsState.opened && memoActionsState.active?.view.memo.id === view.memo.id
	);
	let referenceRelations = $derived(
		view.relations.filter((r) => r.type === 'REFERENCE')
	);
	let visibilityLabel = $derived(
		view.memo.visibility === 'PUBLIC'
			? i18n.t('visibility.public')
			: view.memo.visibility === 'PROTECTED'
				? i18n.t('visibility.protected')
				: i18n.t('visibility.private')
	);

	let groupedReactions = $derived.by(() => {
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

	function openActions(event: MouseEvent): void {
		const rect = (event.currentTarget as HTMLElement | null)?.getBoundingClientRect();
		memoActionsState.open({
			view,
			canManage,
			anchorRect: rect
				? {
						top: rect.top,
						bottom: rect.bottom,
						left: rect.left,
						right: rect.right,
						width: rect.width,
						height: rect.height
					}
				: undefined,
			onEdit,
			onPin,
			onArchive,
			onRestore,
			onDelete
		});
	}
</script>

<article
	class="@container glass-surface relative overflow-hidden rounded-2xl {themeState.density === 'compact' ? 'p-3.5' : 'p-4'}"
>
	<!-- Official Memos 26.11 Top-Right Corner Fold Pin Indicator -->
	{#if view.memo.pinned}
		<span
			class="pointer-events-none absolute top-0 right-0 h-4 w-4 bg-amber-400 dark:bg-amber-400/95"
			style="clip-path: polygon(100% 0, 0 0, 100% 100%);"
			title={i18n.t('memo.pinned')}
			aria-label={i18n.t('memo.pinned')}
		></span>
	{/if}

	<!-- Adaptive Container-Aware Header: inline on wide cards, stacked 2-line on narrow multi-column cards (zero overlap) -->
	<div class="mb-2.5 flex items-center justify-between gap-2">
		<div class="flex min-w-0 flex-1 items-center gap-2 overflow-hidden">
			<button
				type="button"
				aria-label="View profile of {view.creator?.nickname || view.creator?.username || 'Unknown'}"
				class="flex h-7 w-7 shrink-0 items-center justify-center rounded-full border border-[var(--xm-accent-border)] bg-[var(--xm-accent-soft)] text-[11px] font-bold text-[var(--xm-accent)]"
				onclick={() => view.creator && goto(`/u/${encodeURIComponent(view.creator.username)}`)}
			>
				{(view.creator?.nickname || view.creator?.username || 'U').slice(0, 1).toUpperCase()}
			</button>
			<div
				class="flex min-w-0 flex-1 flex-col text-xs @[340px]:flex-row @[340px]:items-center @[340px]:gap-1.5"
			>
				<button
					type="button"
					class="max-w-full truncate text-left leading-tight font-semibold xm-text-primary hover:underline"
					onclick={() => view.creator && goto(`/u/${encodeURIComponent(view.creator.username)}`)}
				>
					{view.creator?.nickname || view.creator?.username || 'Unknown'}
				</button>
				<span class="hidden shrink-0 xm-text-muted @[340px]:inline" aria-hidden="true">·</span>
				<button
					type="button"
					title={formatFullTimestamp(view.memo.createdTs)}
					class="max-w-full truncate text-left text-[11px] leading-tight xm-text-muted tabular-nums hover:underline @[340px]:text-xs"
					onclick={() => goto(`/memo/${view.memo.uid}`)}
				>
					{formatRelativeTime(view.memo.createdTs)}
				</button>
			</div>
		</div>

		<div class="flex shrink-0 items-center gap-0.5">
			<span
				class="flex h-7 w-7 items-center justify-center rounded-full xm-text-muted"
				title={visibilityLabel}
				aria-label={visibilityLabel}
			>
				{#if view.memo.visibility === 'PUBLIC'}
					<Globe class="h-3.5 w-3.5" aria-hidden="true" />
				{:else if view.memo.visibility === 'PROTECTED'}
					<Users class="h-3.5 w-3.5" aria-hidden="true" />
				{:else}
					<Lock class="h-3.5 w-3.5" aria-hidden="true" />
				{/if}
			</span>
			<button
				type="button"
				aria-label={i18n.t('memo.options')}
				aria-haspopup="menu"
				aria-expanded={isMenuOpen}
				class="flex h-8 w-8 items-center justify-center rounded-full transition-colors cursor-pointer {isMenuOpen
					? 'bg-[var(--xm-surface-subtle)] xm-text-primary'
					: 'xm-text-secondary hover:bg-[var(--xm-surface-subtle)]'}"
				onclick={openActions}
			>
				<MoreHorizontal class="h-4 w-4" aria-hidden="true" />
			</button>
		</div>
	</div>

	<!-- Body -->
	<MemoContent
		content={view.memo.content}
		{highlightTerms}
		canToggleTask={isOwner}
		onToggleTask={(idx) => onToggleTask?.(view.memo.id, idx)}
		{onSelectTag}
	/>

	<!-- Attachments -->
	<AttachmentGrid resources={view.resources} />

	<!-- Location & Related Memos Metadata (Official Memos 2026 style with mini-map preview popover) -->
	{#if view.memo.payload.location || referenceRelations.length > 0}
		<div class="mt-2.5 space-y-1 text-xs xm-text-muted">
			{#if view.memo.payload.location}
				<MemoLocationPreview location={view.memo.payload.location} memoUid={view.memo.uid} />
			{/if}
			{#if referenceRelations.length > 0}
				<button
					type="button"
					class="flex items-center gap-1.5 hover:text-[var(--xm-text-primary)] hover:underline"
					onclick={() => goto(`/memo/${view.memo.uid}`)}
				>
					<Link2 class="h-3.5 w-3.5 shrink-0" aria-hidden="true" />
					<span class="tabular-nums">{referenceRelations.length} linked</span>
				</button>
			{/if}
		</div>
	{/if}

	<!-- Footer: Reactions & Comment Count -->
	<div
		class="mt-3 flex flex-wrap items-center gap-1.5 border-t border-[var(--xm-divider)] pt-2.5"
	>
		{#each groupedReactions as item (item.emoji)}
			<Chip
				outline={!item.reactedByMe}
				role="button"
				aria-pressed={item.reactedByMe}
				aria-label="Reaction {item.emoji}, {item.count}"
				class="cursor-pointer !text-xs tabular-nums"
				onclick={() => onToggleReaction?.(view, item.emoji)}
			>
				{item.emoji} {item.count}
			</Chip>
		{/each}

		{#if authState.isAuthenticated && onToggleReaction}
			<button
				type="button"
				aria-label="Toggle reaction picker"
				aria-expanded={reactionPickerOpened}
				class="flex h-8 w-8 items-center justify-center rounded-full xm-text-secondary transition-colors hover:bg-[var(--xm-surface-subtle)]"
				onclick={() => (reactionPickerOpened = !reactionPickerOpened)}
			>
				<SmilePlus class="h-4 w-4" aria-hidden="true" />
			</button>
		{/if}

		{#if isOwner && onRestore}
			<button
				type="button"
				class="inline-flex h-7 items-center gap-1 rounded-full border border-[var(--xm-accent-border)] bg-[var(--xm-accent-soft)] px-2.5 text-xs font-semibold xm-text-accent transition hover:opacity-90 cursor-pointer"
				onclick={() => onRestore(view)}
			>
				<RotateCcw class="h-3 w-3 shrink-0" aria-hidden="true" />
				<span>{i18n.t('common.restore')}</span>
			</button>
		{/if}

		<button
			type="button"
			aria-label="Open details and {view.commentCount} comments"
			class="ml-auto inline-flex min-h-[32px] items-center gap-1.5 rounded-full px-2.5 py-1 text-xs font-medium xm-text-secondary transition-colors hover:bg-[var(--xm-surface-subtle)] tabular-nums"
			onclick={() => goto(`/memo/${view.memo.uid}`)}
		>
			<MessageCircle class="h-3.5 w-3.5" aria-hidden="true" />
			<span>{view.commentCount}</span>
		</button>
	</div>

	{#if reactionPickerOpened}
		<div
			role="group"
			aria-label="Quick reactions"
			class="mt-2 flex flex-wrap gap-1 rounded-xl border border-[var(--xm-divider)] bg-[var(--xm-surface-subtle)] p-1.5"
		>
			{#each QUICK_EMOJIS as emoji (emoji)}
				<button
					type="button"
					aria-label="React with {emoji}"
					class="flex h-9 w-9 items-center justify-center rounded-lg text-base transition-transform active:scale-90 hover:bg-[var(--xm-surface-hover)]"
					onclick={() => {
						onToggleReaction?.(view, emoji);
						reactionPickerOpened = false;
					}}
				>
					{emoji}
				</button>
			{/each}
		</div>
	{/if}
</article>
