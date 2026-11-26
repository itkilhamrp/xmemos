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
	import { Preloader } from 'konsta/svelte';
	import {
		File as FileIcon,
		FileArchive,
		FileCode,
		FileText,
		Film,
		Music,
		X,
		ZoomIn
	} from 'lucide-svelte';

	import { getMediaUrl } from '#lib/api/invoke.ts';
	import type { Resource } from '#lib/types/resource.ts';
	import { i18n } from '#lib/i18n/index.svelte.ts';
	import { lightboxState } from '#lib/state/lightbox.svelte.ts';
	import { formatBytes } from '#lib/utils/date.ts';

	export interface UploadProgressItem {
		id: string;
		filename: string;
		sizeBytes: number;
		percent: number;
		stage: 'compressing' | 'uploading';
	}

	interface Props {
		resources: Resource[];
		previewUrls?: ReadonlyMap<number, string>;
		uploadingCount?: number;
		activeUploads?: UploadProgressItem[];
		onRemove: (resourceId: number) => void;
	}

	let { resources, previewUrls, uploadingCount = 0, activeUploads = [], onRemove }: Props = $props();


	function isImage(mime: string): boolean {
		return mime.startsWith('image/');
	}
	function isVideo(mime: string): boolean {
		return mime.startsWith('video/');
	}
	function isAudio(mime: string): boolean {
		return mime.startsWith('audio/');
	}
	function isArchive(mime: string, filename: string): boolean {
		return (
			mime.includes('zip') ||
			mime.includes('tar') ||
			/\.(zip|tar|gz|7z|rar|xmemos)$/i.test(filename)
		);
	}
	function isCode(mime: string, filename: string): boolean {
		return (
			mime.includes('json') ||
			mime.includes('javascript') ||
			mime.includes('typescript') ||
			mime.includes('xml') ||
			/\.(ts|js|svelte|rs|py|json|md|html|css|sql|toml|yaml|yml)$/i.test(filename)
		);
	}
	function isDocument(mime: string, filename: string): boolean {
		return (
			mime.includes('pdf') ||
			mime.includes('word') ||
			mime.includes('text/') ||
			/\.(pdf|doc|docx|txt|rtf|csv|xlsx|pptx)$/i.test(filename)
		);
	}

	function getFileExtension(filename: string, mime: string): string {
		const dot = filename.lastIndexOf('.');
		if (dot > 0 && dot < filename.length - 1) {
			return filename.slice(dot + 1).toUpperCase().slice(0, 6);
		}
		const slash = mime.indexOf('/');
		if (slash > -1) {
			return mime.slice(slash + 1).toUpperCase().slice(0, 6);
		}
		return 'FILE';
	}

	function resolveUrl(res: Resource, thumb: boolean): string {
		const local = previewUrls?.get(res.id);
		if (local) return local;
		return getMediaUrl(res.uid, thumb);
	}

</script>


{#if resources.length > 0 || uploadingCount > 0}
	<section class="space-y-2" aria-label="Uploaded attachment previews">
		<div class="flex items-center justify-between px-0.5 text-xs font-semibold xm-text-secondary">
			<span>
				Attachments ({resources.length}{uploadingCount > 0 ? ` • +${uploadingCount} uploading` : ''})
			</span>
			<span class="text-[11px] font-normal xm-text-muted">Preview before saving</span>
		</div>

		<div class="grid grid-cols-2 gap-2.5 sm:grid-cols-3">
			{#each resources as res (res.id)}
				{#if isImage(res.type)}
					<div
						class="group relative overflow-hidden rounded-2xl border border-[var(--xm-divider)] bg-[var(--xm-surface-subtle)]"
					>
						<button
							type="button"
							aria-label="Inspect uploaded image {res.filename}"
							class="relative block aspect-[4/3] w-full overflow-hidden bg-black/5 dark:bg-white/5"
							onclick={() => lightboxState.open({ src: resolveUrl(res, false), filename: res.filename, size: res.size })}
						>
							<img
								src={resolveUrl(res, true)}
								alt={res.filename}
								loading="lazy"
								decoding="async"
								class="h-full w-full object-cover transition-transform duration-200 group-hover:scale-[1.03]"
							/>
							<span
								class="pointer-events-none absolute right-2 bottom-2 flex h-7 w-7 items-center justify-center rounded-full bg-black/65 text-white shadow"
								aria-hidden="true"
							>
								<ZoomIn class="h-3.5 w-3.5" />
							</span>
						</button>
						<div class="flex items-center justify-between gap-1.5 px-2.5 py-1.5">
							<div class="min-w-0">
								<div class="truncate text-[11px] font-semibold xm-text-primary">{res.filename}</div>
								<div class="text-[10px] xm-text-muted tabular-nums">
									{res.size > 0 ? formatBytes(res.size) : 'Image'}
								</div>
							</div>
						</div>
						<button
							type="button"
							aria-label="Remove attachment {res.filename}"
							class="absolute top-1.5 right-1.5 flex h-7 w-7 items-center justify-center rounded-full bg-black/70 text-white shadow transition-colors hover:bg-rose-600"
							onclick={() => onRemove(res.id)}
						>
							<X class="h-3.5 w-3.5" aria-hidden="true" />
						</button>
					</div>
				{:else if isVideo(res.type)}
					<div
						class="relative col-span-2 overflow-hidden rounded-2xl border border-[var(--xm-divider)] bg-black/90 p-1.5 sm:col-span-3"
					>
						<!-- svelte-ignore a11y_media_has_caption -->
						<video
							src={resolveUrl(res, false)}
							controls
							preload="metadata"
							class="max-h-44 w-full rounded-xl"
						></video>
						<div class="mt-1 flex items-center justify-between gap-2 px-1.5 text-[11px] text-slate-200">
							<span class="inline-flex min-w-0 items-center gap-1.5">
								<Film class="h-3.5 w-3.5 shrink-0 text-sky-400" aria-hidden="true" />
								<span class="truncate font-medium">{res.filename}</span>
							</span>
							<div class="flex shrink-0 items-center gap-2">
								{#if res.size > 0}
									<span class="tabular-nums text-slate-300">{formatBytes(res.size)}</span>
								{/if}
								<button
									type="button"
									aria-label="Remove attachment {res.filename}"
									class="flex h-7 w-7 items-center justify-center rounded-full bg-white/15 text-white hover:bg-rose-600"
									onclick={() => onRemove(res.id)}
								>
									<X class="h-3.5 w-3.5" aria-hidden="true" />
								</button>
							</div>
						</div>
					</div>
				{:else if isAudio(res.type)}
					<div
						class="relative col-span-2 flex items-center gap-2.5 rounded-2xl border border-[var(--xm-divider)] bg-[var(--xm-surface-subtle)] p-2.5 sm:col-span-3"
					>
						<div
							class="flex h-9 w-9 shrink-0 items-center justify-center rounded-xl bg-[var(--xm-accent-soft)] text-[var(--xm-accent)]"
						>
							<Music class="h-4 w-4" aria-hidden="true" />
						</div>
						<div class="min-w-0 flex-1">
							<div class="flex items-center justify-between gap-2">
								<span class="truncate text-xs font-semibold xm-text-primary">{res.filename}</span>
								{#if res.size > 0}
									<span class="shrink-0 text-[11px] xm-text-muted tabular-nums">
										{formatBytes(res.size)}
									</span>
								{/if}
							</div>
							<audio src={resolveUrl(res, false)} controls class="mt-1 h-8 w-full"></audio>
						</div>
						<button
							type="button"
							aria-label="Remove attachment {res.filename}"
							class="flex h-8 w-8 shrink-0 items-center justify-center rounded-full xm-text-danger hover:bg-[var(--xm-danger-soft)]"
							onclick={() => onRemove(res.id)}
						>
							<X class="h-4 w-4" aria-hidden="true" />
						</button>
					</div>
				{:else}
					<div
						class="relative flex items-center gap-2.5 rounded-2xl border border-[var(--xm-divider)] bg-[var(--xm-surface-subtle)] p-2.5 pr-9"
					>
						<div
							class="flex h-10 w-10 shrink-0 flex-col items-center justify-center rounded-xl bg-[var(--xm-accent-soft)] text-[var(--xm-accent)]"
						>
							{#if isArchive(res.type, res.filename)}
								<FileArchive class="h-4 w-4" aria-hidden="true" />
							{:else if isCode(res.type, res.filename)}
								<FileCode class="h-4 w-4" aria-hidden="true" />
							{:else if isDocument(res.type, res.filename)}
								<FileText class="h-4 w-4" aria-hidden="true" />
							{:else}
								<FileIcon class="h-4 w-4" aria-hidden="true" />
							{/if}
							<span class="mt-0.5 text-[8px] font-bold leading-none">
								{getFileExtension(res.filename, res.type)}
							</span>
						</div>
						<div class="min-w-0 flex-1">
							<div class="truncate text-xs font-semibold xm-text-primary">{res.filename}</div>
							<div class="text-[11px] xm-text-muted tabular-nums">
								{res.size > 0 ? formatBytes(res.size) : res.type || 'Static file'}
							</div>
						</div>
						<button
							type="button"
							aria-label="Remove attachment {res.filename}"
							class="absolute top-1.5 right-1.5 flex h-7 w-7 items-center justify-center rounded-full xm-text-secondary transition-colors hover:bg-[var(--xm-danger-soft)] hover:text-[var(--xm-danger)]"
							onclick={() => onRemove(res.id)}
						>
							<X class="h-3.5 w-3.5" aria-hidden="true" />
						</button>
					</div>
				{/if}
			{/each}

			{#if activeUploads.length > 0}
				{#each activeUploads as item (item.id)}
					<div
						class="col-span-2 flex flex-col gap-2 rounded-2xl border border-[var(--xm-accent-border)] bg-[var(--xm-accent-soft)] p-3 sm:col-span-3"
						role="status"
						aria-live="polite"
					>
						<div class="flex items-center justify-between gap-2">
							<div class="flex min-w-0 items-center gap-2">
								<Preloader class="!h-4 !w-4 shrink-0" />
								<span class="truncate text-xs font-semibold xm-text-primary">{item.filename}</span>
							</div>
							<span class="shrink-0 text-[11px] font-semibold xm-text-accent tabular-nums">
								{Math.round(item.percent)}%
							</span>
						</div>
						<div class="h-1.5 w-full overflow-hidden rounded-full bg-black/10 dark:bg-white/15">
							<div
								class="h-full rounded-full bg-[var(--xm-accent)] transition-[width] duration-200 ease-out"
								style:width={`${Math.max(8, Math.min(100, item.percent))}%`}
							></div>
						</div>
						<div class="flex items-center justify-between text-[10px] xm-text-muted tabular-nums">
							<span>
								{item.stage === 'compressing'
									? i18n.t('editor.compressingFile')
									: i18n.t('editor.uploadingFile')}
							</span>
							{#if item.sizeBytes > 0}
								<span>{formatBytes(item.sizeBytes)}</span>
							{/if}
						</div>
					</div>
				{/each}
			{:else if uploadingCount > 0}
				<div
					class="flex min-h-[96px] flex-col items-center justify-center gap-2 rounded-2xl border border-dashed border-[var(--xm-accent-border)] bg-[var(--xm-accent-soft)] p-3 text-center"
					role="status"
					aria-label="Uploading {uploadingCount} file(s)"
				>
					<Preloader class="!h-5 !w-5" />
					<span class="text-[11px] font-semibold xm-text-accent">
						{i18n.t('editor.uploading', { count: uploadingCount })}
					</span>
				</div>
			{/if}
		</div>
	</section>
{/if}
