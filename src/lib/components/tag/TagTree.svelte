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
	import {
		Check,
		ChevronDown,
		ChevronRight,
		Hash,
		ListTree,
		MoreHorizontal,
		Pencil,
		Trash2
	} from 'lucide-svelte';

	import { i18n } from '#lib/i18n/index.svelte.ts';
	import { tagViewState } from '#lib/state/tag-view.svelte.ts';
	import type { TagCount } from '#lib/types/memo.ts';
	import { buildTagTree, type TagTreeNode } from '#lib/utils/tag.ts';

	interface Props {
		tags: TagCount[];
		selectedTag?: string | null;
		title?: string;
		onSelectTag?: (tag: string | null) => void;
		onRenameTag?: (tag: string) => void;
		onDeleteTag?: (tag: string) => void;
	}

	let {
		tags,
		selectedTag = null,
		title,
		onSelectTag,
		onRenameTag,
		onDeleteTag
	}: Props = $props();

	let menuOpened = $state(false);
	let menuContainerEl = $state<HTMLDivElement | null>(null);
	let collapsedPaths = $state<Set<string>>(new Set());

	let tree = $derived(buildTagTree(tags));

	let flatTags = $derived.by(() =>
		[...tags].sort((a, b) => b.count - a.count || a.name.localeCompare(b.name))
	);

	function toggleCollapse(path: string): void {
		const next = new Set(collapsedPaths);
		if (next.has(path)) next.delete(path);
		else next.add(path);
		collapsedPaths = next;
	}

	function handleWindowClick(event: MouseEvent): void {
		if (!menuOpened || !menuContainerEl) return;
		if (!menuContainerEl.contains(event.target as Node)) {
			menuOpened = false;
		}
	}

	function handleWindowKeydown(event: KeyboardEvent): void {
		if (event.key === 'Escape' && menuOpened) {
			menuOpened = false;
		}
	}
</script>

<svelte:window onclick={handleWindowClick} onkeydown={handleWindowKeydown} />

{#snippet renderNode(node: TagTreeNode, depth: number)}
	{@const hasChildren = node.children.length > 0}
	{@const isCollapsed = collapsedPaths.has(node.fullPath)}
	<div style="padding-left: {depth * 14}px">
		<div
			class="group flex min-h-[36px] items-center justify-between rounded-xl px-2 py-1 text-sm transition-colors {selectedTag === node.fullPath
				? 'bg-[var(--xm-accent-soft)] text-[var(--xm-accent)] font-semibold'
				: 'xm-text-primary hover:bg-[var(--xm-surface-subtle)]'}"
		>
			{#if hasChildren}
				<button
					type="button"
					aria-label={isCollapsed ? `Expand #${node.fullPath}` : `Collapse #${node.fullPath}`}
					class="mr-0.5 flex h-6 w-6 shrink-0 items-center justify-center rounded-md xm-text-muted hover:bg-[var(--xm-surface-hover)]"
					onclick={() => toggleCollapse(node.fullPath)}
				>
					{#if isCollapsed}
						<ChevronRight class="h-3.5 w-3.5" aria-hidden="true" />
					{:else}
						<ChevronDown class="h-3.5 w-3.5" aria-hidden="true" />
					{/if}
				</button>
			{/if}

			<button
				type="button"
				aria-pressed={selectedTag === node.fullPath}
				class="flex min-w-0 flex-1 items-center gap-1.5 text-left"
				onclick={() => onSelectTag?.(selectedTag === node.fullPath ? null : node.fullPath)}
			>
				<Hash class="h-3.5 w-3.5 shrink-0 xm-text-muted" aria-hidden="true" />
				<span class="truncate">{node.segment}</span>
				<span class="ml-auto text-xs xm-text-muted tabular-nums">{node.count}</span>
			</button>

			{#if onRenameTag || onDeleteTag}
				<div class="ml-2 flex items-center gap-1">
					{#if onRenameTag}
						<button
							type="button"
							aria-label="Rename tag #{node.fullPath}"
							class="flex h-7 w-7 items-center justify-center rounded-lg xm-text-secondary hover:bg-[var(--xm-surface-hover)]"
							onclick={() => onRenameTag(node.fullPath)}
						>
							<Pencil class="h-3.5 w-3.5" aria-hidden="true" />
						</button>
					{/if}
					{#if onDeleteTag}
						<button
							type="button"
							aria-label="Delete tag #{node.fullPath}"
							class="flex h-7 w-7 items-center justify-center rounded-lg xm-text-danger hover:bg-[var(--xm-danger-soft)]"
							onclick={() => onDeleteTag(node.fullPath)}
						>
							<Trash2 class="h-3.5 w-3.5" aria-hidden="true" />
						</button>
					{/if}
				</div>
			{/if}
		</div>
		{#if hasChildren && !isCollapsed}
			{#each node.children as child (child.fullPath)}
				{@render renderNode(child, depth + 1)}
			{/each}
		{/if}
	</div>
{/snippet}

<section class="glass-surface rounded-2xl p-3" aria-label={title ?? i18n.t('tags.title')}>
	<div class="mb-2 flex items-center justify-between gap-2 px-1 text-xs font-semibold">
		<span class="xm-text-secondary">{title ?? i18n.t('tags.title')}</span>

		<div class="flex items-center gap-1">
			{#if selectedTag}
				<button
					type="button"
					class="rounded-md px-1.5 py-0.5 text-xs font-semibold xm-text-accent hover:bg-[var(--xm-accent-soft)]"
					onclick={() => onSelectTag?.(null)}
				>
					{i18n.t('tags.clearFilter')}
				</button>
			{/if}

			<!-- Official Memos Tags Options Popover (Show as tree toggle) -->
			<div bind:this={menuContainerEl} class="relative">
				<button
					type="button"
					aria-haspopup="menu"
					aria-expanded={menuOpened}
					aria-label={i18n.t('tags.options')}
					class="flex h-6 w-6 items-center justify-center rounded-md xm-text-muted transition-colors hover:bg-[var(--xm-surface-subtle)] hover:text-[var(--xm-text-primary)]"
					onclick={() => (menuOpened = !menuOpened)}
				>
					<MoreHorizontal class="h-3.5 w-3.5" aria-hidden="true" />
				</button>

				{#if menuOpened}
					<div
						role="menu"
						class="glass-surface absolute top-full right-0 z-30 mt-1 w-44 rounded-xl border border-[var(--xm-divider)] p-1.5 shadow-xl"
					>
						<div class="px-2 py-1 text-[11px] font-semibold xm-text-muted">
							{i18n.t('tags.title')}
						</div>
						<button
							type="button"
							role="menuitemcheckbox"
							aria-checked={tagViewState.showAsTree}
							class="flex w-full items-center gap-2 rounded-lg px-2 py-1.5 text-left text-xs font-medium xm-text-primary transition-colors hover:bg-[var(--xm-surface-subtle)]"
							onclick={() => {
								tagViewState.toggleTree();
								menuOpened = false;
							}}
						>
							<ListTree class="h-3.5 w-3.5 shrink-0 xm-text-secondary" aria-hidden="true" />
							<span class="flex-1">{i18n.t('tags.showAsTree')}</span>
							{#if tagViewState.showAsTree}
								<Check class="h-3.5 w-3.5 shrink-0 xm-text-accent" aria-hidden="true" />
							{/if}
						</button>
					</div>
				{/if}
			</div>
		</div>
	</div>

	{#if tags.length === 0}
		<p class="px-1 py-2 text-xs xm-text-muted">{i18n.t('tags.empty')}</p>
	{:else if tagViewState.showAsTree}
		<div class="space-y-0.5">
			{#each tree as node (node.fullPath)}
				{@render renderNode(node, 0)}
			{/each}
		</div>
	{:else}
		<div class="space-y-0.5">
			{#each flatTags as item (item.name)}
				{@const segments = item.name.split('/').filter(Boolean)}
				<div
					class="group flex min-h-[36px] items-center justify-between rounded-xl px-2.5 py-1 text-sm transition-colors {selectedTag === item.name
						? 'bg-[var(--xm-accent-soft)] text-[var(--xm-accent)] font-semibold'
						: 'xm-text-primary hover:bg-[var(--xm-surface-subtle)]'}"
				>
					<button
						type="button"
						aria-pressed={selectedTag === item.name}
						class="flex min-w-0 flex-1 items-center gap-1.5 text-left"
						onclick={() => onSelectTag?.(selectedTag === item.name ? null : item.name)}
					>
						<Hash class="h-3.5 w-3.5 shrink-0 xm-text-muted" aria-hidden="true" />
						<span class="truncate">
							{#each segments as seg, idx (idx)}
								{#if idx > 0}
									<span class="mx-0.5 xm-text-muted">/</span>
								{/if}
								<span>{seg}</span>
							{/each}
						</span>
						<span class="ml-auto text-xs xm-text-muted tabular-nums">{item.count}</span>
					</button>
					{#if onRenameTag || onDeleteTag}
						<div class="ml-2 flex items-center gap-1">
							{#if onRenameTag}
								<button
									type="button"
									aria-label="Rename tag #{item.name}"
									class="flex h-7 w-7 items-center justify-center rounded-lg xm-text-secondary hover:bg-[var(--xm-surface-hover)]"
									onclick={() => onRenameTag(item.name)}
								>
									<Pencil class="h-3.5 w-3.5" aria-hidden="true" />
								</button>
							{/if}
							{#if onDeleteTag}
								<button
									type="button"
									aria-label="Delete tag #{item.name}"
									class="flex h-7 w-7 items-center justify-center rounded-lg xm-text-danger hover:bg-[var(--xm-danger-soft)]"
									onclick={() => onDeleteTag(item.name)}
								>
									<Trash2 class="h-3.5 w-3.5" aria-hidden="true" />
								</button>
							{/if}
						</div>
					{/if}
				</div>
			{/each}
		</div>
	{/if}
</section>
