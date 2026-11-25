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
	import { BlockTitle, List, ListItem, Segmented, SegmentedButton, Toggle } from 'konsta/svelte';
	import { Languages, Monitor, Moon, Sun } from 'lucide-svelte';

	import { i18n, SUPPORTED_LOCALES, type Locale } from '#lib/i18n/index.svelte.ts';
	import { authState } from '#lib/state/auth.svelte.ts';
	import { confirmDeleteState } from '#lib/state/confirm-delete.svelte.ts';
	import { themeState } from '#lib/state/theme.svelte.ts';
	import { toastState } from '#lib/state/toast.svelte.ts';
	import AppShell from '#lib/components/layout/AppShell.svelte';

	async function selectLocale(code: Locale): Promise<void> {
		if (i18n.locale === code) return;
		await i18n.setLocale(code);
		const meta = SUPPORTED_LOCALES.find((l) => l.code === code);
		toastState.show(i18n.t('pref.languageChanged', { name: meta?.nativeName ?? code }), 'success');
	}

	async function toggleConfirmDelete(): Promise<void> {
		if (!authState.isAuthenticated) return;
		const next = !confirmDeleteState.enabled;
		await confirmDeleteState.setEnabled(next);
		toastState.show(i18n.t(next ? 'pref.confirmEnabled' : 'pref.confirmDisabled'), 'info');
	}
</script>

<AppShell
	title={i18n.t('pref.title')}
	backHref="/settings"
	backLabel={i18n.t('nav.settings')}
	showFab={false}
>
	<div class="space-y-4">
		<!-- Instant Multi-Language Switcher (EN / ID / ES) -->
		<section class="glass-surface space-y-3 rounded-2xl p-4" aria-labelledby="pref-language-heading">
			<div class="flex items-center justify-between gap-2">
				<h2 id="pref-language-heading" class="flex items-center gap-1.5 text-sm font-bold xm-text-primary">
					<Languages class="h-4 w-4 xm-text-accent" aria-hidden="true" />
					<span>{i18n.t('pref.language')}</span>
				</h2>
				<span class="text-xs font-semibold xm-text-accent">{i18n.meta.nativeName}</span>
			</div>
			<p class="text-xs leading-relaxed xm-text-secondary">
				{i18n.t('pref.languageDesc')}
			</p>
			<Segmented rounded strong>
				{#each SUPPORTED_LOCALES as lang (lang.code)}
					<SegmentedButton
						active={i18n.locale === lang.code}
						onclick={() => selectLocale(lang.code)}
					>
						{lang.nativeName}
					</SegmentedButton>
				{/each}
			</Segmented>
		</section>

		<section class="glass-surface space-y-3 rounded-2xl p-4" aria-labelledby="pref-appearance-heading">
			<div class="flex items-center justify-between gap-2">
				<h2 id="pref-appearance-heading" class="text-sm font-bold xm-text-primary">
					{i18n.t('pref.appearance')}
				</h2>
				<span
					class="inline-flex items-center gap-1 rounded-full bg-[var(--xm-accent-soft)] px-2.5 py-0.5 text-[11px] font-semibold xm-text-accent"
				>
					{#if themeState.isDark}
						<Moon class="h-3 w-3" aria-hidden="true" />
						<span>{i18n.t('pref.darkActive')}</span>
					{:else}
						<Sun class="h-3 w-3" aria-hidden="true" />
						<span>{i18n.t('pref.lightActive')}</span>
					{/if}
				</span>
			</div>
			<p class="text-xs leading-relaxed xm-text-secondary">
				{i18n.t('pref.appearanceDesc')}
			</p>
			<Segmented rounded strong>
				<SegmentedButton
					active={themeState.mode === 'system'}
					onclick={() => themeState.setMode('system')}
				>
					<span class="inline-flex items-center gap-1.5">
						<Monitor class="h-3.5 w-3.5" aria-hidden="true" /> {i18n.t('pref.modeSystem')}
					</span>
				</SegmentedButton>
				<SegmentedButton
					active={themeState.mode === 'light'}
					onclick={() => themeState.setMode('light')}
				>
					<span class="inline-flex items-center gap-1.5">
						<Sun class="h-3.5 w-3.5" aria-hidden="true" /> {i18n.t('pref.modeLight')}
					</span>
				</SegmentedButton>
				<SegmentedButton
					active={themeState.mode === 'dark'}
					onclick={() => themeState.setMode('dark')}
				>
					<span class="inline-flex items-center gap-1.5">
						<Moon class="h-3.5 w-3.5" aria-hidden="true" /> {i18n.t('pref.modeDark')}
					</span>
				</SegmentedButton>
			</Segmented>
		</section>

		<section class="glass-surface space-y-3 rounded-2xl p-4" aria-labelledby="pref-glass-heading">
			<h2 id="pref-glass-heading" class="text-sm font-bold xm-text-primary">
				{i18n.t('pref.glassTitle')}
			</h2>
			<p class="text-xs leading-relaxed xm-text-secondary">
				{i18n.t('pref.glassDesc')}
			</p>
			<Segmented rounded strong>
				<SegmentedButton
					active={themeState.glass === 'liquid'}
					onclick={() => themeState.setGlass('liquid')}
				>
					{i18n.t('pref.glassLiquid')}
				</SegmentedButton>
				<SegmentedButton
					active={themeState.glass === 'subtle'}
					onclick={() => themeState.setGlass('subtle')}
				>
					{i18n.t('pref.glassSubtle')}
				</SegmentedButton>
				<SegmentedButton
					active={themeState.glass === 'solid'}
					onclick={() => themeState.setGlass('solid')}
				>
					{i18n.t('pref.glassSolid')}
				</SegmentedButton>
			</Segmented>
		</section>

		<section class="glass-surface space-y-3 rounded-2xl p-4" aria-labelledby="pref-density-heading">
			<h2 id="pref-density-heading" class="text-sm font-bold xm-text-primary">
				{i18n.t('pref.densityTitle')}
			</h2>
			<p class="text-xs leading-relaxed xm-text-secondary">
				{i18n.t('pref.densityDesc')}
			</p>
			<Segmented rounded strong>
				<SegmentedButton
					active={themeState.density === 'comfortable'}
					onclick={() => themeState.setDensity('comfortable')}
				>
					{i18n.t('pref.densityComfortable')}
				</SegmentedButton>
				<SegmentedButton
					active={themeState.density === 'compact'}
					onclick={() => themeState.setDensity('compact')}
				>
					{i18n.t('pref.densityCompact')}
				</SegmentedButton>
			</Segmented>
		</section>

		<div class={!authState.isAuthenticated ? 'opacity-60' : ''}>
			<BlockTitle class="!mt-2 !mb-2 !px-1">{i18n.t('pref.safetySection')}</BlockTitle>
			<List strong inset class="!my-0 !mx-0">
				<ListItem
					title={i18n.t('pref.confirmDeleteTitle')}
					subtitle={authState.isAuthenticated
						? i18n.t('pref.confirmDeleteSub')
						: i18n.t('guest.safetyLockedSub')}
				>
					{#snippet after()}
						<Toggle
							checked={confirmDeleteState.enabled}
							disabled={!authState.isAuthenticated}
							onchange={toggleConfirmDelete}
						/>
					{/snippet}
				</ListItem>
			</List>
		</div>
	</div>
</AppShell>
