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
	import type { Snippet } from 'svelte';
	import { page } from '$app/state';
	import { goto } from '$app/navigation';
	import {
		Button,
		Fab,
		Link,
		Navbar,
		NavbarBackLink,
		Page,
		Panel
	} from 'konsta/svelte';
	import {
		Archive,
		Bell,
		CalendarDays,
		ChevronLeft,
		Compass,
		FolderOpen,
		Home,
		Map as MapIcon,
		Menu,
		PanelLeft,
		PanelRight,
		Plus,
		Settings,
		User as UserIcon,
		X
	} from 'lucide-svelte';

	import { i18n, type TranslationKey } from '#lib/i18n/index.svelte.ts';
	import { authState } from '#lib/state/auth.svelte.ts';
	import { desktopLayoutState } from '#lib/state/desktop-layout.svelte.ts';
	import { feedDisplayState } from '#lib/state/feed-display.svelte.ts';
	import { filterState } from '#lib/state/filters.svelte.ts';
	import { memosState } from '#lib/state/memos.svelte.ts';
	import { shortcutsState } from '#lib/state/shortcuts.svelte.ts';
	import ActivityHeatmap from '#lib/components/calendar/ActivityHeatmap.svelte';
	import TagTree from '#lib/components/tag/TagTree.svelte';
	import AccountSwitcherSheet from '#lib/components/auth/AccountSwitcherSheet.svelte';
	import MemoEditor from '#lib/components/memo/MemoEditor.svelte';
	import DesktopWindowControls from '#lib/components/layout/DesktopWindowControls.svelte';
	import AppIcon from '#lib/components/ui/AppIcon.svelte';
	import { handleTitlebarMouseDown, isDesktopTauri } from '#lib/utils/platform.ts';

	interface Props {
		title: string;
		subtitle?: string;
		backHref?: string;
		backLabel?: string;
		showFab?: boolean;
		showRightRail?: boolean;
		onCompose?: () => void;
		onSelectTag?: (tag: string | null) => void;
		onSelectDate?: (date: string | null) => void;
		desktopLeftSearch?: Snippet;
		desktopLeftControls?: Snippet;
		desktopLeftTags?: Snippet;
		children: Snippet;
	}

	let {
		title,
		subtitle,
		backHref,
		backLabel,
		showFab = true,
		showRightRail,
		onCompose,
		onSelectTag,
		onSelectDate,
		desktopLeftSearch,
		desktopLeftControls,
		desktopLeftTags,
		children
	}: Props = $props();

	let drawerOpened = $state(false);
	let switcherOpened = $state(false);
	let quickComposeOpened = $state(false);
	const desktopWindow = isDesktopTauri();

	let currentPath = $derived(page.url.pathname);

	// Top-level sidebar routes do not need a redundant Back button on Tablet/Desktop where the left rail is visible
	let isTopLevelSidebarRoute = $derived(
		currentPath === '/inbox' || currentPath === '/archived'
	);

	// Smart context-aware right rail / macOS Inspector: shown on timeline/feed routes
	let isSettingsRoute = $derived(currentPath === '/settings' || currentPath.startsWith('/settings/'));
	let isFeedRoute = $derived(currentPath === '/' || currentPath === '/explore');
	let isWideRoute = $derived(currentPath === '/resources');
	let isFullCanvasRoute = $derived(currentPath === '/review' || currentPath === '/map');
	let isExpandedFeed = $derived(isFeedRoute && feedDisplayState.isWideDesktopFeed);
	let rightRailVisible = $derived(
		showRightRail ??
			!(
				isSettingsRoute ||
				currentPath === '/review' ||
				currentPath === '/map' ||
				currentPath === '/resources' ||
				currentPath.startsWith('/u/')
			)
	);

	// Desktop Left Rail hosts Tags on full-canvas routes (/review & /map); Home & Explore host Tags in Right Inspector Rail
	let showDesktopLeftTags = $derived(Boolean(desktopLeftTags) || currentPath === '/review');

	// Adaptive main column width: comfortable reading column on List, full-canvas on Map/Review/Masonry
	let mainWidthClass = $derived(
		isFullCanvasRoute
			? 'w-full'
			: isExpandedFeed
				? 'mx-auto w-full max-w-[1680px]'
				: rightRailVisible
					? 'mx-auto w-full max-w-[680px] lg:max-w-[700px]'
					: isWideRoute
						? 'mx-auto w-full max-w-5xl'
						: 'mx-auto w-full max-w-3xl'
	);

	const WORKSPACE_NAV_ITEMS: readonly {
		path: string;
		labelKey: TranslationKey;
		icon: typeof Home;
	}[] = [
		{ path: '/', labelKey: 'nav.home', icon: Home },
		{ path: '/explore', labelKey: 'nav.explore', icon: Compass },
		{ path: '/review', labelKey: 'nav.review', icon: CalendarDays },
		{ path: '/map', labelKey: 'nav.map', icon: MapIcon },
		{ path: '/resources', labelKey: 'nav.media', icon: FolderOpen }
	];

	const LIBRARY_NAV_ITEMS: readonly {
		path: string;
		labelKey: TranslationKey;
		icon: typeof Bell;
	}[] = [
		{ path: '/inbox', labelKey: 'nav.inbox', icon: Bell },
		{ path: '/archived', labelKey: 'nav.archived', icon: Archive },
		{ path: '/settings', labelKey: 'nav.settings', icon: Settings }
	];

	// Mobile/Tablet combined lists (unchanged for < 1024px)
	const TABLET_NAV_ITEMS = [...WORKSPACE_NAV_ITEMS, ...LIBRARY_NAV_ITEMS];
	const MOBILE_EXTRA_ITEMS = [
		...WORKSPACE_NAV_ITEMS.filter((item) => item.path === '/map'),
		...LIBRARY_NAV_ITEMS.filter((item) => item.path !== '/settings')
	];

	function isNavActive(itemPath: string): boolean {
		if (itemPath === '/') return currentPath === '/';
		return currentPath === itemPath || currentPath.startsWith(`${itemPath}/`);
	}

	function handleSidebarCompose(): void {
		if (onCompose) {
			onCompose();
		} else {
			quickComposeOpened = true;
		}
	}

	function handleTagSelect(tag: string | null): void {
		if (onSelectTag) {
			onSelectTag(tag);
		} else {
			filterState.selectedTag = tag;
			void goto('/');
		}
	}

	function handleDateSelect(date: string | null): void {
		if (onSelectDate) {
			onSelectDate(date);
		} else {
			filterState.selectedDate = date;
			void goto('/');
		}
	}

	$effect(() => {
		return shortcutsState.registerCompose(handleSidebarCompose);
	});
</script>

<a
	href="#main-content"
	class="sr-only focus:not-sr-only focus:fixed focus:top-3 focus:left-3 focus:z-[60] focus:rounded-xl focus:bg-[var(--xm-accent)] focus:px-4 focus:py-2 focus:text-sm focus:font-semibold focus:text-white focus:shadow-lg"
>
	{i18n.t('nav.skipToContent')}
</a>

<Page class="!bg-transparent md:!flex md:!h-dvh md:!flex-col md:!overflow-hidden lg:!flex-row">
	{#if desktopWindow}
		<!-- Narrow Desktop Window (< 1024px on Linux/macOS/Windows): Integrated macOS Liquid Glass Titlebar Strip -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			data-tauri-drag-region
			onmousedown={handleTitlebarMouseDown}
			class="xm-desktop-strip xm-drag-region sticky top-0 z-40 flex h-[34px] shrink-0 items-center justify-between px-3 lg:hidden"
		>
			<DesktopWindowControls compact />
			<span
				data-tauri-drag-region
				class="pointer-events-none truncate text-[12px] font-semibold tracking-[-0.01em] xm-text-secondary"
			>
				Xmemos
			</span>
			<span class="w-[50px] shrink-0" aria-hidden="true"></span>
		</div>
	{/if}

	<!-- Mobile & Tablet/iPad (< 1024px): Native Konsta UI v5 iOS Navbar (100% preserved) -->
	<Navbar
		{title}
		{subtitle}
		centerTitle
		titleClass="!inset-0 !translate-none !transform-none flex items-center justify-center text-center pointer-events-none"
		class="sticky {desktopWindow ? 'top-[34px]' : 'top-0'} z-30 shrink-0 lg:hidden"
	>
		{#snippet left()}
			{#if backHref}
				<NavbarBackLink
					text={backLabel ?? i18n.t('nav.back')}
					class={isTopLevelSidebarRoute ? 'md:hidden' : ''}
					onclick={() => goto(backHref)}
				/>
			{:else}
				<Link
					iconOnly
					aria-label={i18n.t('nav.openMenu')}
					class="md:hidden"
					onclick={() => (drawerOpened = true)}
				>
					<Menu class="h-5 w-5" aria-hidden="true" />
				</Link>
			{/if}
		{/snippet}
		{#snippet right()}
			<Link
				aria-label={authState.currentUser
					? `Switch local account, currently @${authState.currentUser.username}`
					: 'Switch local account, currently Guest'}
				class="!px-2.5 gap-1.5"
				onclick={() => (switcherOpened = true)}
			>
				{#if authState.currentUser}
					<span
						class="flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-[var(--xm-accent-soft)] text-[11px] font-bold xm-text-accent"
						aria-hidden="true"
					>
						{(authState.currentUser.nickname || authState.currentUser.username).slice(0, 1).toUpperCase()}
					</span>
					<span class="max-w-[82px] truncate text-[13px] font-semibold xm-text-primary md:max-w-[120px]">
						@{authState.currentUser.username}
					</span>
				{:else}
					<UserIcon class="h-4 w-4 shrink-0 xm-text-accent" aria-hidden="true" />
					<span class="text-[13px] font-semibold xm-text-accent">{i18n.t('nav.guest')}</span>
				{/if}
			</Link>
		{/snippet}
	</Navbar>

	<!-- Desktop (>= 1024px): macOS 2026 Full-Height Translucent Liquid Glass Sidebar -->
	<aside
		class="xm-macos-sidebar hidden shrink-0 overflow-hidden lg:flex lg:h-full lg:flex-col {desktopLayoutState.sidebarOpen
			? 'lg:w-[244px] xl:w-[256px]'
			: 'lg:w-0 lg:border-r-0'}"
		aria-label={i18n.t('nav.workspaceNav')}
		aria-hidden={!desktopLayoutState.sidebarOpen}
	>
		<div
			class="flex h-full min-w-[244px] flex-col transition-[opacity,transform] duration-150 ease-out will-change-transform xl:min-w-[256px] {desktopLayoutState.sidebarOpen
				? 'translate-x-0 opacity-100'
				: '-translate-x-2 opacity-0 pointer-events-none'}"
		>
			<!-- macOS Sidebar Top Brand Header aligned to 52px Unified Toolbar (Integrated Traffic Lights + Drag Region) -->
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div
				data-tauri-drag-region
				onmousedown={handleTitlebarMouseDown}
				class="xm-drag-region flex h-[52px] shrink-0 items-center gap-2.5 border-b border-[var(--xm-divider)] px-3.5"
			>
				{#if desktopWindow}
					<DesktopWindowControls />
					<span class="mx-0.5 h-3.5 w-px shrink-0 bg-[var(--xm-divider)]" aria-hidden="true"></span>
				{/if}
				<AppIcon class="h-7 w-7 shrink-0" decorative dragRegion />
				<span
					data-tauri-drag-region
					class="truncate text-[14px] font-bold tracking-[-0.015em] xm-text-primary"
				>
					Xmemos
				</span>
			</div>

			<!-- Pinned Top Controls: Search Bar + Filter Tabs + Display (directly accessible at top of Left Rail) -->
			{#if desktopLeftSearch || desktopLeftControls}
				<div class="shrink-0 space-y-2.5 border-b border-[var(--xm-divider)] px-3 py-2.5">
					{#if desktopLeftSearch}
						{@render desktopLeftSearch()}
					{/if}
					{#if desktopLeftControls}
						{@render desktopLeftControls()}
					{/if}
				</div>
			{/if}

			<!-- Pinned macOS Source-List Navigation (never scrolled away by Tags) -->
			<nav
				class="xm-sidebar-scroll shrink-0 space-y-3.5 overflow-y-auto overscroll-contain px-3 py-3 {showDesktopLeftTags ? 'max-h-[48%]' : 'flex-1'}"
				aria-label="Primary"
			>
				<div class="space-y-1">
					<div
						class="px-2.5 pb-1 text-[11px] font-semibold tracking-[0.04em] uppercase xm-text-muted select-none"
					>
						{i18n.t('nav.sectionWorkspace')}
					</div>
					{#each WORKSPACE_NAV_ITEMS as item (item.path)}
						{@const IconComp = item.icon}
						{@const active = isNavActive(item.path)}
						<button
							type="button"
							tabindex={desktopLayoutState.sidebarOpen ? 0 : -1}
							aria-current={active ? 'page' : undefined}
							class="xm-macos-nav-item"
							onclick={() => goto(item.path)}
						>
							<IconComp
								class="h-4 w-4 shrink-0 {active ? 'xm-text-accent' : 'xm-text-secondary'}"
								aria-hidden="true"
							/>
							<span class="truncate">{i18n.t(item.labelKey)}</span>
						</button>
					{/each}
				</div>

				<div class="space-y-1">
					<div
						class="px-2.5 pb-1 text-[11px] font-semibold tracking-[0.04em] uppercase xm-text-muted select-none"
					>
						{i18n.t('nav.sectionLibrary')}
					</div>
					{#each LIBRARY_NAV_ITEMS as item (item.path)}
						{@const IconComp = item.icon}
						{@const active = isNavActive(item.path)}
						<button
							type="button"
							tabindex={desktopLayoutState.sidebarOpen ? 0 : -1}
							aria-current={active ? 'page' : undefined}
							class="xm-macos-nav-item"
							onclick={() => goto(item.path)}
						>
							<IconComp
								class="h-4 w-4 shrink-0 {active ? 'xm-text-accent' : 'xm-text-secondary'}"
								aria-hidden="true"
							/>
							<span class="truncate">{i18n.t(item.labelKey)}</span>
						</button>
					{/each}
				</div>
			</nav>

			<!-- Independent Smooth-Scrolling Tags Region (Review & Map on Desktop Left Rail) -->
			{#if showDesktopLeftTags}
				<div
					class="xm-sidebar-scroll min-h-0 flex-1 overflow-y-auto overscroll-contain border-t border-[var(--xm-divider)] px-3 py-3"
				>
					{#if desktopLeftTags}
						{@render desktopLeftTags()}
					{:else}
						<TagTree
							tags={memosState.tags}
							selectedTag={filterState.selectedTag}
							onSelectTag={handleTagSelect}
						/>
					{/if}
				</div>
			{/if}
		</div>
	</aside>

	<!-- Workspace Right Column (display:contents below lg so Mobile/Tablet DOM flow is 100% untouched) -->
	<div class="contents lg:flex lg:min-w-0 lg:flex-1 lg:flex-col lg:overflow-hidden">
		<!-- Desktop (>= 1024px): macOS 2026 Unified Liquid Glass Window Toolbar (52px, Frameless Drag Region) -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<header
			data-tauri-drag-region
			onmousedown={handleTitlebarMouseDown}
			class="xm-macos-toolbar xm-drag-region hidden h-[52px] shrink-0 items-center justify-between gap-3 px-4 lg:flex"
		>
			<!-- Leading Toolbar Cluster: Collapsed Traffic Lights + Sidebar Toggle + Contextual Back + Window Title -->
			<div data-tauri-drag-region class="flex min-w-0 items-center gap-2">
				{#if desktopWindow && !desktopLayoutState.sidebarOpen}
					<DesktopWindowControls />
					<span class="mx-0.5 h-3.5 w-px shrink-0 bg-[var(--xm-divider)]" aria-hidden="true"></span>
				{/if}
				<button
					type="button"
					class="xm-macos-toolbar-btn"
					aria-label={i18n.t('nav.toggleSidebar')}
					aria-pressed={desktopLayoutState.sidebarOpen}
					title={`${i18n.t('nav.toggleSidebar')} (Ctrl+\\)`}
					onclick={() => desktopLayoutState.toggleSidebar()}
				>
					<PanelLeft class="h-[17px] w-[17px]" aria-hidden="true" />
				</button>

				{#if backHref && !isTopLevelSidebarRoute}
					<button
						type="button"
						class="xm-macos-toolbar-btn !px-2"
						onclick={() => goto(backHref)}
					>
						<ChevronLeft class="h-4 w-4 shrink-0" aria-hidden="true" />
						<span class="text-[13px] font-medium">{backLabel ?? i18n.t('nav.back')}</span>
					</button>
				{/if}

				<div class="ml-1 flex min-w-0 items-baseline gap-2">
					<h1 class="truncate text-[15px] font-semibold tracking-[-0.01em] xm-text-primary">
						{title}
					</h1>
					{#if subtitle}
						<span class="hidden truncate text-xs xm-text-muted xl:inline">{subtitle}</span>
					{/if}
				</div>
			</div>

			<!-- Trailing Toolbar Cluster: Single Compose CTA + Inspector Toggle + Account Pill -->
			<div class="flex shrink-0 items-center gap-2">
				{#if authState.isAuthenticated}
					<button
						type="button"
						class="inline-flex h-8 items-center gap-1.5 rounded-full bg-[var(--xm-accent-btn)] px-3.5 text-[13px] font-semibold text-white shadow-[inset_0_1px_0_rgba(255,255,255,0.22),0_1px_3px_rgba(0,0,0,0.25)] transition-colors hover:bg-[var(--xm-accent-btn-hover)] active:scale-[0.98] cursor-pointer"
						onclick={handleSidebarCompose}
					>
						<Plus class="h-3.5 w-3.5 stroke-[2.5]" aria-hidden="true" />
						<span>{i18n.t('home.createMemo')}</span>
					</button>
				{/if}

				{#if rightRailVisible}
					<button
						type="button"
						class="xm-macos-toolbar-btn"
						aria-label={i18n.t('nav.toggleInspector')}
						aria-pressed={desktopLayoutState.inspectorOpen}
						title={`${i18n.t('nav.toggleInspector')} (Ctrl+Shift+\\)`}
						onclick={() => desktopLayoutState.toggleInspector()}
					>
						<PanelRight class="h-[17px] w-[17px]" aria-hidden="true" />
					</button>
				{/if}

				<button
					type="button"
					aria-label={authState.currentUser
						? `Switch local account, currently @${authState.currentUser.username}`
						: 'Switch local account, currently Guest'}
					class="glass-surface inline-flex h-8 items-center gap-1.5 rounded-full px-2.5 text-[13px] font-semibold transition hover:bg-[var(--xm-surface-subtle)] cursor-pointer"
					onclick={() => (switcherOpened = true)}
				>
					{#if authState.currentUser}
						<span
							class="flex h-5 w-5 shrink-0 items-center justify-center rounded-full bg-[var(--xm-accent-soft)] text-[10px] font-bold xm-text-accent"
							aria-hidden="true"
						>
							{(authState.currentUser.nickname || authState.currentUser.username)
								.slice(0, 1)
								.toUpperCase()}
						</span>
						<span class="max-w-[128px] truncate xm-text-primary">
							@{authState.currentUser.username}
						</span>
					{:else}
						<UserIcon class="h-3.5 w-3.5 shrink-0 xm-text-accent" aria-hidden="true" />
						<span class="xm-text-accent">{i18n.t('nav.guest')}</span>
					{/if}
				</button>
			</div>
		</header>

		<!-- Responsive Content Row: Mobile (1-col) | Tablet/iPad md (2-col 228px+1fr) | Desktop lg (macOS Flex Split) -->
		<div
			class="mx-auto grid w-full grid-cols-1 gap-4 px-3 pt-2.5 pb-[calc(env(safe-area-inset-bottom,0px)+8.5rem)] md:min-h-0 md:flex-1 md:overflow-hidden md:grid-cols-[228px_minmax(0,1fr)] md:gap-5 md:px-5 md:pt-0 md:pb-0 lg:mx-0 lg:flex lg:max-w-none lg:gap-0 lg:px-0"
		>
			<!-- Tablet/iPad Only (768px–1023px): Original Sticky Left Rail (100% preserved) -->
			<aside
				class="xm-sidebar-scroll hidden space-y-3 md:block md:h-full md:overflow-y-auto md:overscroll-contain md:py-3 lg:hidden"
				aria-label={i18n.t('nav.workspaceNav')}
			>
				<nav class="glass-surface space-y-1 rounded-2xl p-2" aria-label="Primary">
					{#each TABLET_NAV_ITEMS as item (item.path)}
						{@const IconComp = item.icon}
						{@const active = isNavActive(item.path)}
						<button
							type="button"
							aria-current={active ? 'page' : undefined}
							class="flex min-h-[40px] w-full items-center gap-2.5 rounded-xl px-3 py-2 text-sm font-medium transition-colors {active
								? 'bg-[var(--xm-accent-soft)] text-[var(--xm-accent)] font-semibold'
								: 'xm-text-primary hover:bg-[var(--xm-surface-subtle)]'}"
							onclick={() => goto(item.path)}
						>
							<IconComp class="h-4 w-4 shrink-0" aria-hidden="true" />
							<span class="truncate">{i18n.t(item.labelKey)}</span>
						</button>
					{/each}
				</nav>

				{#if authState.isAuthenticated}
					<Button
						rounded
						class="!h-11 !text-sm !font-semibold !shadow-sm"
						onclick={handleSidebarCompose}
					>
						<Plus class="mr-1.5 h-4 w-4 stroke-[2.4]" aria-hidden="true" />
						<span>{i18n.t('home.createMemo')}</span>
					</Button>
				{/if}

				{#if rightRailVisible}
					<TagTree
						tags={memosState.tags}
						selectedTag={filterState.selectedTag}
						onSelectTag={handleTagSelect}
					/>
				{/if}
			</aside>

			<!-- Single Primary Content Canvas (mounted once across all viewports; independent scroll on md+) -->
			<main
				id="main-content"
				class="xm-page-enter xm-sidebar-scroll min-w-0 md:h-full md:overflow-y-auto md:overscroll-contain md:py-3 md:pr-1 lg:flex-1 {isFullCanvasRoute
					? 'lg:px-5 lg:py-4'
					: 'lg:px-6 lg:py-4'}"
				tabindex="-1"
			>
				<div class={mainWidthClass}>
					{@render children()}
				</div>
			</main>

			<!-- Desktop (>= 1024px): Collapsible macOS 2026 Trailing Inspector Rail (Pinned Calendar + Independent Scroll Tags) -->
			{#if rightRailVisible && desktopLayoutState.inspectorOpen}
				<aside
					class="xm-macos-inspector xm-panel-reveal hidden lg:flex lg:h-full lg:w-[288px] lg:shrink-0 lg:flex-col lg:gap-3.5 lg:overflow-hidden lg:p-4 xl:w-[304px]"
					aria-label={i18n.t('nav.activityInsights')}
				>
					<div class="shrink-0">
						<ActivityHeatmap
							days={memosState.heatmap}
							selectedDate={filterState.selectedDate}
							onSelectDate={handleDateSelect}
						/>
					</div>
					<div class="xm-sidebar-scroll min-h-0 flex-1 overflow-y-auto overscroll-contain">
						<TagTree
							tags={memosState.tags}
							selectedTag={filterState.selectedTag}
							onSelectTag={handleTagSelect}
						/>
					</div>
				</aside>
			{/if}
		</div>
	</div>

	<!-- Clean iOS 26 Floating Action Button (mobile only; Tablet/Desktop uses persistent Compose CTA) -->
	{#if showFab && authState.isAuthenticated && onCompose}
		<Fab
			aria-label={i18n.t('nav.createMemo')}
			class="fixed right-4 bottom-[calc(env(safe-area-inset-bottom,0px)+5.75rem)] z-30 !h-13 !w-13 !shadow-lg transition-transform active:scale-95 md:hidden"
			onclick={onCompose}
		>
			{#snippet icon()}
				<Plus class="h-6 w-6 stroke-[2.25]" aria-hidden="true" />
			{/snippet}
		</Fab>
	{/if}
</Page>

<!-- Mobile Slide-Over Drawer for Tags, Heatmap, Inbox, and Archived (100% preserved) -->
<Panel side="left" opened={drawerOpened} onBackdropClick={() => (drawerOpened = false)}>
	<Page>
		<Navbar
			title={i18n.t('nav.menu')}
			centerTitle
			titleClass="!inset-0 !translate-none !transform-none flex items-center justify-center text-center pointer-events-none"
			class="sticky top-0 z-20"
		>
			{#snippet right()}
				<Link
					iconOnly
					aria-label={i18n.t('nav.closeMenu')}
					onclick={() => (drawerOpened = false)}
				>
					<X class="h-5 w-5" aria-hidden="true" />
				</Link>
			{/snippet}
		</Navbar>
		<div class="space-y-3 p-3 pb-8">
			<nav
				class="glass-surface space-y-1 rounded-2xl p-2"
				aria-label={i18n.t('nav.additionalSections')}
			>
				{#each MOBILE_EXTRA_ITEMS as item (item.path)}
					{@const IconComp = item.icon}
					{@const active = isNavActive(item.path)}
					<button
						type="button"
						aria-current={active ? 'page' : undefined}
						class="flex min-h-[44px] w-full items-center gap-2.5 rounded-xl px-3 py-2.5 text-sm font-medium transition-colors {active
							? 'bg-[var(--xm-accent-soft)] text-[var(--xm-accent)] font-semibold'
							: 'xm-text-primary hover:bg-[var(--xm-surface-subtle)]'}"
						onclick={() => {
							drawerOpened = false;
							goto(item.path);
						}}
					>
						<IconComp class="h-4 w-4 shrink-0" aria-hidden="true" />
						<span>{i18n.t(item.labelKey)}</span>
					</button>
				{/each}
			</nav>
			<ActivityHeatmap
				days={memosState.heatmap}
				selectedDate={filterState.selectedDate}
				weeks={12}
				onSelectDate={(d) => {
					drawerOpened = false;
					handleDateSelect(d);
				}}
			/>
			<TagTree
				tags={memosState.tags}
				selectedTag={filterState.selectedTag}
				onSelectTag={(t) => {
					drawerOpened = false;
					handleTagSelect(t);
				}}
			/>
		</div>
	</Page>
</Panel>

<AccountSwitcherSheet opened={switcherOpened} onClose={() => (switcherOpened = false)} />

{#if !onCompose}
	<MemoEditor
		opened={quickComposeOpened}
		onClose={() => (quickComposeOpened = false)}
	/>
{/if}
