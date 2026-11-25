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
	import {
		ChevronLeft,
		ChevronRight,
		Download,
		Info,
		RotateCcw,
		RotateCw,
		X,
		ZoomIn,
		ZoomOut
	} from 'lucide-svelte';

	import { resourceApi } from '#lib/api/resource.ts';
	import { formatSavedLabel, saveBlobWithNativeDialog } from '#lib/utils/download.ts';
	import { i18n } from '#lib/i18n/index.svelte.ts';
	import { lightboxState } from '#lib/state/lightbox.svelte.ts';
	import { toastState } from '#lib/state/toast.svelte.ts';
	import { formatBytes } from '#lib/utils/date.ts';

	const MIN_SCALE = 0.25;
	const MAX_SCALE = 4;
	const STEP_SCALE = 0.25;

	let scale = $state(1);
	let rotation = $state(0);
	let offsetX = $state(0);
	let offsetY = $state(0);
	let dragging = $state(false);
	let dragStartX = 0;
	let dragStartY = 0;
	let dragOriginX = 0;
	let dragOriginY = 0;
	let infoOpened = $state(false);
	let downloading = $state(false);
	let naturalWidth = $state<number | null>(null);
	let naturalHeight = $state<number | null>(null);

	let isTransformed = $derived(scale !== 1 || rotation !== 0 || offsetX !== 0 || offsetY !== 0);
	let zoomPercent = $derived(Math.round(scale * 100));

	function resetTransform(): void {
		scale = 1;
		rotation = 0;
		offsetX = 0;
		offsetY = 0;
	}

	function zoomIn(): void {
		scale = Math.min(MAX_SCALE, Math.round((scale + STEP_SCALE) * 100) / 100);
	}

	function zoomOut(): void {
		const next = Math.max(MIN_SCALE, Math.round((scale - STEP_SCALE) * 100) / 100);
		scale = next;
		if (next <= 1) {
			offsetX = 0;
			offsetY = 0;
		}
	}

	function rotateRight(): void {
		rotation = (rotation + 90) % 360;
	}

	function toggleDoubleZoom(): void {
		if (scale > 1) {
			scale = 1;
			offsetX = 0;
			offsetY = 0;
		} else {
			scale = 2;
		}
	}

	function handleWheel(e: WheelEvent): void {
		e.preventDefault();
		if (e.deltaY < 0) {
			zoomIn();
		} else {
			zoomOut();
		}
	}

	function handlePointerDown(e: PointerEvent): void {
		if (scale <= 1) return;
		e.preventDefault();
		dragging = true;
		dragStartX = e.clientX;
		dragStartY = e.clientY;
		dragOriginX = offsetX;
		dragOriginY = offsetY;
		(e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
	}

	function handlePointerMove(e: PointerEvent): void {
		if (!dragging) return;
		offsetX = dragOriginX + (e.clientX - dragStartX);
		offsetY = dragOriginY + (e.clientY - dragStartY);
	}

	function handlePointerUp(e: PointerEvent): void {
		if (!dragging) return;
		dragging = false;
		try {
			(e.currentTarget as HTMLElement).releasePointerCapture(e.pointerId);
		} catch {
			// Ignore if capture was already released
		}
	}

	async function handleDownload(): Promise<void> {
		const media = lightboxState.current;
		if (!media || downloading) return;
		downloading = true;
		try {
			// Primary native Tauri path: opens OS File Manager Save Dialog on Desktop, Downloads on Android
			if (media.uid) {
				const savedPath = await resourceApi.exportToDownloads(media.uid);
				if (savedPath !== null) {
					toastState.show(
						i18n.t('lightbox.downloaded', {
							filename: formatSavedLabel(savedPath, media.filename || 'image')
						}),
						'success'
					);
				}
				return;
			}
			// Fallback for unsaved local blob: preview URLs
			const res = await fetch(media.src);
			if (!res.ok) throw new Error(`HTTP ${res.status}`);
			const blob = await res.blob();
			const savedPath = await saveBlobWithNativeDialog(blob, media.filename || 'image', blob.type);
			if (savedPath !== null) {
				toastState.show(
					i18n.t('lightbox.downloaded', {
						filename: formatSavedLabel(savedPath, media.filename || 'image')
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
		} finally {
			downloading = false;
		}
	}

	function handleWindowKeydown(e: KeyboardEvent): void {
		if (!lightboxState.current) return;
		switch (e.key) {
			case 'Escape':
				e.preventDefault();
				lightboxState.close();
				break;
			case '+':
			case '=':
				e.preventDefault();
				zoomIn();
				break;
			case '-':
			case '_':
				e.preventDefault();
				zoomOut();
				break;
			case '0':
				e.preventDefault();
				resetTransform();
				break;
			case 'r':
			case 'R':
				e.preventDefault();
				rotateRight();
				break;
			case 'i':
			case 'I':
				e.preventDefault();
				infoOpened = !infoOpened;
				break;
			case 'ArrowLeft':
				if (lightboxState.hasMultiple) {
					e.preventDefault();
					lightboxState.prev();
				}
				break;
			case 'ArrowRight':
				if (lightboxState.hasMultiple) {
					e.preventDefault();
					lightboxState.next();
				}
				break;
		}
	}

	$effect(() => {
		const activeSrc = lightboxState.current?.src;
		void activeSrc;
		untrack(() => {
			resetTransform();
			naturalWidth = null;
			naturalHeight = null;
			if (!activeSrc) {
				infoOpened = false;
			}
		});
	});

	$effect(() => {
		if (typeof document === 'undefined') return;
		if (lightboxState.current) {
			const prevOverflow = document.body.style.overflow;
			document.body.style.overflow = 'hidden';
			return () => {
				document.body.style.overflow = prevOverflow;
			};
		}
	});
</script>

<svelte:window onkeydown={handleWindowKeydown} />

{#if lightboxState.current}
	{@const media = lightboxState.current}
	<!-- Official Memos 2026 Fullscreen Image Preview with Konsta iOS Floating Controls -->
	<!-- svelte-ignore a11y_click_events_have_key_events -->
	<!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
	<div
		class="fixed inset-0 z-[90] flex h-[100dvh] w-[100vw] flex-col bg-black/95 select-none"
		role="dialog"
		aria-modal="true"
		tabindex="-1"
		aria-label="Image preview: {media.filename}"
		onclick={(e) => {
			if (e.target === e.currentTarget) lightboxState.close();
		}}
	>
		<!-- Top Bar: Filename (left), Info + Download + Close (right) -->
		<header
			class="relative z-20 flex shrink-0 items-center justify-between gap-3 px-4 pt-[max(0.75rem,env(safe-area-inset-top))] pb-2.5"
		>
			<div class="flex min-w-0 flex-1 items-center gap-2">
				<span class="truncate text-sm font-medium text-white/95">{media.filename}</span>
				{#if lightboxState.hasMultiple}
					<span
						class="shrink-0 rounded-full bg-white/12 px-2 py-0.5 text-[11px] font-medium text-white/80 tabular-nums"
					>
						{lightboxState.index + 1} / {lightboxState.items.length}
					</span>
				{/if}
			</div>

			<div class="flex shrink-0 items-center gap-1.5">
				<button
					type="button"
					aria-label={i18n.t('lightbox.info')}
					aria-pressed={infoOpened}
					title={i18n.t('lightbox.info')}
					class="flex h-9 w-9 items-center justify-center rounded-xl border border-white/12 text-white transition-colors active:scale-95 {infoOpened
						? 'bg-white/25'
						: 'bg-white/10 hover:bg-white/20'}"
					onclick={() => (infoOpened = !infoOpened)}
				>
					<Info class="h-4 w-4" aria-hidden="true" />
				</button>
				<button
					type="button"
					aria-label={i18n.t('lightbox.download')}
					title={i18n.t('lightbox.download')}
					class="flex h-9 w-9 items-center justify-center rounded-xl border border-white/12 bg-white/10 text-white transition-colors hover:bg-white/20 active:scale-95"
					disabled={downloading}
					onclick={() => void handleDownload()}
				>
					<Download class="h-4 w-4" aria-hidden="true" />
				</button>
				<button
					type="button"
					aria-label={i18n.t('lightbox.close')}
					title={i18n.t('lightbox.close')}
					class="flex h-9 w-9 items-center justify-center rounded-xl border border-white/12 bg-white/10 text-white transition-colors hover:bg-white/20 active:scale-95"
					onclick={() => lightboxState.close()}
				>
					<X class="h-4 w-4" aria-hidden="true" />
				</button>
			</div>

			<!-- Floating Image Info Popover -->
			{#if infoOpened}
				<div
					class="absolute top-full right-4 z-30 mt-1.5 w-64 space-y-2 rounded-2xl border border-white/15 bg-[#1c1c1e]/92 p-3.5 text-xs text-white shadow-2xl backdrop-blur-xl"
				>
					<div class="font-semibold text-white/95">{i18n.t('lightbox.details')}</div>
					<dl class="space-y-1.5 text-[11px]">
						<div class="flex justify-between gap-2">
							<dt class="text-white/55">{i18n.t('lightbox.filename')}</dt>
							<dd class="truncate text-right font-medium text-white/90">{media.filename}</dd>
						</div>
						{#if naturalWidth && naturalHeight}
							<div class="flex justify-between gap-2">
								<dt class="text-white/55">{i18n.t('lightbox.dimensions')}</dt>
								<dd class="font-medium text-white/90 tabular-nums">
									{naturalWidth} × {naturalHeight} px
								</dd>
							</div>
						{/if}
						{#if media.size && media.size > 0}
							<div class="flex justify-between gap-2">
								<dt class="text-white/55">{i18n.t('lightbox.size')}</dt>
								<dd class="font-medium text-white/90 tabular-nums">{formatBytes(media.size)}</dd>
							</div>
						{/if}
						<div class="flex justify-between gap-2">
							<dt class="text-white/55">{i18n.t('lightbox.zoom')}</dt>
							<dd class="font-medium text-white/90 tabular-nums">{zoomPercent}%</dd>
						</div>
					</dl>
				</div>
			{/if}
		</header>

		<!-- Centered Interactive Viewport Stage -->
		<!-- svelte-ignore a11y_click_events_have_key_events -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			class="relative flex min-h-0 flex-1 items-center justify-center overflow-hidden px-4"
			onwheel={handleWheel}
			onclick={(e) => {
				if (e.target === e.currentTarget) lightboxState.close();
			}}
		>
			{#if lightboxState.hasMultiple}
				<button
					type="button"
					aria-label={i18n.t('lightbox.prev')}
					class="absolute left-3 z-20 flex h-10 w-10 items-center justify-center rounded-full border border-white/15 bg-black/55 text-white backdrop-blur-md transition-colors hover:bg-white/20 active:scale-95"
					onclick={(e) => {
						e.stopPropagation();
						lightboxState.prev();
					}}
				>
					<ChevronLeft class="h-5 w-5" aria-hidden="true" />
				</button>
				<button
					type="button"
					aria-label={i18n.t('lightbox.next')}
					class="absolute right-3 z-20 flex h-10 w-10 items-center justify-center rounded-full border border-white/15 bg-black/55 text-white backdrop-blur-md transition-colors hover:bg-white/20 active:scale-95"
					onclick={(e) => {
						e.stopPropagation();
						lightboxState.next();
					}}
				>
					<ChevronRight class="h-5 w-5" aria-hidden="true" />
				</button>
			{/if}

			<img
				src={media.src}
				alt={media.filename}
				draggable="false"
				onload={(e) => {
					const img = e.currentTarget as HTMLImageElement;
					naturalWidth = img.naturalWidth || null;
					naturalHeight = img.naturalHeight || null;
				}}
				ondblclick={toggleDoubleZoom}
				onpointerdown={handlePointerDown}
				onpointermove={handlePointerMove}
				onpointerup={handlePointerUp}
				onpointercancel={handlePointerUp}
				style="transform: translate3d({offsetX}px, {offsetY}px, 0) scale({scale}) rotate({rotation}deg);"
				class="max-h-full max-w-full rounded-lg object-contain shadow-2xl {dragging
					? 'cursor-grabbing transition-none'
					: scale > 1
						? 'cursor-grab transition-transform duration-150 ease-out'
						: 'cursor-zoom-in transition-transform duration-150 ease-out'}"
			/>
		</div>

		<!-- Official Memos Floating Bottom Zoom & Rotate Pill Toolbar -->
		<footer
			class="relative z-20 flex shrink-0 items-center justify-center px-4 pt-2.5 pb-[max(1rem,env(safe-area-inset-bottom))]"
		>
			<div
				class="inline-flex items-center gap-1 rounded-2xl border border-white/15 bg-[#1c1c1e]/85 p-1.5 text-white shadow-2xl backdrop-blur-xl"
			>
				<button
					type="button"
					aria-label={i18n.t('lightbox.zoomOut')}
					title={i18n.t('lightbox.zoomOut')}
					disabled={scale <= MIN_SCALE}
					class="flex h-8 w-8 items-center justify-center rounded-xl bg-white/10 text-white transition-colors hover:bg-white/20 active:scale-95 disabled:opacity-35"
					onclick={zoomOut}
				>
					<ZoomOut class="h-4 w-4" aria-hidden="true" />
				</button>

				<button
					type="button"
					aria-label={i18n.t('lightbox.reset')}
					title={i18n.t('lightbox.reset')}
					class="min-w-[52px] rounded-xl px-2 py-1 text-center text-xs font-semibold text-white/95 tabular-nums transition-colors hover:bg-white/10"
					onclick={resetTransform}
				>
					{zoomPercent}%
				</button>

				<button
					type="button"
					aria-label={i18n.t('lightbox.zoomIn')}
					title={i18n.t('lightbox.zoomIn')}
					disabled={scale >= MAX_SCALE}
					class="flex h-8 w-8 items-center justify-center rounded-xl bg-white/10 text-white transition-colors hover:bg-white/20 active:scale-95 disabled:opacity-35"
					onclick={zoomIn}
				>
					<ZoomIn class="h-4 w-4" aria-hidden="true" />
				</button>

				<span class="mx-1 h-4 w-px bg-white/18" aria-hidden="true"></span>

				<button
					type="button"
					aria-label={i18n.t('lightbox.rotate')}
					title={i18n.t('lightbox.rotate')}
					class="flex h-8 w-8 items-center justify-center rounded-xl bg-white/10 text-white transition-colors hover:bg-white/20 active:scale-95"
					onclick={rotateRight}
				>
					<RotateCw class="h-4 w-4" aria-hidden="true" />
				</button>

				{#if isTransformed}
					<button
						type="button"
						aria-label={i18n.t('lightbox.reset')}
						title={i18n.t('lightbox.reset')}
						class="flex h-8 w-8 items-center justify-center rounded-xl bg-white/10 text-white transition-colors hover:bg-white/20 active:scale-95"
						onclick={resetTransform}
					>
						<RotateCcw class="h-4 w-4" aria-hidden="true" />
					</button>
				{/if}
			</div>
		</footer>
	</div>
{/if}
