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
	import { BlockTitle, List, ListItem, Toggle } from 'konsta/svelte';
	import { systemApi } from '#lib/api/system.ts';
	import { i18n } from '#lib/i18n/index.svelte.ts';
	import { authState } from '#lib/state/auth.svelte.ts';
	import { toastState } from '#lib/state/toast.svelte.ts';
	import AppShell from '#lib/components/layout/AppShell.svelte';

	let disallowReg = $state(false);

	onMount(async () => {
		const settings = await systemApi.listSettings();
		disallowReg = settings.some((s) => s.name === 'disallow-user-registration' && s.value === 'true');
	});

	async function toggleDisallowReg(): Promise<void> {
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
</script>

<AppShell title={i18n.t('system.title')} backHref="/settings" backLabel={i18n.t('nav.settings')} showFab={false}>
	<div class="space-y-2">
		<BlockTitle class="!mt-1 !mb-2 !px-1">{i18n.t('system.regPolicy')}</BlockTitle>
		<List strong inset class="!my-0 !mx-0">
			<ListItem
				title={i18n.t('system.disallowRegTitle')}
				subtitle={i18n.t('system.disallowRegSub')}
			>
				{#snippet after()}
					<Toggle checked={disallowReg} onchange={toggleDisallowReg} />
				{/snippet}
			</ListItem>
		</List>
	</div>
</AppShell>
