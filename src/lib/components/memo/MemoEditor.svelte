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
	import { i18n } from '#lib/i18n/index.svelte.ts';
	import { onDestroy, untrack } from 'svelte';
	import { Button, Segmented, SegmentedButton } from 'konsta/svelte';
	import {
		Bold,
		Calendar,
		CheckSquare,
		Clock,
		Code,
		Eye,
		Hash,
		Heading2,
		Italic,
		Link2,
		LocateFixed,
		MapPin,
		Paperclip,
		PencilLine,
		Quote,
		RotateCcw,
		Sigma,
		UploadCloud,
		X
	} from 'lucide-svelte';

	import { memoApi } from '#lib/api/memo.ts';
	import { resourceApi } from '#lib/api/resource.ts';
	import { systemApi } from '#lib/api/system.ts';
	import type { MemoLocation, MemoView, MemoVisibility } from '#lib/types/memo.ts';
	import type { Resource } from '#lib/types/resource.ts';
	import { memosState } from '#lib/state/memos.svelte.ts';
	import { toastState } from '#lib/state/toast.svelte.ts';
	import { isCoordinateOnlyLabel, reverseGeocodeNearestPlace } from '#lib/utils/geocode.ts';
	import { formatCurrentLocalDateTimeInput, formatDateKeyWithCurrentTime, formatLocalDateTimeInput, parseLocalDateTimeInput } from '#lib/utils/date.ts';
	import { prepareSmartUpload } from '#lib/utils/media-compress.ts';
	import MemoContent from './MemoContent.svelte';
	import AttachmentGrid from '#lib/components/media/AttachmentGrid.svelte';
	import AttachmentPreviewStrip, { type UploadProgressItem } from '#lib/components/media/AttachmentPreviewStrip.svelte';
	import OsmMapCanvas from '#lib/components/map/OsmMapCanvas.svelte';

	interface Props {
		opened: boolean;
		editingMemo?: MemoView | null;
		parentMemoId?: number | null;
		isComment?: boolean;
		initialDate?: string | null;
		initialLocation?: MemoLocation | null;
		onClose: () => void;
		onSaved?: (saved: MemoView) => void;
	}

	let {
		opened,
		editingMemo = null,
		parentMemoId = null,
		isComment = false,
		initialDate = null,
		initialLocation = null,
		onClose,
		onSaved
	}: Props = $props();

	let isCommentMode = $derived(
		Boolean(parentMemoId) ||
			isComment ||
			Boolean(editingMemo?.relations.some((r) => r.memoId === editingMemo?.memo.id && r.type === 'COMMENT'))
	);
	let editorTab = $state<'write' | 'preview'>('write');
	let content = $state('');
	let visibility = $state<MemoVisibility>('PRIVATE');
	let customDateStr = $state('');
	let originalDateStr = $state('');
	let useCustomTimestamp = $state(false);
	let locationPickerOpened = $state(false);
	let locationLabel = $state('');
	let locationLat = $state('');
	let locationLng = $state('');
	let hadOriginalLocation = $state(false);
	let locatingGps = $state(false);
	let attachedResources = $state<Resource[]>([]);
	let localPreviewUrls = $state<Map<number, string>>(new Map());
	let activeUploads = $state<UploadProgressItem[]>([]);
	let uploadingCount = $derived(activeUploads.length);
	let maxUploadSizeMb = $state<number>(32);
	let uploadValidationError = $state<string | null>(null);
	let dragActive = $state(false);
	let saving = $state(false);
	let fileInputEl = $state<HTMLInputElement | null>(null);
	let textareaEl = $state<HTMLTextAreaElement | null>(null);

	let charCount = $derived(content.length);

	let parsedLocation = $derived.by((): MemoLocation | null => {
		const lat = Number.parseFloat(locationLat);
		const lng = Number.parseFloat(locationLng);
		if (!Number.isFinite(lat) || !Number.isFinite(lng)) return null;
		if (lat < -90 || lat > 90 || lng < -180 || lng > 180) return null;
		return {
			placeholder: locationLabel.trim() || `${lat.toFixed(5)}, ${lng.toFixed(5)}`,
			latitude: lat,
			longitude: lng
		};
	});

	function clearLocation(): void {
		locationLabel = '';
		locationLat = '';
		locationLng = '';
		locationPickerOpened = false;
	}

	let resolvingPlace = $state(false);
	let autoFilledLocationLabel = $state(true);
	let geocodeSeq = 0;

	async function resolveLocationLabel(lat: number, lng: number, forceOverwrite = false): Promise<void> {
		const shouldUpdate =
			forceOverwrite || autoFilledLocationLabel || isCoordinateOnlyLabel(locationLabel);
		if (!shouldUpdate) return;
		const seq = ++geocodeSeq;
		resolvingPlace = true;
		const place = await reverseGeocodeNearestPlace(lat, lng, i18n.bcp47);
		if (seq !== geocodeSeq) return;
		resolvingPlace = false;
		if (place) {
			locationLabel = place;
			autoFilledLocationLabel = true;
		} else if (!locationLabel.trim()) {
			locationLabel = `${lat.toFixed(4)}, ${lng.toFixed(4)}`;
			autoFilledLocationLabel = true;
		}
	}

	function detectCurrentLocation(): void {
		if (typeof navigator === 'undefined' || !navigator.geolocation) {
			toastState.show(i18n.t('map.gpsUnavailable'), 'info');
			return;
		}
		locatingGps = true;
		navigator.geolocation.getCurrentPosition(
			(pos) => {
				locationLat = pos.coords.latitude.toFixed(6);
				locationLng = pos.coords.longitude.toFixed(6);
				locatingGps = false;
				void resolveLocationLabel(pos.coords.latitude, pos.coords.longitude);
			},
			() => {
				locatingGps = false;
				toastState.show(i18n.t('map.gpsUnavailable'), 'info');
			},
			{ enableHighAccuracy: true, timeout: 8000 }
		);
	}

	let customTimestampKind = $derived.by((): 'past' | 'future' | null => {
		if (!useCustomTimestamp || !customDateStr) return null;
		const parsed = parseLocalDateTimeInput(customDateStr);
		if (parsed === undefined) return null;
		const diff = parsed - Math.floor(Date.now() / 1000);
		if (diff < -120) return 'past';
		if (diff > 120) return 'future';
		return null;
	});

	let activeTagPrefix = $derived.by(() => {
		const match = /(?:^|\s)#([\w\-/]*)$/.exec(content);
		return match ? (match[1] ?? '') : null;
	});

	let tagSuggestions = $derived.by(() => {
		if (activeTagPrefix === null) return [];
		const q = activeTagPrefix.toLowerCase();
		return memosState.tags.filter((t) => t.name.toLowerCase().startsWith(q)).slice(0, 6);
	});

	function revokeAllPreviewUrls(): void {
		const current = untrack(() => localPreviewUrls);
		if (current.size === 0) return;
		for (const url of current.values()) {
			URL.revokeObjectURL(url);
		}
		localPreviewUrls = new Map();
	}

	function revokePreviewUrl(resourceId: number): void {
		const existing = localPreviewUrls.get(resourceId);
		if (existing) {
			URL.revokeObjectURL(existing);
			const next = new Map(localPreviewUrls);
			next.delete(resourceId);
			localPreviewUrls = next;
		}
	}

	onDestroy(() => {
		revokeAllPreviewUrls();
	});

	$effect(() => {
		if (!opened) return;
		const curEditing = editingMemo;
		const curInitialDate = initialDate;
		const curInitialLocation = initialLocation;
		void [curEditing, curInitialDate, curInitialLocation];
		untrack(() => {
		editorTab = 'write';
		dragActive = false;
		uploadValidationError = null;
		void systemApi
			.listSettings()
			.then((settings) => {
				const found = settings.find((s) => s.name === 'max-upload-size-mb');
				if (found) {
					const parsed = Number.parseInt(found.value, 10);
					if (Number.isFinite(parsed) && parsed >= 0) {
						maxUploadSizeMb = parsed;
					}
				}
			})
			.catch(() => {});
		if (curEditing) {
			revokeAllPreviewUrls();
			content = editingMemo.memo.content;
			visibility = editingMemo.memo.visibility;
			attachedResources = [...editingMemo.resources];
			const formatted = formatLocalDateTimeInput(editingMemo.memo.createdTs);
			customDateStr = formatted;
			originalDateStr = formatted;
			useCustomTimestamp = true;
			const existingLoc = editingMemo.memo.payload.location;
			if (existingLoc) {
				locationLabel = existingLoc.placeholder || '';
				locationLat = String(existingLoc.latitude);
				locationLng = String(existingLoc.longitude);
				hadOriginalLocation = true;
			} else {
				locationLabel = '';
				locationLat = '';
				locationLng = '';
				hadOriginalLocation = false;
			}
			locationPickerOpened = false;
		} else {
			originalDateStr = '';
			if (initialDate) {
				customDateStr = formatDateKeyWithCurrentTime(initialDate);
				useCustomTimestamp = true;
			} else {
				customDateStr = '';
				useCustomTimestamp = false;
			}
			if (initialLocation) {
				locationLabel = initialLocation.placeholder || '';
				locationLat = String(initialLocation.latitude);
				locationLng = String(initialLocation.longitude);
				autoFilledLocationLabel = isCoordinateOnlyLabel(locationLabel);
				locationPickerOpened = true;
				if (autoFilledLocationLabel) {
					void resolveLocationLabel(initialLocation.latitude, initialLocation.longitude, true);
				}
			} else {
				locationLabel = '';
				locationLat = '';
				locationLng = '';
				locationPickerOpened = false;
			}
			hadOriginalLocation = false;
			attachedResources = [];
			revokeAllPreviewUrls();
			void memoApi
				.loadDraft()
				.then((draft) => {
					if (draft && !editingMemo) {
						content = draft.content;
						visibility = draft.visibility;
					} else {
						content = '';
					}
				})
				.catch(() => {
					content = '';
				});
		}
		});
	});

	// Autosave draft to xmemos_local.db when composing a new memo
	let draftTimer: ReturnType<typeof setTimeout> | null = null;
	$effect(() => {
		if (!opened || editingMemo || parentMemoId) return;
		const currentContent = content;
		const currentVis = visibility;
		const resIds = attachedResources.map((r) => r.id);
		if (draftTimer) clearTimeout(draftTimer);
		draftTimer = setTimeout(() => {
			void memoApi.saveDraft(currentContent, currentVis, resIds, []).catch(() => {});
		}, 450);
	});

	function insertSnippet(before: string, after = ''): void {
		if (editorTab !== 'write') {
			editorTab = 'write';
		}
		if (!textareaEl) {
			content += `${before}${after}`;
			return;
		}
		const start = textareaEl.selectionStart;
		const end = textareaEl.selectionEnd;
		const selected = content.slice(start, end);
		content = content.slice(0, start) + before + selected + after + content.slice(end);
		queueMicrotask(() => {
			if (textareaEl) {
				textareaEl.focus();
				const cursor = start + before.length + selected.length;
				textareaEl.setSelectionRange(cursor, cursor);
			}
		});
	}

	function applyTagSuggestion(tag: string): void {
		content = content.replace(/(^|\s)#[\w\-/]*$/, `$1#${tag} `);
		textareaEl?.focus();
	}

	function checkUploadSizeLimit(filename: string, sizeBytes: number): boolean {
		if (maxUploadSizeMb <= 0) return true;
		const limitBytes = maxUploadSizeMb * 1024 * 1024;
		if (sizeBytes > limitBytes) {
			const actualMb = (sizeBytes / (1024 * 1024)).toFixed(1);
			const msg = i18n.t('editor.uploadSizeExceeded', {
				filename,
				size: actualMb,
				limit: String(maxUploadSizeMb)
			});
			uploadValidationError = msg;
			toastState.show(msg, 'error');
			return false;
		}
		return true;
	}

	function startProgressTicker(uploadId: string, maxPercent = 92): ReturnType<typeof setInterval> {
		return setInterval(() => {
			activeUploads = activeUploads.map((item) =>
				item.id === uploadId && item.percent < maxPercent
					? { ...item, percent: Math.min(maxPercent, item.percent + Math.max(2, (maxPercent - item.percent) * 0.18)) }
					: item
			);
		}, 180);
	}

	function updateUploadItem(uploadId: string, patch: Partial<UploadProgressItem>): void {
		activeUploads = activeUploads.map((item) => (item.id === uploadId ? { ...item, ...patch } : item));
	}

	function removeUploadItem(uploadId: string): void {
		activeUploads = activeUploads.filter((item) => item.id !== uploadId);
	}

	async function uploadFiles(files: File[]): Promise<void> {
		if (files.length === 0) return;
		uploadValidationError = null;
		const validFiles = files.filter((file) => checkUploadSizeLimit(file.name, file.size));
		if (validFiles.length === 0) return;

		for (const file of validFiles) {
			const uploadId = `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
			const isCompressible = file.type.startsWith('image/') || file.type.startsWith('video/');
			activeUploads = [
				...activeUploads,
				{
					id: uploadId,
					filename: file.name,
					sizeBytes: file.size,
					percent: 12,
					stage: isCompressible ? 'compressing' : 'uploading'
				}
			];
			const ticker = startProgressTicker(uploadId, 92);
			const shouldPreviewObjectUrl =
				file.type.startsWith('image/') ||
				file.type.startsWith('video/') ||
				file.type.startsWith('audio/');
			const objectUrl = shouldPreviewObjectUrl ? URL.createObjectURL(file) : null;
			try {
				const prepared = await prepareSmartUpload(file);
				updateUploadItem(uploadId, { stage: 'uploading', percent: 62 });
				const res = await resourceApi.uploadBytes({
					filename: prepared.filename,
					mimeType: prepared.mimeType,
					base64Data: prepared.base64Data
				});
				updateUploadItem(uploadId, { percent: 100 });
				if (objectUrl) {
					const next = new Map(localPreviewUrls);
					next.set(res.id, objectUrl);
					localPreviewUrls = next;
				}
				attachedResources = [...attachedResources, res];
				toastState.show(`Attached ${res.filename}`, 'success');
			} catch (err) {
				if (objectUrl) URL.revokeObjectURL(objectUrl);
				const msg = err instanceof Error ? err.message : String(err);
				uploadValidationError = msg;
				toastState.show(`Upload failed: ${msg}`, 'error');
			} finally {
				clearInterval(ticker);
				removeUploadItem(uploadId);
			}
		}
	}

	async function handlePickAttachment(): Promise<void> {
		uploadValidationError = null;
		try {
			const picked = await resourceApi.pickDocument(
				['image/*', 'video/*', 'audio/*', 'application/pdf', '*/*'],
				true
			);
			if (!picked.cancelled && picked.localPath && picked.filename) {
				if (picked.size > 0 && !checkUploadSizeLimit(picked.filename, picked.size)) {
					return;
				}
				const uploadId = `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;
				const isCompressible = Boolean(picked.mimeType?.startsWith('image/') || picked.mimeType?.startsWith('video/'));
				activeUploads = [
					...activeUploads,
					{
						id: uploadId,
						filename: picked.filename,
						sizeBytes: picked.size ?? 0,
						percent: 20,
						stage: isCompressible ? 'compressing' : 'uploading'
					}
				];
				const ticker = startProgressTicker(uploadId, 92);
				try {
					const res = await resourceApi.importFile({
						localPath: picked.localPath,
						filename: picked.filename,
						mimeType: picked.mimeType ?? undefined
					});
					updateUploadItem(uploadId, { percent: 100 });
					attachedResources = [...attachedResources, res];
					toastState.show(`Attached ${res.filename}`, 'success');
				} catch (err) {
					const msg = err instanceof Error ? err.message : String(err);
					uploadValidationError = msg;
					toastState.show(`Upload failed: ${msg}`, 'error');
				} finally {
					clearInterval(ticker);
					removeUploadItem(uploadId);
				}
				return;
			}
		} catch {
			// Fallback to HTML file input on desktop/browser
		}
		fileInputEl?.click();
	}

	async function handleFileInputChange(event: Event): Promise<void> {
		const input = event.currentTarget as HTMLInputElement;
		const files = input.files ? Array.from(input.files) : [];
		input.value = '';
		await uploadFiles(files);
	}

	async function handlePaste(event: ClipboardEvent): Promise<void> {
		const items = event.clipboardData?.files;
		if (!items || items.length === 0) return;
		const files = Array.from(items);
		if (files.length > 0) {
			event.preventDefault();
			await uploadFiles(files);
		}
	}

	async function handleDrop(event: DragEvent): Promise<void> {
		event.preventDefault();
		dragActive = false;
		const files = event.dataTransfer?.files ? Array.from(event.dataTransfer.files) : [];
		if (files.length > 0) {
			await uploadFiles(files);
		}
	}

	function handleRemoveAttachment(resourceId: number): void {
		revokePreviewUrl(resourceId);
		attachedResources = attachedResources.filter((r) => r.id !== resourceId);
	}

	function applyLinePrefix(prefix: string, stripHeading = false): void {
		if (!textareaEl) {
			content += prefix;
			return;
		}
		const start = textareaEl.selectionStart;
		const end = textareaEl.selectionEnd;
		const lineStart = content.lastIndexOf('\n', Math.max(0, start - 1)) + 1;
		const lineEndIdx = content.indexOf('\n', end);
		const lineEnd = lineEndIdx === -1 ? content.length : lineEndIdx;
		const line = content.slice(lineStart, lineEnd);
		const cleaned = stripHeading
			? line.replace(/^#{1,6}\s+/, '')
			: line.replace(/^(?:#{1,6}\s+|[-*]\s+\[[ xX]\]\s+|[-*]\s+|\d+\.\s+)/, '');
		const nextLine = `${prefix}${cleaned}`;
		content = content.slice(0, lineStart) + nextLine + content.slice(lineEnd);
		const cursor = lineStart + nextLine.length;
		queueMicrotask(() => {
			textareaEl?.focus();
			textareaEl?.setSelectionRange(cursor, cursor);
		});
	}

	function handleEditorKeydown(event: KeyboardEvent): void {
		if (event.isComposing) return;
		const mod = event.metaKey || event.ctrlKey;
		const lower = event.key.toLowerCase();

		// Official Memos 2026 Editor Keymap (usememos.com/features/keyboard-shortcuts)
		if (mod && event.key === 'Enter') {
			event.preventDefault();
			void handleSubmit();
			return;
		}
		if (mod && event.shiftKey && !event.altKey && lower === 'p') {
			event.preventDefault();
			editorTab = editorTab === 'write' ? 'preview' : 'write';
			return;
		}
		if (mod && !event.shiftKey && !event.altKey) {
			if (lower === 'b') {
				event.preventDefault();
				insertSnippet('**', '**');
				return;
			}
			if (lower === 'i') {
				event.preventDefault();
				insertSnippet('*', '*');
				return;
			}
			if (lower === 'e') {
				event.preventDefault();
				insertSnippet('`', '`');
				return;
			}
		}
		if (mod && event.shiftKey && !event.altKey) {
			if (lower === 's') {
				event.preventDefault();
				insertSnippet('~~', '~~');
				return;
			}
			if (event.code === 'Digit7' || event.key === '&' || event.key === '7') {
				event.preventDefault();
				applyLinePrefix('1. ');
				return;
			}
			if (event.code === 'Digit8' || event.key === '*' || event.key === '8') {
				event.preventDefault();
				applyLinePrefix('- ');
				return;
			}
			if (event.code === 'Digit9' || event.key === '(' || event.key === '9') {
				event.preventDefault();
				applyLinePrefix('- [ ] ');
				return;
			}
		}
		if (mod && event.altKey && !event.shiftKey) {
			if (event.code === 'Digit1' || event.key === '1') {
				event.preventDefault();
				applyLinePrefix('# ', true);
				return;
			}
			if (event.code === 'Digit2' || event.key === '2') {
				event.preventDefault();
				applyLinePrefix('## ', true);
				return;
			}
			if (event.code === 'Digit3' || event.key === '3') {
				event.preventDefault();
				applyLinePrefix('### ', true);
				return;
			}
			if (event.code === 'Digit0' || event.key === '0') {
				event.preventDefault();
				applyLinePrefix('', true);
				return;
			}
			if (lower === 'c' || event.code === 'KeyC') {
				event.preventDefault();
				insertSnippet('```\n', '\n```');
				return;
			}
		}
		if (event.key === 'Tab' && textareaEl) {
			event.preventDefault();
			const start = textareaEl.selectionStart;
			const end = textareaEl.selectionEnd;
			const lineStart = content.lastIndexOf('\n', Math.max(0, start - 1)) + 1;
			if (event.shiftKey) {
				if (content.slice(lineStart, lineStart + 2) === '  ') {
					content = content.slice(0, lineStart) + content.slice(lineStart + 2);
					const nextPos = Math.max(lineStart, start - 2);
					queueMicrotask(() => textareaEl?.setSelectionRange(nextPos, Math.max(nextPos, end - 2)));
				} else if (content[lineStart] === '\t') {
					content = content.slice(0, lineStart) + content.slice(lineStart + 1);
					const nextPos = Math.max(lineStart, start - 1);
					queueMicrotask(() => textareaEl?.setSelectionRange(nextPos, Math.max(nextPos, end - 1)));
				}
			} else {
				content = content.slice(0, lineStart) + '  ' + content.slice(lineStart);
				queueMicrotask(() => textareaEl?.setSelectionRange(start + 2, end + 2));
			}
			return;
		}
		if (event.key === 'Escape' && textareaEl) {
			event.preventDefault();
			textareaEl.blur();
		}
	}

	async function handleSubmit(): Promise<void> {
		if (uploadingCount > 0) {
			toastState.show('Please wait for attachments to finish uploading', 'info');
			return;
		}
		if (!content.trim() && attachedResources.length === 0) {
			toastState.show('Write something or attach media first', 'info');
			return;
		}
		saving = true;
		try {
			const parsedTs =
				useCustomTimestamp && customDateStr ? parseLocalDateTimeInput(customDateStr) : undefined;
			const createdTs = parsedTs !== undefined && !Number.isNaN(parsedTs) ? parsedTs : undefined;
			const resourceIds = attachedResources.map((r) => r.id);

			// Ensure pinned location resolves to the nearest real region name before persisting to SQLite
			let finalLocation = parsedLocation ? { ...parsedLocation } : undefined;
			if (finalLocation && isCoordinateOnlyLabel(finalLocation.placeholder)) {
				const resolved = await reverseGeocodeNearestPlace(
					finalLocation.latitude,
					finalLocation.longitude,
					i18n.bcp47
				);
				if (resolved) {
					finalLocation.placeholder = resolved;
					locationLabel = resolved;
				}
			}

			let saved: MemoView;
			if (editingMemo) {
				const timestampChanged = useCustomTimestamp && customDateStr !== originalDateStr;
				saved = await memoApi.update({
					id: editingMemo.memo.id,
					content,
					visibility,
					createdTs: timestampChanged ? createdTs : undefined,
					location: finalLocation,
					clearLocation: hadOriginalLocation && !parsedLocation,
					resourceIds
				});
				memosState.replaceItem(saved);
				toastState.show(i18n.t('editor.updated'), 'success');
			} else if (parentMemoId) {
				saved = await memoApi.create({
					content,
					visibility,
					resourceIds,
					parentMemoId
				});
				toastState.show('Comment posted', 'success');
			} else {
				saved = await memosState.createMemo({
					content,
					visibility,
					createdTs: Number.isNaN(createdTs) ? undefined : createdTs,
					location: finalLocation,
					resourceIds
				});
				await memoApi.saveDraft('', visibility, [], []).catch(() => {});
				toastState.show('Memo saved offline', 'success');
			}
			content = '';
			attachedResources = [];
			revokeAllPreviewUrls();
			onSaved?.(saved);
			onClose();
		} catch (err) {
			toastState.show(err instanceof Error ? err.message : String(err), 'error');
		} finally {
			saving = false;
		}
	}
</script>

{#if opened}
	<div
		class="xm-kb-editor-backdrop fixed inset-0 z-[59] bg-black/45"
		aria-hidden="true"
		onclick={onClose}
	></div>
	<div
		role="dialog"
		aria-modal="true"
		aria-label={i18n.t(editingMemo ? 'editor.editMemo' : parentMemoId ? 'editor.addComment' : 'editor.newMemo')}
		class="xm-kb-editor-sheet fixed inset-x-0 z-[60] mx-auto flex w-full max-w-2xl flex-col overflow-hidden bg-[var(--xm-bg-base)] md:inset-x-4 md:bottom-4 md:top-auto md:max-h-[calc(100dvh-2rem)] md:rounded-3xl md:border md:border-[var(--xm-divider)] md:shadow-2xl"
	>
		<!-- Fixed Header: Safe-Area Top + Title, Close, Write/Preview Toggle, and Pinned Formatting Toolbar -->
		<div class="shrink-0 space-y-2.5 border-b border-[var(--xm-divider)] px-4 pt-[max(0.875rem,env(safe-area-inset-top))] pb-3 md:pt-3.5">
			<div class="flex items-center justify-between gap-2">
				<h2 class="text-base font-bold xm-text-primary">
					{i18n.t(editingMemo ? 'editor.editMemo' : parentMemoId ? 'editor.addComment' : 'editor.newMemo')}
				</h2>
				<button
					type="button"
					aria-label="Close memo editor"
					class="flex h-9 w-9 items-center justify-center rounded-full xm-text-secondary transition-colors hover:bg-[var(--xm-surface-subtle)]"
					onclick={onClose}
				>
					<X class="h-5 w-5" aria-hidden="true" />
				</button>
			</div>

			<!-- Konsta UI iOS Segmented Write / Preview Toggle Tab -->
			<div role="tablist" aria-label="Editor view mode">
				<Segmented rounded strong>
					<SegmentedButton
						role="tab"
						aria-selected={editorTab === 'write'}
						aria-controls="memo-editor-write-panel"
						active={editorTab === 'write'}
						onclick={() => (editorTab = 'write')}
					>
						<span class="inline-flex items-center gap-1.5">
							<PencilLine class="h-3.5 w-3.5" aria-hidden="true" />
							<span>{i18n.t('editor.write')}</span>
						</span>
					</SegmentedButton>
					<SegmentedButton
						role="tab"
						aria-selected={editorTab === 'preview'}
						aria-controls="memo-editor-preview-panel"
						active={editorTab === 'preview'}
						onclick={() => (editorTab = 'preview')}
					>
						<span class="inline-flex items-center gap-1.5">
							<Eye class="h-3.5 w-3.5" aria-hidden="true" />
							<span>{i18n.t('editor.preview')}</span>
						</span>
					</SegmentedButton>
				</Segmented>
			</div>

			{#if editorTab === 'write'}
				<!-- Pinned Markdown quick-format toolbar (always visible, never clipped when typing long memos) -->
				<div
					class="grid grid-cols-10 items-center gap-0.5 rounded-2xl border border-[var(--xm-divider)] bg-[var(--xm-surface-subtle)] p-1"
					role="toolbar"
					aria-label="Markdown formatting toolbar"
				>
					<button
						type="button"
						aria-label="Bold"
						title="Bold"
						class="flex h-9 w-full items-center justify-center rounded-xl xm-text-primary transition-colors hover:bg-[var(--xm-surface-hover)]"
						onclick={() => insertSnippet('**', '**')}
					>
						<Bold class="h-4 w-4" aria-hidden="true" />
					</button>
					<button
						type="button"
						aria-label="Italic"
						title="Italic"
						class="flex h-9 w-full items-center justify-center rounded-xl xm-text-primary transition-colors hover:bg-[var(--xm-surface-hover)]"
						onclick={() => insertSnippet('*', '*')}
					>
						<Italic class="h-4 w-4" aria-hidden="true" />
					</button>
					<button
						type="button"
						aria-label="Heading"
						title="Heading"
						class="flex h-9 w-full items-center justify-center rounded-xl xm-text-primary transition-colors hover:bg-[var(--xm-surface-hover)]"
						onclick={() => insertSnippet('## ')}
					>
						<Heading2 class="h-4 w-4" aria-hidden="true" />
					</button>
					<button
						type="button"
						aria-label="Checklist item"
						title="Checklist"
						class="flex h-9 w-full items-center justify-center rounded-xl xm-text-primary transition-colors hover:bg-[var(--xm-surface-hover)]"
						onclick={() => insertSnippet('- [ ] ')}
					>
						<CheckSquare class="h-4 w-4" aria-hidden="true" />
					</button>
					<button
						type="button"
						aria-label="Code block"
						title="Code block"
						class="flex h-9 w-full items-center justify-center rounded-xl xm-text-primary transition-colors hover:bg-[var(--xm-surface-hover)]"
						onclick={() => insertSnippet('```\n', '\n```')}
					>
						<Code class="h-4 w-4" aria-hidden="true" />
					</button>
					<button
						type="button"
						aria-label="Blockquote"
						title="Quote"
						class="flex h-9 w-full items-center justify-center rounded-xl xm-text-primary transition-colors hover:bg-[var(--xm-surface-hover)]"
						onclick={() => insertSnippet('> ')}
					>
						<Quote class="h-4 w-4" aria-hidden="true" />
					</button>
					<button
						type="button"
						aria-label="Insert tag"
						title="Tag"
						class="flex h-9 w-full items-center justify-center rounded-xl xm-text-primary transition-colors hover:bg-[var(--xm-surface-hover)]"
						onclick={() => insertSnippet('#')}
					>
						<Hash class="h-4 w-4" aria-hidden="true" />
					</button>
					<button
						type="button"
						aria-label="LaTeX math"
						title="LaTeX Math"
						class="flex h-9 w-full items-center justify-center rounded-xl xm-text-primary transition-colors hover:bg-[var(--xm-surface-hover)]"
						onclick={() => insertSnippet('$', '$')}
					>
						<Sigma class="h-4 w-4" aria-hidden="true" />
					</button>
					<button
						type="button"
						aria-label="Insert link"
						title="Link"
						class="flex h-9 w-full items-center justify-center rounded-xl xm-text-primary transition-colors hover:bg-[var(--xm-surface-hover)]"
						onclick={() => insertSnippet('[', '](https://)')}
					>
						<Link2 class="h-4 w-4" aria-hidden="true" />
					</button>
					{#if !isCommentMode}
						<button
							type="button"
							aria-label={i18n.t('editor.addLocation')}
							title={i18n.t('editor.addLocation')}
							aria-pressed={locationPickerOpened || Boolean(parsedLocation)}
							class="flex h-8 w-8 shrink-0 items-center justify-center rounded-lg transition-colors {locationPickerOpened || parsedLocation
								? 'bg-[var(--xm-accent-soft)] xm-text-accent'
								: 'xm-text-secondary hover:bg-[var(--xm-surface-hover)]'}"
							onclick={() => (locationPickerOpened = !locationPickerOpened)}
						>
							<MapPin class="h-4 w-4" aria-hidden="true" />
						</button>
					{/if}
					<button
						type="button"
						aria-label="Attach image or static file"
						title="Attach file"
						class="relative flex h-9 w-full items-center justify-center rounded-xl bg-[var(--xm-accent-soft)] xm-text-accent transition-opacity active:opacity-75"
						onclick={handlePickAttachment}
					>
						<Paperclip class="h-4 w-4" aria-hidden="true" />
						{#if attachedResources.length > 0}
							<span
								class="absolute -top-1 -right-1 flex h-4 min-w-[16px] items-center justify-center rounded-full bg-[var(--xm-accent)] px-1 text-[10px] font-bold text-white tabular-nums"
							>
								{attachedResources.length}
							</span>
						{/if}
					</button>
				</div>
			{/if}
		</div>

		<!-- Scrollable Composer Body -->
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			class="min-h-0 flex-1 space-y-3 overflow-y-auto overscroll-contain px-4 py-3"
			onkeydown={handleEditorKeydown}
			ondragover={(e) => {
				e.preventDefault();
				dragActive = true;
			}}
			ondragleave={() => (dragActive = false)}
			ondrop={handleDrop}
		>
			<input
				bind:this={fileInputEl}
				type="file"
				multiple
				class="hidden"
				onchange={handleFileInputChange}
			/>

			{#if dragActive}
				<div
					class="flex items-center justify-center gap-2 rounded-2xl border-2 border-dashed border-[var(--xm-accent)] bg-[var(--xm-accent-soft)] px-4 py-6 text-sm font-semibold xm-text-accent"
				>
					<UploadCloud class="h-5 w-5" aria-hidden="true" />
					<span>{i18n.t('editor.dropFiles')}</span>
				</div>
			{/if}

			{#if editorTab === 'write'}
				<div id="memo-editor-write-panel" role="tabpanel" class="space-y-3">
					<div>
						<label for="memo-editor-textarea" class="sr-only">Memo Markdown content</label>
						<textarea
							id="memo-editor-textarea"
							bind:this={textareaEl}
							bind:value={content}
							rows="7"
							onpaste={handlePaste}
							placeholder={i18n.t('editor.placeholder')}
							class="xm-input min-h-[150px] max-h-[55dvh] resize-y overflow-y-auto"
						></textarea>
						<div class="mt-1 flex items-center justify-between px-1 text-[11px] xm-text-muted tabular-nums">
							<span>{i18n.t('editor.hint')}</span>
							<span>{i18n.t('editor.chars', { count: charCount })}</span>
						</div>
					</div>

					<!-- Live #tag autocomplete -->
					{#if tagSuggestions.length > 0}
						<div class="flex flex-wrap gap-1.5" role="group" aria-label="Tag suggestions">
							{#each tagSuggestions as suggestion (suggestion.name)}
								<button
									type="button"
									class="rounded-full border border-[var(--xm-accent-border)] bg-[var(--xm-accent-soft)] px-3 py-1 text-xs font-semibold xm-text-accent"
									onclick={() => applyTagSuggestion(suggestion.name)}
								>
									#{suggestion.name}
								</button>
							{/each}
						</div>
					{/if}

					{#if uploadValidationError}
						<div
							role="alert"
							class="flex items-start justify-between gap-2 rounded-xl border border-rose-500/35 bg-rose-500/10 px-3 py-2 text-xs font-medium text-rose-700 dark:text-rose-300"
						>
							<span>{uploadValidationError}</span>
							<button
								type="button"
								aria-label={i18n.t('common.close')}
								class="shrink-0 rounded-full p-0.5 hover:bg-rose-500/15"
								onclick={() => (uploadValidationError = null)}
							>
								<X class="h-3.5 w-3.5" aria-hidden="true" />
							</button>
						</div>
					{/if}

					<!-- Pre-save static file & image upload preview strip -->
					<AttachmentPreviewStrip
						resources={attachedResources}
						previewUrls={localPreviewUrls}
						{uploadingCount}
						{activeUploads}
						onRemove={handleRemoveAttachment}
					/>

					<!-- Attached Location Pill -->
					{#if !parentMemoId && parsedLocation && !locationPickerOpened}
						<div class="flex items-center justify-between gap-2 rounded-xl border border-[var(--xm-accent-border)] bg-[var(--xm-accent-soft)] px-3 py-2 text-xs">
							<button
								type="button"
								class="flex min-w-0 flex-1 items-center gap-1.5 text-left font-medium xm-text-accent"
								onclick={() => (locationPickerOpened = true)}
							>
								<MapPin class="h-3.5 w-3.5 shrink-0" aria-hidden="true" />
								<span class="truncate">{parsedLocation.placeholder}</span>
							</button>
							<button
								type="button"
								aria-label={i18n.t('editor.removeLocation')}
								class="flex h-6 w-6 shrink-0 items-center justify-center rounded-full xm-text-accent hover:bg-[var(--xm-accent)]/15"
								onclick={clearLocation}
							>
								<X class="h-3.5 w-3.5" aria-hidden="true" />
							</button>
						</div>
					{/if}

					<!-- Collapsible Official Memos Location Picker (Map + Coordinates + GPS) -->
					{#if !parentMemoId && locationPickerOpened}
						<div class="space-y-2.5 rounded-2xl border border-[var(--xm-divider)] bg-[var(--xm-surface-subtle)] p-3">
							<div class="flex items-center justify-between gap-2">
								<span class="flex items-center gap-1.5 text-xs font-semibold xm-text-primary">
									<MapPin class="h-3.5 w-3.5 xm-text-accent" aria-hidden="true" />
									{i18n.t('editor.addLocation')}
								</span>
								<div class="flex items-center gap-1.5">
									<button
										type="button"
										disabled={locatingGps}
										class="inline-flex h-7 items-center gap-1 rounded-full bg-[var(--xm-accent-soft)] px-2.5 text-[11px] font-semibold xm-text-accent disabled:opacity-50"
										onclick={detectCurrentLocation}
									>
										<LocateFixed class="h-3 w-3" aria-hidden="true" />
										<span>{i18n.t('map.useGps')}</span>
									</button>
									{#if parsedLocation || locationLabel}
										<button
											type="button"
											class="inline-flex h-7 items-center rounded-full px-2 text-[11px] font-medium xm-text-danger hover:bg-[var(--xm-danger-soft)]"
											onclick={clearLocation}
										>
											{i18n.t('editor.removeLocation')}
										</button>
									{/if}
								</div>
							</div>

							<div class="h-40 w-full">
								<OsmMapCanvas
									pickerMode
									compact
									droppedPin={parsedLocation}
									onDropPin={(lat, lng) => {
										locationLat = lat.toFixed(6);
										locationLng = lng.toFixed(6);
										if (autoFilledLocationLabel || isCoordinateOnlyLabel(locationLabel)) {
											locationLabel = `${lat.toFixed(4)}, ${lng.toFixed(4)}`;
										}
										void resolveLocationLabel(lat, lng);
									}}
								/>
							</div>

							<input
								type="text"
								bind:value={locationLabel}
								oninput={() => (autoFilledLocationLabel = false)}
								placeholder={resolvingPlace
									? i18n.t('map.resolvingPlace')
									: i18n.t('editor.locationPlaceholder')}
								class="xm-input !py-1.5 !text-xs"
							/>

							<div class="grid grid-cols-2 gap-2">
								<input
									type="number"
									step="any"
									min="-90"
									max="90"
									bind:value={locationLat}
									placeholder="Latitude (-90..90)"
									class="xm-input !py-1.5 !text-xs tabular-nums"
								/>
								<input
									type="number"
									step="any"
									min="-180"
									max="180"
									bind:value={locationLng}
									placeholder="Longitude (-180..180)"
									class="xm-input !py-1.5 !text-xs tabular-nums"
								/>
							</div>
						</div>
					{/if}
				</div>
			{:else}
				<!-- Live Rendered Memo Preview Panel -->
				<div
					id="memo-editor-preview-panel"
					role="tabpanel"
					class="min-h-[180px] space-y-3 rounded-2xl border border-[var(--xm-divider)] bg-[var(--xm-surface-subtle)] p-4"
				>
					{#if !content.trim() && attachedResources.length === 0}
						<div class="flex min-h-[140px] flex-col items-center justify-center text-center">
							<Eye class="mb-2 h-6 w-6 xm-text-muted" aria-hidden="true" />
							<p class="mt-1 max-w-xs text-xs leading-relaxed xm-text-secondary">
								{i18n.t('editor.emptyPreview')}
							</p>
						</div>
					{:else}
						{#if content.trim()}
							<MemoContent {content} canToggleTask={false} />
						{/if}
						{#if attachedResources.length > 0}
							<AttachmentGrid resources={attachedResources} previewUrls={localPreviewUrls} />
						{/if}
					{/if}
				</div>
			{/if}
		</div>

		<!-- Fixed Footer: Visibility, Optional Timestamp, Save CTA -->
		<div
			class="shrink-0 space-y-2.5 border-t border-[var(--xm-divider)] px-4 pt-3 pb-[max(1.25rem,env(safe-area-inset-bottom))]"
		>
			{#if !isCommentMode}
				<Segmented rounded strong>
					<SegmentedButton active={visibility === 'PRIVATE'} onclick={() => (visibility = 'PRIVATE')}>
						{i18n.t('visibility.private')}
					</SegmentedButton>
					<SegmentedButton
						active={visibility === 'PROTECTED'}
						onclick={() => (visibility = 'PROTECTED')}
					>
						{i18n.t('visibility.protected')}
					</SegmentedButton>
					<SegmentedButton active={visibility === 'PUBLIC'} onclick={() => (visibility = 'PUBLIC')}>
						{i18n.t('visibility.public')}
					</SegmentedButton>
				</Segmented>
			{/if}

			{#if !isCommentMode}
				<div class="flex flex-wrap items-center justify-between gap-2 text-xs xm-text-secondary">
					<div class="flex items-center gap-1.5">
						<Clock class="h-3.5 w-3.5 shrink-0 xm-text-muted" aria-hidden="true" />
						<label for="memo-custom-ts" class="font-medium">{i18n.t('editor.customTimestamp')}</label>
						{#if customTimestampKind === 'past'}
							<span class="rounded-full bg-amber-500/15 px-2 py-0.5 text-[10px] font-semibold text-amber-700 dark:text-amber-300">
								{i18n.t('editor.timestampPast')}
							</span>
						{:else if customTimestampKind === 'future'}
							<span class="rounded-full bg-[var(--xm-accent-soft)] px-2 py-0.5 text-[10px] font-semibold xm-text-accent">
								{i18n.t('editor.timestampFuture')}
							</span>
						{/if}
					</div>
					{#if useCustomTimestamp}
						<div class="flex flex-wrap items-center gap-1.5">
							<input
								id="memo-custom-ts"
								type="datetime-local"
								bind:value={customDateStr}
								class="xm-input !w-auto !py-1.5 !text-xs tabular-nums"
							/>
							<button
								type="button"
								title={editingMemo ? i18n.t('editor.setNow') : i18n.t('editor.resetToNow')}
								class="inline-flex h-8 items-center gap-1 rounded-full border border-[var(--xm-divider)] bg-[var(--xm-surface-subtle)] px-2.5 text-[11px] font-medium xm-text-secondary transition-colors hover:bg-[var(--xm-surface-hover)] hover:text-[var(--xm-text-primary)]"
								onclick={() => {
									if (editingMemo) {
										customDateStr = formatCurrentLocalDateTimeInput();
									} else {
										customDateStr = '';
										useCustomTimestamp = false;
									}
								}}
							>
								<RotateCcw class="h-3 w-3" aria-hidden="true" />
								<span>{editingMemo ? i18n.t('editor.setNow') : i18n.t('editor.resetToNow')}</span>
							</button>
						</div>
					{:else}
						<div class="flex items-center gap-2">
							<span class="rounded-full bg-[var(--xm-surface-subtle)] px-2.5 py-1 text-[11px] font-medium xm-text-muted">
								{i18n.t('editor.timestampAuto')}
							</span>
							<button
								type="button"
								class="inline-flex h-8 items-center gap-1.5 rounded-full border border-[var(--xm-accent-border)] bg-[var(--xm-accent-soft)] px-3 text-[11px] font-semibold xm-text-accent transition-opacity hover:opacity-90"
								onclick={() => {
									customDateStr = formatCurrentLocalDateTimeInput();
									useCustomTimestamp = true;
								}}
							>
								<Calendar class="h-3.5 w-3.5" aria-hidden="true" />
								<span>{i18n.t('editor.setCustomTime')}</span>
							</button>
						</div>
					{/if}
				</div>
			{/if}

			<Button large rounded disabled={saving || uploadingCount > 0} onclick={handleSubmit}>
				{saving
					? i18n.t('common.saving')
					: uploadingCount > 0
						? i18n.t('editor.uploading', { count: uploadingCount })
						: editingMemo
							? i18n.t('editor.updateMemo')
							: parentMemoId
								? i18n.t('editor.postComment')
								: i18n.t('editor.saveMemo')}
			</Button>
		</div>
	</div>
{/if}
