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
	import { goto } from '$app/navigation';
	import { BlockTitle, List, ListItem } from 'konsta/svelte';
	import {
		HardDrive,
		Info,
		Palette,
		Server,
		User as UserIcon,
		Users
	} from 'lucide-svelte';

	import { i18n } from '#lib/i18n/index.svelte.ts';
	import { authState } from '#lib/state/auth.svelte.ts';
	import AppShell from '#lib/components/layout/AppShell.svelte';
</script>

<AppShell title={i18n.t('settings.title')} showFab={false}>
	<div class="space-y-4">
		<BlockTitle class="!mt-1 !mb-2 !px-1">{i18n.t('settings.sectionPersonal')}</BlockTitle>
		<List strong inset class="!my-0 !mx-0">
			<ListItem
				link
				title={i18n.t('settings.accountTitle')}
				subtitle={authState.currentUser
					? `@${authState.currentUser.username} (${authState.currentUser.role})`
					: i18n.t('settings.notSignedIn')}
				onclick={() => goto('/settings/account')}
			>
				{#snippet media()}
					<UserIcon class="h-5 w-5 xm-text-accent" aria-hidden="true" />
				{/snippet}
			</ListItem>
			<ListItem
				link
				title={i18n.t('settings.preferencesTitle')}
				subtitle={i18n.t('settings.preferencesSub')}
				onclick={() => goto('/settings/preferences')}
			>
				{#snippet media()}
					<Palette class="h-5 w-5 text-purple-600 dark:text-purple-400" aria-hidden="true" />
				{/snippet}
			</ListItem>
		</List>

		<BlockTitle class="!mt-5 !mb-2 !px-1">{i18n.t('settings.sectionWorkspace')}</BlockTitle>
		<List strong inset class="!my-0 !mx-0">
			{#if authState.isAdminOrHost}
				<ListItem
					link
					title={i18n.t('settings.usersTitle')}
					subtitle={i18n.t('settings.usersSub')}
					onclick={() => goto('/settings/users')}
				>
					{#snippet media()}
						<Users class="h-5 w-5 xm-text-success" aria-hidden="true" />
					{/snippet}
				</ListItem>
			{/if}
			{#if authState.isHost}
				<ListItem
					link
					title={i18n.t('settings.systemTitle')}
					subtitle={i18n.t('settings.systemSub')}
					onclick={() => goto('/settings/system')}
				>
					{#snippet media()}
						<Server class="h-5 w-5 xm-text-warning" aria-hidden="true" />
					{/snippet}
				</ListItem>
			{/if}
			{#if authState.isAuthenticated}
				<ListItem
					link
					title={i18n.t('settings.storageTitle')}
					subtitle={i18n.t('settings.storageSub')}
					onclick={() => goto('/settings/storage')}
				>
					{#snippet media()}
						<HardDrive class="h-5 w-5 text-indigo-600 dark:text-indigo-400" aria-hidden="true" />
					{/snippet}
				</ListItem>
			{/if}
			<ListItem
				link
				title={i18n.t('settings.aboutTitle')}
				subtitle={i18n.t('settings.aboutSub')}
				onclick={() => goto('/settings/about')}
			>
				{#snippet media()}
					<Info class="h-5 w-5 xm-text-secondary" aria-hidden="true" />
				{/snippet}
			</ListItem>
		</List>
	</div>
</AppShell>
