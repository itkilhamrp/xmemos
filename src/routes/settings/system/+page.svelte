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
	import { onMount } from 'svelte';
	import { BlockTitle, Button, List, ListItem, Toggle } from 'konsta/svelte';
	import { HardDriveUpload, Lock } from 'lucide-svelte';

	import { systemApi } from '#lib/api/system.ts';
	import { i18n } from '#lib/i18n/index.svelte.ts';
	import { authState } from '#lib/state/auth.svelte.ts';
	import { toastState } from '#lib/state/toast.svelte.ts';
	import AppShell from '#lib/components/layout/AppShell.svelte';

	const PRESET_LIMITS_MB = [10, 32, 64, 128, 0] as const;

	let disallowReg = $state(false);
	let maxUploadSizeMb = $state(32);
	let uploadLimitInput = $state('32');
	let savingLimit = $state(false);

	let canManageSystem = $derived(authState.isAdminOrHost);

	onMount(async () => {
		const settings = await systemApi.listSettings();
		disallowReg = settings.some((s) => s.name === 'disallow-user-registration' && s.value === 'true');
		const uploadSetting = settings.find((s) => s.name === 'max-upload-size-mb');
		if (uploadSetting) {
			const parsed = Number.parseInt(uploadSetting.value, 10);
			if (Number.isFinite(parsed) && parsed >= 0) {
				maxUploadSizeMb = parsed;
				uploadLimitInput = String(parsed);
			}
		}
	});

	async function toggleDisallowReg(): Promise<void> {
		if (!canManageSystem) {
			toastState.show(i18n.t('system.adminOnlyNote'), 'error');
			return;
		}
		const next = !disallowReg;
		try {
			await systemApi.upsertSetting(
				'disallow-user-registration',
				next ? 'true' : 'false',
				'Prevent self-service local account sign-up'
			);
			disallowReg = next;
			await authState.refresh();
			toastState.show(i18n.t('system.updated'), 'success');
		} catch (err) {
			toastState.show(err instanceof Error ? err.message : String(err), 'error');
		}
	}

	async function saveUploadLimit(rawValue?: number): Promise<void> {
		if (!canManageSystem) {
			toastState.show(i18n.t('system.adminOnlyNote'), 'error');
			return;
		}
		const parsed =
			rawValue !== undefined ? rawValue : Number.parseInt(uploadLimitInput.trim(), 10);
		if (!Number.isFinite(parsed) || parsed < 0 || parsed > 2048 || !Number.isInteger(parsed)) {
			toastState.show(i18n.t('system.uploadLimitInvalid'), 'error');
			return;
		}
		savingLimit = true;
		try {
			await systemApi.upsertSetting(
				'max-upload-size-mb',
				String(parsed),
				'Maximum static file upload size in MB (0 = unlimited)'
			);
			maxUploadSizeMb = parsed;
			uploadLimitInput = String(parsed);
			toastState.show(
				parsed === 0
					? i18n.t('system.uploadLimitSavedUnlimited')
					: i18n.t('system.uploadLimitSaved', { mb: String(parsed) }),
				'success'
			);
		} catch (err) {
			toastState.show(err instanceof Error ? err.message : String(err), 'error');
		} finally {
			savingLimit = false;
		}
	}
</script>

<AppShell title={i18n.t('system.title')} backHref="/settings" backLabel={i18n.t('nav.settings')} showFab={false}>
	<div class="space-y-4">
		{#if !canManageSystem}
			<div class="glass-surface flex items-center gap-2.5 rounded-2xl p-3.5 text-xs xm-text-secondary">
				<Lock class="h-4 w-4 shrink-0 xm-text-warning" aria-hidden="true" />
				<span>{i18n.t('system.adminOnlyNote')}</span>
			</div>
		{/if}

		<div>
			<BlockTitle class="!mt-1 !mb-2 !px-1">{i18n.t('system.regPolicy')}</BlockTitle>
			<List strong inset class="!my-0 !mx-0">
				<ListItem
					title={i18n.t('system.disallowRegTitle')}
					subtitle={i18n.t('system.disallowRegSub')}
				>
					{#snippet after()}
						<Toggle checked={disallowReg} disabled={!canManageSystem} onchange={toggleDisallowReg} />
					{/snippet}
				</ListItem>
			</List>
		</div>

		<div>
			<BlockTitle class="!mt-1 !mb-2 !px-1">{i18n.t('system.uploadPolicy')}</BlockTitle>
			<section class="glass-surface space-y-3 rounded-2xl p-4" aria-labelledby="upload-limit-heading">
				<div class="flex items-start justify-between gap-3">
					<div class="space-y-1">
						<h2 id="upload-limit-heading" class="flex items-center gap-2 text-sm font-bold xm-text-primary">
							<HardDriveUpload class="h-4 w-4 shrink-0 xm-text-accent" aria-hidden="true" />
							<span>{i18n.t('system.uploadLimitTitle')}</span>
						</h2>
						<p class="text-xs leading-relaxed xm-text-secondary">
							{i18n.t('system.uploadLimitDesc')}
						</p>
					</div>
					<span class="shrink-0 rounded-full bg-[var(--xm-accent-soft)] px-2.5 py-1 text-xs font-bold xm-text-accent tabular-nums">
						{maxUploadSizeMb === 0 ? i18n.t('system.uploadUnlimited') : `${maxUploadSizeMb} MB`}
					</span>
				</div>

				{#if canManageSystem}
					<div class="flex flex-wrap gap-1.5" role="group" aria-label={i18n.t('system.uploadLimitTitle')}>
						{#each PRESET_LIMITS_MB as preset (preset)}
							<button
								type="button"
								disabled={savingLimit}
								class="rounded-full border px-3 py-1 text-xs font-semibold transition-colors tabular-nums {maxUploadSizeMb === preset
									? 'border-[var(--xm-accent)] bg-[var(--xm-accent)] text-white'
									: 'border-[var(--xm-divider)] bg-[var(--xm-surface-subtle)] xm-text-secondary hover:text-[var(--xm-text-primary)]'}"
								onclick={() => saveUploadLimit(preset)}
							>
								{preset === 0 ? i18n.t('system.uploadUnlimited') : `${preset} MB`}
							</button>
						{/each}
					</div>

					<div class="flex items-center gap-2">
						<label for="upload-limit-mb-input" class="sr-only">{i18n.t('system.uploadLimitTitle')}</label>
						<input
							id="upload-limit-mb-input"
							type="number"
							min="0"
							max="2048"
							step="1"
							bind:value={uploadLimitInput}
							placeholder="32"
							class="xm-input !py-2 !text-xs tabular-nums"
						/>
						<span class="shrink-0 text-xs font-semibold xm-text-muted">MB</span>
						<Button
							rounded
							disabled={savingLimit}
							onclick={() => saveUploadLimit()}
							class="!w-auto shrink-0"
						>
							{i18n.t('common.save')}
						</Button>
					</div>
					<p class="text-[11px] xm-text-muted">{i18n.t('system.uploadLimitHint')}</p>
				{/if}
			</section>
		</div>
	</div>
</AppShell>
