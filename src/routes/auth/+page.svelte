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
	import { Button, Page, Segmented, SegmentedButton } from 'konsta/svelte';

	import { i18n } from '#lib/i18n/index.svelte.ts';
	import { authState } from '#lib/state/auth.svelte.ts';
	import { toastState } from '#lib/state/toast.svelte.ts';
	import DesktopWindowControls from '#lib/components/layout/DesktopWindowControls.svelte';
	import AppIcon from '#lib/components/ui/AppIcon.svelte';
	import { handleTitlebarMouseDown, isDesktopTauri } from '#lib/utils/platform.ts';

	let mode = $state<'signin' | 'signup'>('signin');
	let username = $state('');
	let password = $state('');
	let confirmPassword = $state('');
	let nickname = $state('');
	let busy = $state(false);
	const desktopWindow = isDesktopTauri();

	let isHostSetup = $derived(authState.initialized && !authState.hasHost);
	let isCreatingAccount = $derived(isHostSetup || mode === 'signup');
	let passwordTooShort = $derived(isCreatingAccount && password.length > 0 && password.length < 4);
	let passwordMismatch = $derived(
		isCreatingAccount && confirmPassword.length > 0 && password !== confirmPassword
	);
	let submitDisabled = $derived(
		busy ||
			(isCreatingAccount &&
				(password.length > 0 || confirmPassword.length > 0) &&
				(password.length < 4 || password !== confirmPassword))
	);

	function switchMode(next: 'signin' | 'signup'): void {
		if (mode === next) return;
		mode = next;
		confirmPassword = '';
	}

	async function handleSubmit(e: SubmitEvent): Promise<void> {
		e.preventDefault();
		const normalizedUsername = username.trim().toLowerCase();
		if (!normalizedUsername || !password) {
			toastState.show(i18n.t('auth.requiredFields'), 'info');
			return;
		}
		if (isCreatingAccount) {
			if (password.length < 4) {
				toastState.show(i18n.t('auth.passwordTooShort'), 'error');
				return;
			}
			if (password !== confirmPassword) {
				toastState.show(i18n.t('auth.passwordMismatch'), 'error');
				return;
			}
		}
		busy = true;
		try {
			if (isHostSetup) {
				await authState.setupHost(normalizedUsername, password, nickname.trim() || undefined);
				toastState.show(i18n.t('auth.hostCreated'), 'success');
			} else if (mode === 'signup') {
				await authState.signUp(normalizedUsername, password, nickname.trim() || undefined);
				toastState.show(i18n.t('auth.accountCreated'), 'success');
			} else {
				await authState.signIn(normalizedUsername, password);
				toastState.show(i18n.t('auth.signedIn'), 'success');
			}
			confirmPassword = '';
			await goto('/');
		} catch (err) {
			toastState.show(err instanceof Error ? err.message : String(err), 'error');
		} finally {
			busy = false;
		}
	}
</script>

<Page class="!bg-transparent flex min-h-dvh items-center justify-center p-4">
	{#if desktopWindow}
		<!-- svelte-ignore a11y_no_static_element_interactions -->
		<div
			data-tauri-drag-region
			onmousedown={handleTitlebarMouseDown}
			class="xm-drag-region fixed inset-x-0 top-0 z-40 flex h-11 items-center px-4"
		>
			<DesktopWindowControls />
		</div>
	{/if}
	<main class="glass-elevated w-full max-w-md space-y-5 rounded-3xl p-6">
		<div class="space-y-2 text-center">
			<AppIcon class="mx-auto h-16 w-16" decorative />
			<h1 class="text-2xl font-bold tracking-tight xm-text-primary">Xmemos</h1>
			<p class="text-xs leading-relaxed xm-text-secondary">
				{isHostSetup
					? i18n.t('auth.setupDesc')
					: i18n.t('auth.defaultDesc')}
			</p>
		</div>

		{#if !isHostSetup && !authState.disallowUserRegistration}
			<Segmented rounded strong>
				<SegmentedButton active={mode === 'signin'} onclick={() => switchMode('signin')}>
					{i18n.t('auth.signIn')}
				</SegmentedButton>
				<SegmentedButton active={mode === 'signup'} onclick={() => switchMode('signup')}>
					{i18n.t('auth.signUp')}
				</SegmentedButton>
			</Segmented>
		{/if}

		<form class="space-y-3.5" novalidate onsubmit={handleSubmit}>
			<div>
				<label for="auth-username" class="mb-1.5 block text-xs font-semibold xm-text-secondary">
					{i18n.t('auth.username')}
				</label>
				<input
					id="auth-username"
					type="text"
					autocomplete="username"
					autocapitalize="none"
					autocorrect="off"
					spellcheck={false}
					required
					value={username}
					oninput={(e) => {
						username = e.currentTarget.value.toLowerCase();
					}}
					placeholder={i18n.t('auth.usernamePlaceholder')}
					class="xm-input"
				/>
			</div>

			{#if isCreatingAccount}
				<div>
					<label for="auth-nickname" class="mb-1.5 block text-xs font-semibold xm-text-secondary">
						{i18n.t('auth.nickname')}
					</label>
					<input
						id="auth-nickname"
						type="text"
						bind:value={nickname}
						placeholder={i18n.t('auth.nicknamePlaceholder')}
						class="xm-input"
					/>
				</div>
			{/if}

			<div>
				<label for="auth-password" class="mb-1.5 block text-xs font-semibold xm-text-secondary">
					{i18n.t('auth.password')}
				</label>
				<input
					id="auth-password"
					type="password"
					required
					autocomplete={isCreatingAccount ? 'new-password' : 'current-password'}
					bind:value={password}
					placeholder={i18n.t('auth.passwordPlaceholder')}
					aria-invalid={passwordTooShort}
					class="xm-input"
				/>
				{#if passwordTooShort}
					<p class="mt-1 text-xs font-medium text-[#d70015] dark:text-[#ff453a]" role="alert">
						{i18n.t('auth.passwordTooShort')}
					</p>
				{/if}
			</div>

			{#if isCreatingAccount}
				<div>
					<label
						for="auth-password-confirm"
						class="mb-1.5 block text-xs font-semibold xm-text-secondary"
					>
						{i18n.t('auth.confirmPassword')}
					</label>
					<input
						id="auth-password-confirm"
						type="password"
						required
						autocomplete="new-password"
						bind:value={confirmPassword}
						placeholder={i18n.t('auth.confirmPasswordPlaceholder')}
						aria-invalid={passwordMismatch}
						class="xm-input"
					/>
					{#if passwordMismatch}
						<p class="mt-1 text-xs font-medium text-[#d70015] dark:text-[#ff453a]" role="alert">
							{i18n.t('auth.passwordMismatch')}
						</p>
					{/if}
				</div>
			{/if}

			<div class="pt-1">
				<Button large rounded type="submit" disabled={submitDisabled}>
					{busy
						? i18n.t('common.working')
						: isHostSetup
							? i18n.t('auth.createHost')
							: mode === 'signup'
								? i18n.t('auth.createLocal')
								: i18n.t('auth.signIn')}
				</Button>
			</div>
		</form>

		{#if !isHostSetup}
			<div class="text-center">
				<button
					type="button"
					class="rounded-lg px-3 py-2 text-xs font-semibold xm-text-accent hover:bg-[var(--xm-accent-soft)]"
					onclick={() => goto('/explore')}
				>
					{i18n.t('auth.continueGuest')}
				</button>
			</div>
		{/if}
	</main>
</Page>
