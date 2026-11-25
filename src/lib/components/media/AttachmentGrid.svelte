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
	import type { Resource } from '#lib/types/resource.ts';
	import { getMediaUrl } from '#lib/api/invoke.ts';
	import { resourceApi } from '#lib/api/resource.ts';
	import { i18n } from '#lib/i18n/index.svelte.ts';
	import { lightboxState } from '#lib/state/lightbox.svelte.ts';
	import { toastState } from '#lib/state/toast.svelte.ts';
	import { formatBytes } from '#lib/utils/date.ts';
	import { formatSavedLabel } from '#lib/utils/download.ts';
	import { Download, FileArchive, FileCode, FileText, Film, Music, ZoomIn } from 'lucide-svelte';

	interface Props {
		resources: Resource[];
		compact?: boolean;
		previewUrls?: ReadonlyMap<number, string>;
	}

	let { resources, compact = false, previewUrls }: Props = $props();
	let downloadingUid = $state<string | null>(null);

	function isImage(mime: string): boolean {
		return mime.startsWith('image/');
	}
	function isVideo(mime: string): boolean {
		return mime.startsWith('video/');
	}
	function isAudio(mime: string): boolean {
		return mime.startsWith('audio/');
	}
	function isArchive(mime: string, name: string): boolean {
		return mime.includes('zip') || mime.includes('tar') || /\.(zip|tar|gz|7z|rar|xmemos)$/i.test(name);
	}
	function isCode(mime: string, name: string): boolean {
		return (
			mime.includes('json') ||
			mime.includes('javascript') ||
			mime.includes('typescript') ||
			mime.includes('xml') ||
			/\.(ts|js|svelte|rs|py|json|md|html|css|sql|toml|yaml|yml)$/i.test(name)
		);
	}

	function resolveUrl(res: Resource, thumb: boolean): string {
		const local = previewUrls?.get(res.id);
		if (local) return local;
		return getMediaUrl(res.uid, thumb);
	}

	async function handleDownload(res: Resource): Promise<void> {
		if (downloadingUid === res.uid) return;
		downloadingUid = res.uid;
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
		} finally {
			downloadingUid = null;
		}
	}
</script>

{#if resources.length > 0}
	<div
		class="mt-3 grid gap-2.5 {resources.length === 1 && !compact
			? 'grid-cols-1'
			: 'grid-cols-2 sm:grid-cols-3'}"
	>
		{#each resources as res (res.id)}
			{#if isImage(res.type)}
				<button
					type="button"
					aria-label="Preview image {res.filename}"
					class="group relative overflow-hidden rounded-2xl border border-[var(--xm-divider)] bg-[var(--xm-surface-subtle)] aspect-video max-h-[380px] lg:max-h-[420px] w-full text-left"
					onclick={() => {
						const imgs = resources.filter((item) => isImage(item.type));
						const gallery = imgs.map((item) => ({
							uid: item.uid,
							src: resolveUrl(item, false),
							filename: item.filename,
							size: item.size,
							mimeType: item.type
						}));
						const idx = Math.max(0, imgs.findIndex((item) => item.id === res.id));
						lightboxState.open(gallery[idx]!, gallery, idx);
					}}
				>
					<img
						src={resolveUrl(res, true)}
						alt={res.filename}
						loading="lazy"
						decoding="async"
						fetchpriority="low"
						class="h-full w-full object-cover transition-transform duration-200 group-hover:scale-[1.02]"
					/>
					<span
						class="pointer-events-none absolute right-2 bottom-2 flex h-7 w-7 items-center justify-center rounded-full bg-black/60 text-white opacity-0 transition-opacity group-hover:opacity-100 group-focus-visible:opacity-100"
						aria-hidden="true"
					>
						<ZoomIn class="h-3.5 w-3.5" />
					</span>
				</button>
			{:else if isVideo(res.type)}
				<div
					class="overflow-hidden rounded-2xl border border-[var(--xm-divider)] bg-black/90 p-1.5"
				>
					<!-- svelte-ignore a11y_media_has_caption -->
					<video
						src={resolveUrl(res, false)}
						controls
						preload="none"
						class="max-h-64 w-full rounded-xl"
					></video>
					<div class="mt-1 flex items-center justify-between gap-2 px-1.5 text-[11px] text-slate-200">
						<span class="truncate font-medium">{res.filename}</span>
						<div class="flex shrink-0 items-center gap-1.5">
							{#if res.size > 0}
								<span class="tabular-nums text-slate-300">{formatBytes(res.size)}</span>
							{/if}
							<button
								type="button"
								aria-label="Download {res.filename}"
								title="Download {res.filename}"
								disabled={downloadingUid === res.uid}
								class="flex h-6 w-6 items-center justify-center rounded-full text-slate-200 transition-colors hover:bg-white/15 disabled:opacity-50"
								onclick={() => handleDownload(res)}
							>
								<Download class="h-3.5 w-3.5" aria-hidden="true" />
							</button>
						</div>
					</div>
				</div>
			{:else if isAudio(res.type)}
				<div
					class="col-span-full flex items-center gap-3 rounded-2xl border border-[var(--xm-divider)] bg-[var(--xm-surface-subtle)] p-3"
				>
					<div
						class="flex h-10 w-10 shrink-0 items-center justify-center rounded-xl bg-[var(--xm-accent-soft)] text-[var(--xm-accent)]"
					>
						<Music class="h-5 w-5" aria-hidden="true" />
					</div>
					<div class="min-w-0 flex-1">
						<div class="flex items-center justify-between gap-2">
							<span class="truncate text-xs font-semibold xm-text-primary">{res.filename}</span>
							<div class="flex shrink-0 items-center gap-1.5">
								{#if res.size > 0}
									<span class="text-[11px] xm-text-muted tabular-nums">
										{formatBytes(res.size)}
									</span>
								{/if}
								<button
									type="button"
									aria-label="Download {res.filename}"
									title="Download {res.filename}"
									disabled={downloadingUid === res.uid}
									class="flex h-7 w-7 items-center justify-center rounded-full xm-text-accent transition-colors hover:bg-[var(--xm-accent-soft)] disabled:opacity-50"
									onclick={() => handleDownload(res)}
								>
									<Download class="h-3.5 w-3.5" aria-hidden="true" />
								</button>
							</div>
						</div>
						<audio src={resolveUrl(res, false)} controls preload="none" class="mt-1.5 h-8 w-full"></audio>
					</div>
				</div>
			{:else}
				<div
					class="flex items-center gap-2.5 rounded-2xl border border-[var(--xm-divider)] bg-[var(--xm-surface-subtle)] px-3 py-2.5"
				>
					<div
						class="flex h-9 w-9 shrink-0 items-center justify-center rounded-xl bg-[var(--xm-accent-soft)] text-[var(--xm-accent)]"
					>
						{#if res.type.includes('video')}
							<Film class="h-4 w-4" aria-hidden="true" />
						{:else if isArchive(res.type, res.filename)}
							<FileArchive class="h-4 w-4" aria-hidden="true" />
						{:else if isCode(res.type, res.filename)}
							<FileCode class="h-4 w-4" aria-hidden="true" />
						{:else}
							<FileText class="h-4 w-4" aria-hidden="true" />
						{/if}
					</div>
					<div class="min-w-0 flex-1">
						<div class="truncate text-xs font-semibold xm-text-primary">{res.filename}</div>
						<div class="text-[11px] xm-text-muted tabular-nums">
							{res.size > 0 ? formatBytes(res.size) : res.type || 'File'}
						</div>
					</div>
					<button
						type="button"
						aria-label="Download {res.filename}"
						title="Download {res.filename}"
						disabled={downloadingUid === res.uid}
						class="flex h-8 w-8 shrink-0 items-center justify-center rounded-full xm-text-accent transition-colors hover:bg-[var(--xm-accent-soft)] disabled:opacity-50"
						onclick={() => handleDownload(res)}
					>
						<Download class="h-4 w-4" aria-hidden="true" />
					</button>
				</div>
			{/if}
		{/each}
	</div>
{/if}
