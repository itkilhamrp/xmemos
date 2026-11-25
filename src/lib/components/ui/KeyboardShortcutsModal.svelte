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
	import { Keyboard, X } from 'lucide-svelte';

	import { i18n, type TranslationKey } from '#lib/i18n/index.svelte.ts';
	import { shortcutsState } from '#lib/state/shortcuts.svelte.ts';

	interface ShortcutRow {
		labelKey: TranslationKey;
		keys: string[][];
	}

	interface ShortcutSection {
		titleKey: TranslationKey;
		rows: ShortcutRow[];
	}

	let mod = $derived(shortcutsState.modKey);
	let alt = $derived(shortcutsState.isMac ? '⌥' : 'Alt');

	let sections = $derived<ShortcutSection[]>([
		{
			titleKey: 'shortcuts.sectionGeneral',
			rows: [
				{ labelKey: 'shortcuts.showHelp', keys: [[mod, '/'], ['?']] },
				{ labelKey: 'shortcuts.focusSearch', keys: [[mod, 'K'], ['/']] },
				{ labelKey: 'shortcuts.newMemo', keys: [['N'], ['C']] },
				{ labelKey: 'shortcuts.closeModal', keys: [['Esc']] }
			]
		},
		{
			titleKey: 'shortcuts.sectionWorkspace',
			rows: [
				{ labelKey: 'shortcuts.toggleSidebar', keys: [[mod, '\\']] },
				{ labelKey: 'shortcuts.toggleInspector', keys: [[mod, 'Shift', '\\']] },
				{ labelKey: 'shortcuts.openSettings', keys: [[mod, ','], ['G', 'S']] },
				{ labelKey: 'shortcuts.historyBack', keys: [[alt, '←'], ['⌘', '[']] },
				{ labelKey: 'shortcuts.historyForward', keys: [[alt, '→'], ['⌘', ']']] }
			]
		},
		{
			titleKey: 'shortcuts.sectionNavigation',
			rows: [
				{ labelKey: 'nav.home', keys: [['G', 'H'], [alt, '1']] },
				{ labelKey: 'nav.explore', keys: [['G', 'E'], [alt, '2']] },
				{ labelKey: 'nav.review', keys: [['G', 'R'], [alt, '3']] },
				{ labelKey: 'nav.map', keys: [['G', 'M'], [alt, '4']] },
				{ labelKey: 'nav.media', keys: [['G', 'F'], [alt, '5']] },
				{ labelKey: 'nav.inbox', keys: [['G', 'I']] },
				{ labelKey: 'nav.archived', keys: [['G', 'A']] }
			]
		},
		{
			titleKey: 'shortcuts.sectionEditor',
			rows: [
				{ labelKey: 'shortcuts.saveMemo', keys: [[mod, 'Enter']] },
				{ labelKey: 'shortcuts.togglePreview', keys: [[mod, 'Shift', 'P']] },
				{ labelKey: 'shortcuts.boldItalicCode', keys: [[mod, 'B'], [mod, 'I'], [mod, 'E']] },
				{ labelKey: 'shortcuts.strikethrough', keys: [[mod, 'Shift', 'S']] },
				{ labelKey: 'shortcuts.headings', keys: [[mod, alt, '1–3'], [mod, alt, '0']] },
				{ labelKey: 'shortcuts.codeBlock', keys: [[mod, alt, 'C']] },
				{ labelKey: 'shortcuts.lists', keys: [[mod, 'Shift', '7–9']] },
				{ labelKey: 'shortcuts.indentList', keys: [['Tab'], ['Shift', 'Tab']] }
			]
		}
	]);
</script>

{#if shortcutsState.cheatsheetOpened}
	<div
		class="fixed inset-0 z-[90] flex items-center justify-center bg-black/45 p-3 backdrop-blur-[4px] sm:p-5"
		role="dialog"
		aria-modal="true"
		aria-labelledby="xm-shortcuts-title"
		tabindex="-1"
		onmousedown={(e) => {
			if (e.target === e.currentTarget) shortcutsState.closeCheatsheet();
		}}
	>
		<div
			class="xm-shortcuts-dialog flex max-h-[min(84dvh,640px)] w-full max-w-[680px] flex-col overflow-hidden rounded-2xl"
		>
			<!-- Header -->
			<div
				class="flex shrink-0 items-center justify-between gap-3 border-b border-[var(--xm-divider)] px-4 py-3.5 sm:px-5"
			>
				<div class="flex min-w-0 items-center gap-2.5">
					<span
						class="flex h-8 w-8 shrink-0 items-center justify-center rounded-xl bg-[var(--xm-accent-soft)] xm-text-accent"
						aria-hidden="true"
					>
						<Keyboard class="h-4 w-4" />
					</span>
					<div class="min-w-0">
						<h2
							id="xm-shortcuts-title"
							class="truncate text-[15px] font-bold tracking-[-0.01em] xm-text-primary"
						>
							{i18n.t('shortcuts.title')}
						</h2>
						<p class="truncate text-xs xm-text-muted">{i18n.t('shortcuts.subtitle')}</p>
					</div>
				</div>
				<button
					type="button"
					aria-label={i18n.t('common.close')}
					class="flex h-8 w-8 shrink-0 items-center justify-center rounded-full xm-text-secondary transition-colors hover:bg-[var(--xm-surface-subtle)] hover:text-[var(--xm-text-primary)] cursor-pointer"
					onclick={() => shortcutsState.closeCheatsheet()}
				>
					<X class="h-4 w-4" aria-hidden="true" />
				</button>
			</div>

			<!-- Adaptive 2-column Desktop / 1-column Mobile Body -->
			<div class="xm-sidebar-scroll min-h-0 flex-1 overflow-y-auto overscroll-contain p-4 sm:p-5">
				<div class="grid grid-cols-1 gap-5 md:grid-cols-2">
					{#each sections as section (section.titleKey)}
						<section class="space-y-2">
							<h3
								class="px-1 text-[11px] font-semibold tracking-[0.05em] uppercase xm-text-muted"
							>
								{i18n.t(section.titleKey)}
							</h3>
							<div
								class="divide-y divide-[var(--xm-divider)] rounded-xl border border-[var(--xm-divider)] bg-[var(--xm-surface-subtle)]"
							>
								{#each section.rows as row (row.labelKey)}
									<div class="flex items-center justify-between gap-2 px-3 py-2">
										<span class="min-w-0 truncate text-[12.5px] font-medium xm-text-primary">
											{i18n.t(row.labelKey)}
										</span>
										<div class="flex shrink-0 flex-wrap items-center justify-end gap-1.5">
											{#each row.keys as combo, ci (ci)}
												{#if ci > 0}
													<span class="text-[10px] xm-text-muted">/</span>
												{/if}
												<span class="inline-flex items-center gap-0.5">
													{#each combo as keycap, ki (ki)}
														<kbd class="xm-kbd">{keycap}</kbd>
													{/each}
												</span>
											{/each}
										</div>
									</div>
								{/each}
							</div>
						</section>
					{/each}
				</div>
			</div>

			<!-- Footer Hint -->
			<div
				class="flex shrink-0 items-center justify-between gap-2 border-t border-[var(--xm-divider)] px-4 py-2.5 text-[11.5px] xm-text-muted sm:px-5"
			>
				<span>{i18n.t('shortcuts.footerHint')}</span>
				<span class="inline-flex items-center gap-1">
					<kbd class="xm-kbd">{mod}</kbd>
					<kbd class="xm-kbd">/</kbd>
				</span>
			</div>
		</div>
	</div>
{/if}
