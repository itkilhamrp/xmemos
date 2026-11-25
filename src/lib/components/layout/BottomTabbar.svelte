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
	import { page } from '$app/state';
	import { Tabbar, TabbarLink, ToolbarPane } from 'konsta/svelte';
	import { CalendarDays, Compass, FolderOpen, Home, Settings } from 'lucide-svelte';
	import { i18n, type TranslationKey } from '#lib/i18n/index.svelte.ts';

	const NAV_ITEMS: readonly { path: string; labelKey: TranslationKey; icon: typeof Home }[] = [
		{ path: '/', labelKey: 'nav.home', icon: Home },
		{ path: '/explore', labelKey: 'nav.explore', icon: Compass },
		{ path: '/review', labelKey: 'nav.review', icon: CalendarDays },
		{ path: '/resources', labelKey: 'nav.media', icon: FolderOpen },
		{ path: '/settings', labelKey: 'nav.settings', icon: Settings }
	];

	let currentPath = $derived(page.url.pathname);
	let visible = $derived(currentPath !== '/auth');

	function isNavActive(itemPath: string): boolean {
		if (itemPath === '/') return currentPath === '/';
		return currentPath === itemPath || currentPath.startsWith(`${itemPath}/`);
	}

	function handleSelect(itemPath: string): void {
		if (currentPath === itemPath) {
			if (typeof window !== 'undefined') {
				window.scrollTo({ top: 0, behavior: 'smooth' });
			}
			return;
		}
		void goto(itemPath);
	}
</script>

{#if visible}
	<!-- Persistent root-level Konsta UI v5 iOS 26 Tabbar so slider glides smoothly without resetting to Home -->
	<Tabbar
		labels
		icons
		class="fixed right-0 bottom-0 left-0 z-30 md:hidden"
		aria-label={i18n.t('nav.primaryNav')}
	>
		<ToolbarPane>
			{#each NAV_ITEMS as item (item.path)}
				{@const IconComp = item.icon}
				{@const active = isNavActive(item.path)}
				<TabbarLink
					{active}
					aria-current={active ? 'page' : undefined}
					onclick={() => handleSelect(item.path)}
				>
					{#snippet icon()}
						<IconComp class="h-5 w-5" aria-hidden="true" />
					{/snippet}
					{#snippet label()}
						{i18n.t(item.labelKey)}
					{/snippet}
				</TabbarLink>
			{/each}
		</ToolbarPane>
	</Tabbar>
{/if}
