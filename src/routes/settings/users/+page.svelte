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
	import { BlockTitle, Button, List, ListItem, Segmented, SegmentedButton } from 'konsta/svelte';
	import { Trash2 } from 'lucide-svelte';

	import { userApi } from '#lib/api/user.ts';
	import type { User } from '#lib/types/user.ts';
	import { i18n } from '#lib/i18n/index.svelte.ts';
	import { authState } from '#lib/state/auth.svelte.ts';
	import { confirmDeleteState } from '#lib/state/confirm-delete.svelte.ts';
	import { toastState } from '#lib/state/toast.svelte.ts';
	import AppShell from '#lib/components/layout/AppShell.svelte';

	let users = $state<User[]>([]);
	let username = $state('');
	let password = $state('');
	let nickname = $state('');
	let role = $state<'USER' | 'ADMIN'>('USER');
	let busy = $state(false);

	async function load(): Promise<void> {
		users = await userApi.list(true);
	}

	async function createAccount(e?: SubmitEvent): Promise<void> {
		e?.preventDefault();
		if (!username.trim() || !password) return;
		busy = true;
		try {
			await userApi.adminCreate({
				username: username.trim().toLowerCase(),
				password,
				role,
				nickname: nickname.trim() || undefined
			});
			username = '';
			password = '';
			nickname = '';
			await load();
			await authState.refresh();
			toastState.show(i18n.t('users.created'), 'success');
		} catch (err) {
			toastState.show(err instanceof Error ? err.message : String(err), 'error');
		} finally {
			busy = false;
		}
	}

	onMount(() => {
		void load();
	});
</script>

<AppShell title={i18n.t('users.title')} backHref="/settings" backLabel={i18n.t('nav.settings')} showFab={false}>
	<div class="space-y-4">
		<form class="glass-surface space-y-3.5 rounded-2xl p-4" onsubmit={createAccount}>
			<h2 class="text-sm font-bold xm-text-primary">{i18n.t('users.addAccount')}</h2>
			<div>
				<label for="new-user-username" class="mb-1 block text-xs font-semibold xm-text-secondary">
					{i18n.t('auth.username')}
				</label>
				<input
					id="new-user-username"
					type="text"
					autocapitalize="none"
					autocorrect="off"
					spellcheck={false}
					required
					value={username}
					oninput={(e) => {
						username = e.currentTarget.value.toLowerCase();
					}}
					placeholder={i18n.t('auth.username')}
					class="xm-input"
				/>
			</div>
			<div>
				<label for="new-user-nickname" class="mb-1 block text-xs font-semibold xm-text-secondary">
					{i18n.t('auth.nickname')}
				</label>
				<input
					id="new-user-nickname"
					type="text"
					bind:value={nickname}
					placeholder={i18n.t('auth.nicknamePlaceholder')}
					class="xm-input"
				/>
			</div>
			<div>
				<label for="new-user-password" class="mb-1 block text-xs font-semibold xm-text-secondary">
					{i18n.t('auth.password')}
				</label>
				<input
					id="new-user-password"
					type="password"
					required
					autocomplete="new-password"
					bind:value={password}
					placeholder={i18n.t('auth.password')}
					class="xm-input"
				/>
			</div>
			{#if authState.isHost}
				<div>
					<span class="mb-1.5 block text-xs font-semibold xm-text-secondary">{i18n.t('users.role')}</span>
					<Segmented rounded strong>
						<SegmentedButton active={role === 'USER'} onclick={() => (role = 'USER')}>
							USER
						</SegmentedButton>
						<SegmentedButton active={role === 'ADMIN'} onclick={() => (role = 'ADMIN')}>
							ADMIN
						</SegmentedButton>
					</Segmented>
				</div>
			{/if}
			<Button rounded type="submit" disabled={busy}>{i18n.t('users.createUser')}</Button>
		</form>

		<div>
			<BlockTitle class="!mt-1 !mb-2 !px-1">{i18n.t('users.localAccounts', { count: users.length })}</BlockTitle>
			<List strong inset class="!my-0 !mx-0">
				{#each users as u (u.id)}
					<ListItem
						title={u.nickname || u.username}
						subtitle={`@${u.username} • ${u.role} • ${u.rowStatus}`}
					>
						{#snippet after()}
							{#if u.role !== 'HOST' && u.id !== authState.currentUser?.id}
								<button
									type="button"
									aria-label="Delete user @{u.username}"
									class="flex h-10 w-10 items-center justify-center rounded-full xm-text-danger transition-colors hover:bg-[var(--xm-danger-soft)]"
									onclick={() => {
										void confirmDeleteState.run({
											title: i18n.t('users.deleteTitle', { username: u.username }),
											message: i18n.t('users.deleteMsg'),
											confirmLabel: i18n.t('common.delete'),
											onConfirm: async () => {
												await userApi.delete(u.id);
												await load();
												await authState.refresh();
												toastState.show(i18n.t('users.removed', { username: u.username }), 'info');
											}
										});
									}}
								>
									<Trash2 class="h-4 w-4" aria-hidden="true" />
								</button>
							{/if}
						{/snippet}
					</ListItem>
				{/each}
			</List>
		</div>
	</div>
</AppShell>
