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
		BlockTitle,
		Button,
		Dialog,
		DialogButton,
		List,
		ListItem,
		Segmented,
		SegmentedButton
	} from 'konsta/svelte';
	import {
		Database,
		Download,
		Package,
		ShieldCheck,
		Upload,
		User as UserIcon,
		Users
	} from 'lucide-svelte';

	import { backupApi } from '#lib/api/backup.ts';
	import { resourceApi } from '#lib/api/resource.ts';
	import type { BackupHistoryEntry, BackupScope, ImportPreview } from '#lib/types/backup.ts';
	import { i18n } from '#lib/i18n/index.svelte.ts';
	import { authState } from '#lib/state/auth.svelte.ts';
	import { toastState } from '#lib/state/toast.svelte.ts';
	import { formatBytes, formatFullTimestamp } from '#lib/utils/date.ts';
	import { formatSavedLabel } from '#lib/utils/download.ts';

	interface Props {
		history: BackupHistoryEntry[];
		onChanged?: () => void;
	}

	let { history, onChanged }: Props = $props();

	let busy = $state(false);
	let importPreview = $state<ImportPreview | null>(null);
	let manualImportPath = $state('');
	let selectedScope = $state<BackupScope>('ACTIVE_USER');

	let canBackupAllUsers = $derived(authState.isAdminOrHost);
	let activeUsername = $derived(authState.currentUser?.username ?? 'user');
	let effectiveScope = $derived<BackupScope>(canBackupAllUsers ? selectedScope : 'ACTIVE_USER');

	// Default HOST/ADMIN to ALL_USERS once auth initializes
	let scopeInitialized = false;
	$effect(() => {
		if (!scopeInitialized && authState.initialized) {
			scopeInitialized = true;
			selectedScope = authState.isAdminOrHost ? 'ALL_USERS' : 'ACTIVE_USER';
		}
	});

	function isLocalCopyablePath(path: string): boolean {
		if (!path || path.startsWith('content://')) return false;
		return path.includes('/backups/') || path.includes('\\backups\\');
	}

	async function handleExportDb(): Promise<void> {
		busy = true;
		let stagedPath: string | null = null;
		let finalized = false;
		try {
			const res = await backupApi.exportDb(effectiveScope);
			stagedPath = res.path;
			const saved = await backupApi.saveToSaf(res.path, res.filename, 'application/x-sqlite3');
			const scopeLabel = res.scope === 'ALL_USERS' ? 'All Users' : `@${activeUsername}`;
			if (saved.cancelled) {
				await backupApi.finalizeExport(res.path, null, null);
				finalized = true;
				toastState.show(i18n.t('backup.exportCancelled'), 'info');
			} else {
				const savedDest = saved.savedUri ?? res.filename;
				await backupApi.finalizeExport(res.path, savedDest, res);
				finalized = true;
				const targetName = saved.savedUri
					? formatSavedLabel(saved.savedUri, res.filename)
					: res.filename;
				toastState.show(
					`Saved ${scopeLabel} DB: ${targetName} (${formatBytes(res.sizeBytes)})`,
					'success'
				);
				onChanged?.();
			}
		} catch (err) {
			toastState.show(err instanceof Error ? err.message : String(err), 'error');
		} finally {
			if (stagedPath && !finalized) {
				await backupApi.finalizeExport(stagedPath, null, null).catch(() => {});
			}
			busy = false;
		}
	}

	async function handleExportFull(): Promise<void> {
		busy = true;
		let stagedPath: string | null = null;
		let finalized = false;
		try {
			const res = await backupApi.exportFull(effectiveScope);
			stagedPath = res.path;
			const saved = await backupApi.saveToSaf(res.path, res.filename, 'application/zip');
			const scopeLabel = res.scope === 'ALL_USERS' ? 'All Users' : `@${activeUsername}`;
			if (saved.cancelled) {
				await backupApi.finalizeExport(res.path, null, null);
				finalized = true;
				toastState.show(i18n.t('backup.exportCancelled'), 'info');
			} else {
				const savedDest = saved.savedUri ?? res.filename;
				await backupApi.finalizeExport(res.path, savedDest, res);
				finalized = true;
				const targetName = saved.savedUri
					? formatSavedLabel(saved.savedUri, res.filename)
					: res.filename;
				toastState.show(
					`Saved ${scopeLabel} Bundle: ${targetName} (${res.mediaCount} assets, ${formatBytes(res.sizeBytes)})`,
					'success'
				);
				onChanged?.();
			}
		} catch (err) {
			toastState.show(err instanceof Error ? err.message : String(err), 'error');
		} finally {
			if (stagedPath && !finalized) {
				await backupApi.finalizeExport(stagedPath, null, null).catch(() => {});
			}
			busy = false;
		}
	}

	async function handleDownloadHistoryEntry(entry: BackupHistoryEntry): Promise<void> {
		busy = true;
		try {
			const mime = entry.mode === 'FULL_BUNDLE' ? 'application/zip' : 'application/x-sqlite3';
			const saved = await backupApi.saveToSaf(entry.path, entry.filename, mime);
			if (!saved.cancelled) {
				const targetName = saved.savedUri
					? formatSavedLabel(saved.savedUri, entry.filename)
					: entry.filename;
				toastState.show(`Saved backup copy: ${targetName}`, 'success');
			}
		} catch (err) {
			toastState.show(err instanceof Error ? err.message : String(err), 'error');
		} finally {
			busy = false;
		}
	}

	async function handlePickImport(): Promise<void> {
		busy = true;
		try {
			const picked = await resourceApi.pickDocument(
				['application/zip', 'application/octet-stream', 'application/x-sqlite3', '*/*'],
				true
			);
			if (!picked.cancelled && picked.localPath) {
				importPreview = await backupApi.previewImport(picked.localPath);
			}
		} catch (err) {
			toastState.show(err instanceof Error ? err.message : String(err), 'error');
		} finally {
			busy = false;
		}
	}

	async function handlePreviewManualPath(): Promise<void> {
		if (!manualImportPath.trim()) return;
		busy = true;
		try {
			importPreview = await backupApi.previewImport(manualImportPath.trim());
		} catch (err) {
			toastState.show(err instanceof Error ? err.message : String(err), 'error');
		} finally {
			busy = false;
		}
	}

	async function confirmAtomicImport(): Promise<void> {
		if (!importPreview) return;
		busy = true;
		try {
			const result = await backupApi.executeImport(importPreview.sourcePath);
			importPreview = null;
			manualImportPath = '';
			await authState.refresh();
			toastState.show(
				i18n.t('backup.restored', { memos: result.memoCount, files: result.restoredMediaFiles }),
				'success'
			);
			onChanged?.();
		} catch (err) {
			toastState.show(err instanceof Error ? err.message : String(err), 'error');
		} finally {
			busy = false;
		}
	}
</script>

<div class="space-y-4">
	<!-- Backup Scope Selector (Official Memos RBAC: All Users restricted to HOST/ADMIN) -->
	<section class="glass-surface space-y-3 rounded-2xl p-4" aria-labelledby="backup-scope-heading">
		<div class="flex items-center justify-between gap-2">
			<h2 id="backup-scope-heading" class="text-sm font-bold xm-text-primary">{i18n.t('backup.scopeTitle')}</h2>
			<span class="text-[11px] font-semibold xm-text-muted">
				{authState.currentUser?.role ?? 'USER'}
			</span>
		</div>

		{#if canBackupAllUsers}
			<Segmented rounded strong>
				<SegmentedButton
					active={effectiveScope === 'ALL_USERS'}
					onclick={() => (selectedScope = 'ALL_USERS')}
				>
					<span class="inline-flex items-center gap-1.5">
						<Users class="h-3.5 w-3.5" aria-hidden="true" />
						<span>{i18n.t('backup.scopeAll')}</span>
					</span>
				</SegmentedButton>
				<SegmentedButton
					active={effectiveScope === 'ACTIVE_USER'}
					onclick={() => (selectedScope = 'ACTIVE_USER')}
				>
					<span class="inline-flex items-center gap-1.5">
						<UserIcon class="h-3.5 w-3.5" aria-hidden="true" />
						<span>{i18n.t('backup.scopeActive', { username: activeUsername })}</span>
					</span>
				</SegmentedButton>
			</Segmented>
		{/if}

		<p class="text-xs leading-relaxed xm-text-secondary">
			{#if effectiveScope === 'ALL_USERS'}
				{i18n.t('backup.scopeAllDesc')}
			{:else}
				{i18n.t('backup.scopeActiveDesc', { username: activeUsername })}
				{#if !canBackupAllUsers}
					{' '}{i18n.t('backup.scopeAdminNote')}
				{/if}
			{/if}
		</p>
	</section>

	<div class="grid gap-3 sm:grid-cols-2">
		<div class="glass-surface flex flex-col justify-between space-y-3 rounded-2xl p-4">
			<div class="space-y-2">
				<div class="flex items-center gap-2 font-semibold xm-text-primary">
					<Database class="h-5 w-5 shrink-0 xm-text-accent" aria-hidden="true" />
					<span>{i18n.t('backup.mode1Title')}</span>
				</div>
				<p class="text-xs leading-relaxed xm-text-secondary">
					{#if effectiveScope === 'ALL_USERS'}
						{i18n.t('backup.mode1AllDesc')}
					{:else}
						{i18n.t('backup.mode1ActiveDesc', { username: activeUsername })}
					{/if}
				</p>
			</div>
			<Button rounded disabled={busy} onclick={handleExportDb}>
				<Download class="mr-1.5 h-4 w-4" aria-hidden="true" />
				{effectiveScope === 'ALL_USERS' ? i18n.t('backup.exportAllDb') : i18n.t('backup.exportUserDb', { username: activeUsername })}
			</Button>
		</div>

		<div class="glass-surface flex flex-col justify-between space-y-3 rounded-2xl p-4">
			<div class="space-y-2">
				<div class="flex items-center gap-2 font-semibold xm-text-primary">
					<Package class="h-5 w-5 shrink-0 xm-text-accent" aria-hidden="true" />
					<span>{i18n.t('backup.mode2Title')}</span>
				</div>
				<p class="text-xs leading-relaxed xm-text-secondary">
					{#if effectiveScope === 'ALL_USERS'}
						{i18n.t('backup.mode2AllDesc')}
					{:else}
						{i18n.t('backup.mode2ActiveDesc', { username: activeUsername })}
					{/if}
				</p>
			</div>
			<Button rounded disabled={busy} onclick={handleExportFull}>
				<Download class="mr-1.5 h-4 w-4" aria-hidden="true" />
{effectiveScope === 'ALL_USERS' ? i18n.t('backup.exportAllBundle') : i18n.t('backup.exportUserBundle', { username: activeUsername })}
			</Button>
		</div>
	</div>

	{#if canBackupAllUsers}
		<div class="glass-surface space-y-3 rounded-2xl p-4">
			<div class="flex items-center gap-2 font-semibold xm-text-primary">
				<Upload class="h-5 w-5 shrink-0 xm-text-success" aria-hidden="true" />
				<span>{i18n.t('backup.importTitle')}</span>
			</div>
			<p class="text-xs leading-relaxed xm-text-secondary">
				{i18n.t('backup.importDesc')}
			</p>
			<div class="flex flex-col gap-2.5 sm:flex-row sm:items-center">
				<Button rounded outline disabled={busy} onclick={handlePickImport} class="sm:w-auto sm:shrink-0">
					{i18n.t('backup.chooseFile')}
				</Button>
				<div class="flex flex-1 items-center gap-2">
					<label for="manual-import-path" class="sr-only">Local backup file path</label>
					<input
						id="manual-import-path"
						type="text"
						bind:value={manualImportPath}
						placeholder={i18n.t('backup.manualPlaceholder')}
						class="xm-input !py-2 !text-xs"
					/>
					<Button
						rounded
						outline
						disabled={busy || !manualImportPath.trim()}
						onclick={handlePreviewManualPath}
						class="!w-auto shrink-0"
					>
						{i18n.t('backup.inspect')}
					</Button>
				</div>
			</div>
		</div>
	{/if}

	{#if history.length > 0}
		<div>
			<BlockTitle class="!mt-3 !mb-2 !px-1">{i18n.t('backup.recent')}</BlockTitle>
			<List strong inset class="!my-0 !mx-0">
				{#each history as entry (entry.id)}
					<ListItem
						title={entry.filename}
						subtitle={`${entry.mode === 'FULL_BUNDLE' ? 'Full Bundle' : 'DB Only'} • ${formatBytes(entry.sizeBytes)} • ${formatFullTimestamp(entry.createdTs)}`}
						footer={`SHA-256: ${entry.sha256.slice(0, 16)}…`}
					>
						{#snippet after()}
							{#if isLocalCopyablePath(entry.path)}
								<button
									type="button"
									aria-label="Save {entry.filename}"
									title="Save {entry.filename}"
									disabled={busy}
									class="flex h-8 w-8 items-center justify-center rounded-full xm-text-accent transition-colors hover:bg-[var(--xm-accent-soft)] disabled:opacity-50"
									onclick={() => handleDownloadHistoryEntry(entry)}
								>
									<Download class="h-4 w-4" aria-hidden="true" />
								</button>
							{/if}
						{/snippet}
					</ListItem>
				{/each}
			</List>
		</div>
	{/if}
</div>

<Dialog opened={importPreview !== null} onBackdropClick={() => (importPreview = null)}>
	{#snippet title()}
		<span class="flex items-center gap-1.5">
			<ShieldCheck class="h-5 w-5 xm-text-success" aria-hidden="true" /> {i18n.t('backup.confirmRestoreTitle')}
		</span>
	{/snippet}
	{#if importPreview}
		<div class="space-y-1.5 text-left text-xs xm-text-primary">
			<p>Format: <strong>{importPreview.detectedFormat}</strong></p>
			<p>Users: <strong class="tabular-nums">{importPreview.userCount}</strong></p>
			<p>Memos: <strong class="tabular-nums">{importPreview.memoCount}</strong></p>
			<p>
				Attachments: <strong class="tabular-nums">{importPreview.resourceCount}</strong>
				({importPreview.mediaFileCount} files)
			</p>
			<p>Archive Size: <strong class="tabular-nums">{formatBytes(importPreview.archiveSizeBytes)}</strong></p>
			<p>
				SHA-256 Checksum:
				<strong>{importPreview.checksumVerified ? 'Verified' : 'Raw DB / Unmanifested'}</strong>
			</p>
			{#each importPreview.compatibilityNotes as note (note)}
				<p class="xm-text-warning">• {note}</p>
			{/each}
		</div>
	{/if}
	{#snippet buttons()}
		<DialogButton
			class="!bg-black/8 dark:!bg-white/14 !text-[var(--xm-text-primary)] !font-medium"
			onclick={() => (importPreview = null)}
		>
			{i18n.t('common.cancel')}
		</DialogButton>
		<DialogButton
			strong
			class="!bg-[#007aff] dark:!bg-[#0a84ff] !text-white !font-semibold"
			onclick={confirmAtomicImport}
		>
			{i18n.t('common.restore')}
		</DialogButton>
	{/snippet}
</Dialog>
