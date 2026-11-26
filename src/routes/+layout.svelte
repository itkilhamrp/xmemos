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
	import '../app.css';
	import type { Snippet } from 'svelte';
	import { onMount } from 'svelte';
	import { invoke, isTauri } from '@tauri-apps/api/core';
	import { getCurrentWindow } from '@tauri-apps/api/window';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { App, Button, Dialog, DialogButton, Toast } from 'konsta/svelte';

	import { shortcutApi } from '#lib/api/shortcut.ts';
	import { isDesktopTauri } from '#lib/utils/platform.ts';
	import { initKeyboardViewportSync } from '#lib/utils/keyboard.svelte.ts';
	import { systemApi } from '#lib/api/system.ts';
	import { authState } from '#lib/state/auth.svelte.ts';
	import { confirmDeleteState } from '#lib/state/confirm-delete.svelte.ts';
	import { desktopLayoutState } from '#lib/state/desktop-layout.svelte.ts';
	import { i18n } from '#lib/i18n/index.svelte.ts';
	import { themeState } from '#lib/state/theme.svelte.ts';
	import { shortcutsState } from '#lib/state/shortcuts.svelte.ts';
	import { toastState } from '#lib/state/toast.svelte.ts';
	import PinLockOverlay from '#lib/components/auth/PinLockOverlay.svelte';
	import BottomTabbar from '#lib/components/layout/BottomTabbar.svelte';
	import ImageLightbox from '#lib/components/media/ImageLightbox.svelte';
	import LocationPreviewPopover from '#lib/components/map/LocationPreviewPopover.svelte';
	import TimelineDisplayPopover from '#lib/components/ui/TimelineDisplayPopover.svelte';
	import KeyboardShortcutsModal from '#lib/components/ui/KeyboardShortcutsModal.svelte';
	import MemoActionsSheet from '#lib/components/memo/MemoActionsSheet.svelte';

	interface Props {
		children: Snippet;
	}
	let { children }: Props = $props();
	let isStickyRoute = $derived(page.url.pathname.startsWith('/sticky/'));

	// Synchronous frame-0 hydration before first paint
	i18n.init();
	themeState.init();
	confirmDeleteState.init();
	desktopLayoutState.init();
	authState.hydrateFromCache();

	async function openPendingShortcutMemo(): Promise<void> {
		if (!authState.hasHost || isStickyRoute) return;
		const memoUid = await shortcutApi.consumePendingMemoShortcut();
		if (memoUid) {
			if (isDesktopTauri()) {
				await shortcutApi.openStickyMemoWindow(memoUid);
			} else {
				await goto(`/memo/${encodeURIComponent(memoUid)}`);
			}
		}
	}

	onMount(() => {
		const revealMainWindow = () => {
			if (!isDesktopTauri()) return;
			requestAnimationFrame(() => {
				requestAnimationFrame(() => {
					void systemApi.showMainWindow().catch(() => {});
				});
			});
		};

		const stopKeyboardSync = initKeyboardViewportSync();
		let unlistenShortcut: (() => void) | null = null;
		let unlistenMainOpen: (() => void) | null = null;
		const handleVisibilityResume = () => {
			if (document.visibilityState === 'visible') {
				void openPendingShortcutMemo();
			}
		};
		document.addEventListener('visibilitychange', handleVisibilityResume);

		// Prevent desktop touchpad 2-finger pinch gestures from zooming the entire WebView app UI
		const blockAppPinchWheel = (ev: WheelEvent) => {
			if (ev.ctrlKey) {
				ev.preventDefault();
			}
		};
		const blockAppGesture = (ev: Event) => {
			ev.preventDefault();
		};
		window.addEventListener('wheel', blockAppPinchWheel, { passive: false });
		document.addEventListener('gesturestart', blockAppGesture, { passive: false });
		document.addEventListener('gesturechange', blockAppGesture, { passive: false });
		document.addEventListener('gestureend', blockAppGesture, { passive: false });

		void (async () => {
			await authState.refresh();
			if (!authState.hasHost && page.url.pathname !== '/auth') {
				revealMainWindow();
				await goto('/auth', { replaceState: true });
				return;
			}
			// On Desktop, launching a Desktop Sticky Note shortcut opens the floating MS Sticky Note
			// window directly while keeping the main window hidden (unless PIN-locked).
			if (isDesktopTauri() && !isStickyRoute) {
				const pendingUid = await shortcutApi.consumePendingMemoShortcut();
				if (pendingUid && authState.hasHost && !authState.isLocked) {
					try {
						await shortcutApi.openStickyMemoWindow(pendingUid);
					} catch {
						revealMainWindow();
					}
				} else {
					revealMainWindow();
					if (pendingUid && authState.hasHost) {
						void shortcutApi.openStickyMemoWindow(pendingUid);
					}
				}
			} else {
				revealMainWindow();
				void openPendingShortcutMemo();
			}
			void shortcutApi
				.onMemoShortcutOpened((memoUid) => {
					if (authState.hasHost) {
						void goto(`/memo/${encodeURIComponent(memoUid)}`);
					}
				})
				.then((fn) => {
					unlistenShortcut = fn;
				});
			if (isDesktopTauri() && !isStickyRoute) {
				try {
					unlistenMainOpen = await getCurrentWindow().listen<string>(
						'xmemos:navigate-main',
						(ev) => {
							const targetRoute = typeof ev.payload === 'string' ? ev.payload.trim() : '';
							if (targetRoute.startsWith('/') && !targetRoute.startsWith('//') && authState.hasHost) {
								void goto(targetRoute);
							}
						}
					);
				} catch {
					// Ignore event listener errors outside Desktop Tauri
				}
			}
		})();

		return () => {
			document.removeEventListener('visibilitychange', handleVisibilityResume);
			window.removeEventListener('wheel', blockAppPinchWheel);
			document.removeEventListener('gesturestart', blockAppGesture);
			document.removeEventListener('gesturechange', blockAppGesture);
			document.removeEventListener('gestureend', blockAppGesture);
			stopKeyboardSync();
			unlistenShortcut?.();
			unlistenMainOpen?.();
		};
	});

	function handleExternalLinkClick(event: MouseEvent): void {
		const target = event.target as HTMLElement | null;
		const anchor = target?.closest('a[href]') as HTMLAnchorElement | null;
		if (!anchor) return;
		const href = anchor.getAttribute('href')?.trim() ?? '';
		if (!/^https?:\/\//i.test(href)) return;
		event.preventDefault();
		if (isTauri()) {
			void invoke('plugin:opener|open_url', { url: href }).catch(() => {
				window.open(href, '_blank', 'noopener,noreferrer');
			});
		} else {
			window.open(href, '_blank', 'noopener,noreferrer');
		}
	}

	let pendingGoChord = false;
	let goChordTimer: ReturnType<typeof setTimeout> | null = null;

	function clearGoChord(): void {
		pendingGoChord = false;
		if (goChordTimer) {
			clearTimeout(goChordTimer);
			goChordTimer = null;
		}
	}

	function isTypingElement(target: EventTarget | null): boolean {
		const el = target as HTMLElement | null;
		if (!el) return false;
		return (
			el.tagName === 'INPUT' ||
			el.tagName === 'TEXTAREA' ||
			el.tagName === 'SELECT' ||
			el.isContentEditable
		);
	}

	function hasBlockingOverlay(): boolean {
		return Boolean(
			confirmDeleteState.pending !== null ||
			document.querySelector('[role="dialog"][aria-modal="true"], .k-sheet-modal-in, .k-popup-in')
		);
	}

	function handleGlobalNavKeys(event: KeyboardEvent): void {
		if (isStickyRoute) return;
		if (event.isComposing || event.key === 'Process') return;

		const mod = event.metaKey || event.ctrlKey;
		const key = event.key;
		const lower = key.toLowerCase();
		const typing = isTypingElement(event.target);

		// 1. Ctrl+/ or Cmd+/ always toggles the Keyboard Shortcuts cheatsheet
		if (mod && !event.altKey && (key === '/' || key === '?' || event.code === 'Slash')) {
			event.preventDefault();
			clearGoChord();
			shortcutsState.toggleCheatsheet();
			return;
		}

		// 2. Escape closes shortcuts modal first (or blurs focused input cleanly)
		if (key === 'Escape') {
			clearGoChord();
			if (shortcutsState.cheatsheetOpened) {
				event.preventDefault();
				shortcutsState.closeCheatsheet();
				return;
			}
			return;
		}

		// Block other shortcuts while shortcuts cheatsheet is open
		if (shortcutsState.cheatsheetOpened) return;

		// 3. Ctrl/Cmd+K focuses Search (works even from other inputs, navigates Home if needed)
		if (mod && !event.altKey && !event.shiftKey && lower === 'k') {
			event.preventDefault();
			clearGoChord();
			if (!shortcutsState.triggerFocusSearch()) {
				void goto('/').then(() => {
					setTimeout(() => shortcutsState.triggerFocusSearch(), 120);
				});
			}
			return;
		}

		// 4. Ctrl/Cmd+, opens Settings (macOS standard)
		if (mod && !event.altKey && !event.shiftKey && key === ',') {
			event.preventDefault();
			clearGoChord();
			void goto('/settings');
			return;
		}

		// 5. Desktop Split-View shortcuts: Ctrl/Cmd+\ (Sidebar), Ctrl/Cmd+Shift+\ (Inspector)
		if (mod && !event.altKey && (key === '\\' || event.code === 'Backslash')) {
			if (typeof window !== 'undefined' && window.innerWidth >= 1024 && !typing) {
				event.preventDefault();
				clearGoChord();
				if (event.shiftKey) {
					desktopLayoutState.toggleInspector();
				} else {
					desktopLayoutState.toggleSidebar();
				}
			}
			return;
		}

		if (typing) return;

		// 6. Standard macOS/Desktop Back & Forward navigation shortcuts
		if ((event.altKey && key === 'ArrowLeft') || (event.metaKey && key === '[')) {
			event.preventDefault();
			clearGoChord();
			window.history.back();
			return;
		}
		if ((event.altKey && key === 'ArrowRight') || (event.metaKey && key === ']')) {
			event.preventDefault();
			clearGoChord();
			window.history.forward();
			return;
		}

		// 7. Alt+1..5 quick workspace route jump
		if (event.altKey && !mod && !event.shiftKey) {
			const altRoutes: Record<string, string> = {
				'1': '/',
				'2': '/explore',
				'3': '/review',
				'4': '/map',
				'5': '/resources'
			};
			if (altRoutes[key]) {
				event.preventDefault();
				clearGoChord();
				void goto(altRoutes[key]);
				return;
			}
		}

		// Never fire single-key shortcuts with modifiers or when a modal/sheet is active
		if (mod || event.altKey || hasBlockingOverlay()) {
			clearGoChord();
			return;
		}

		// 8. '?' opens Keyboard Shortcuts cheatsheet
		if (key === '?') {
			event.preventDefault();
			clearGoChord();
			shortcutsState.toggleCheatsheet();
			return;
		}

		// 9. '/' focuses Search bar
		if (key === '/') {
			event.preventDefault();
			clearGoChord();
			if (!shortcutsState.triggerFocusSearch()) {
				void goto('/').then(() => {
					setTimeout(() => shortcutsState.triggerFocusSearch(), 120);
				});
			}
			return;
		}

		// 10. Two-key 'G' navigation chords (G H/E/R/M/F/I/A/S)
		if (pendingGoChord) {
			const goRoutes: Record<string, string> = {
				h: '/',
				e: '/explore',
				r: '/review',
				m: '/map',
				f: '/resources',
				i: '/inbox',
				a: '/archived',
				s: '/settings'
			};
			clearGoChord();
			if (goRoutes[lower]) {
				event.preventDefault();
				void goto(goRoutes[lower]);
				return;
			}
		}

		if (lower === 'g' && !event.shiftKey) {
			pendingGoChord = true;
			goChordTimer = setTimeout(clearGoChord, 750);
			return;
		}

		// 11. 'N' or 'C' opens Create New Memo (when authenticated)
		if ((lower === 'n' || lower === 'c') && !event.shiftKey && authState.isAuthenticated) {
			if (shortcutsState.triggerCompose()) {
				event.preventDefault();
			}
		}
	}
</script>

<svelte:window onclick={handleExternalLinkClick} onkeydown={handleGlobalNavKeys} />

<!-- Konsta UI v5 iOS theme: dark={true} enables dark: variants controlled by <html class="dark"> -->
<App theme="ios" dark={true} safeAreas>
	{@render children()}

	{#if !isStickyRoute}
		<!-- Persistent mobile Tabbar mounted once at root for smooth native iOS slider transitions -->
		<BottomTabbar />

		<!-- Root-mounted Memo Actions Sheet (isolated from card stacking/containment contexts) -->
		<MemoActionsSheet />

		<ImageLightbox />

		<!-- Root-mounted Official Memos Mini-Map Location Preview Popover (immune to card backdrop-filter) -->
		<LocationPreviewPopover />
		<TimelineDisplayPopover />
		{#if isDesktopTauri()}
			<KeyboardShortcutsModal />
		{/if}
	{/if}

	<PinLockOverlay />

	<!-- Authentic Konsta UI v5 iOS 26 Delete Confirmation Dialog (supports dynamic breakdown details) -->
	<Dialog
		opened={confirmDeleteState.pending !== null}
		onBackdropClick={() => confirmDeleteState.cancel()}
	>
		{#snippet title()}
			{confirmDeleteState.pending?.title ?? i18n.t('common.confirmDelete')}
		{/snippet}
		<div class="space-y-3 text-left">
			<p class="text-center text-[13px] leading-relaxed xm-text-secondary">
				{confirmDeleteState.pending?.message ?? i18n.t('common.cannotUndo')}
			</p>
			{#if confirmDeleteState.pending?.details && confirmDeleteState.pending.details.length > 0}
				<div
					class="divide-y divide-[var(--xm-divider)] rounded-2xl border border-[var(--xm-divider)] bg-[var(--xm-surface-subtle)] px-3 py-1"
				>
					{#each confirmDeleteState.pending.details as row (row.label)}
						<div class="flex items-center justify-between gap-3 py-2 text-xs">
							<span class="xm-text-secondary">{row.label}</span>
							<span class="font-semibold xm-text-primary tabular-nums">{row.value}</span>
						</div>
					{/each}
				</div>
			{/if}
			{#if confirmDeleteState.pending?.samples && confirmDeleteState.pending.samples.length > 0}
				<div class="space-y-1 px-0.5">
					<div class="text-[11px] font-medium xm-text-muted">{i18n.t('common.examples')}</div>
					<div class="flex flex-wrap gap-1">
						{#each confirmDeleteState.pending.samples as name (name)}
							<span
								class="max-w-full truncate rounded-lg bg-[var(--xm-surface-subtle)] px-2 py-0.5 text-[11px] xm-text-secondary"
							>
								{name}
							</span>
						{/each}
					</div>
				</div>
			{/if}
		</div>
		{#snippet buttons()}
			<DialogButton
				class="!bg-black/8 dark:!bg-white/14 !text-[var(--xm-text-primary)] !font-medium"
				disabled={confirmDeleteState.busy}
				onclick={() => confirmDeleteState.cancel()}
			>
				{i18n.t('common.cancel')}
			</DialogButton>
			<DialogButton
				strong
				class="!bg-[#d70015] dark:!bg-[#ff453a] !text-white !font-semibold"
				disabled={confirmDeleteState.busy}
				onclick={() => confirmDeleteState.confirm()}
			>
				{confirmDeleteState.busy
					? i18n.t('common.deleting')
					: (confirmDeleteState.pending?.confirmLabel ?? i18n.t('common.delete'))}
			</DialogButton>
		{/snippet}
	</Dialog>

	<!-- Toast positioned cleanly above the mobile floating Tabbar so it never overlaps navigation -->
	<Toast
		position="center"
		opened={toastState.opened}
		class="!bottom-[calc(env(safe-area-inset-bottom,0px)+5.5rem)] md:!bottom-6 z-50 !max-w-[min(calc(100vw-2rem),26rem)] !rounded-2xl !shadow-xl"
	>
		{#snippet button()}
			<Button clear inline onclick={() => toastState.dismiss()}>{i18n.t('common.ok')}</Button>
		{/snippet}
		<div class="shrink text-[13px] font-medium leading-snug" role="status" aria-live="polite">
			{toastState.message}
		</div>
	</Toast>
</App>
