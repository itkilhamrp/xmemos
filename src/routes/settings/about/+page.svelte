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
	import { List, ListItem } from 'konsta/svelte';
	import { i18n } from '#lib/i18n/index.svelte.ts';
	import { systemApi } from '#lib/api/system.ts';
	import type { SystemInfo } from '#lib/types/setting.ts';
	import AppShell from '#lib/components/layout/AppShell.svelte';
	import AppIcon from '#lib/components/ui/AppIcon.svelte';
	import { isDesktopTauri } from '#lib/utils/platform.ts';

	let info = $state<SystemInfo | null>(null);

	onMount(async () => {
		try {
			info = await systemApi.getInfo();
		} catch {
			// Browser preview fallback
		}
	});
</script>

<AppShell title="About Xmemos" backHref="/settings" backLabel="Settings" showFab={false}>
	<div class="space-y-4">
		<div class="glass-surface flex items-center gap-4 rounded-3xl p-5">
			<AppIcon class="h-14 w-14 shrink-0" decorative />
			<div class="min-w-0 space-y-1">
				<h1 class="text-xl font-bold xm-text-primary">Xmemos</h1>
				<p class="text-xs leading-relaxed xm-text-secondary">
					Privacy-first, 100% offline {isDesktopTauri() ? 'Desktop' : 'Android'} microblogging and
					knowledge-capture app with 1:1 Official FOSS Memos SQLite compatibility.
				</p>
			</div>
		</div>

		<List strong inset class="!my-0 !mx-0">
			<ListItem title="Application Version" after={info ? `v${info.appVersion}` : 'v2.9.6'} />
			<ListItem title={i18n.t('about.license')} after={info?.license ?? 'AGPL-3.0-or-later'} />
			<ListItem title="Memos Schema Parity" after={info?.memosSchemaVersion ?? '0.24.0'} />
			<ListItem title="SQLite Engine" after={info?.sqliteVersion ?? 'Bundled SQLite 3'} />
			<ListItem title="Journal Mode" after={info?.journalMode.toUpperCase() ?? 'WAL'} />
			{#if info?.platform}
				<ListItem
					title="Platform"
					after={`${info.platform} (${info.targetArch ?? 'unknown'})${info.portableMode ? ' · Portable' : ''}`}
				/>
			{/if}
		</List>
	</div>
</AppShell>
