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
	import { Button, List, ListItem, Toggle } from 'konsta/svelte';
	import { Lock } from 'lucide-svelte';

	import { authApi } from '#lib/api/auth.ts';
	import { systemApi } from '#lib/api/system.ts';
	import { userApi } from '#lib/api/user.ts';
	import { i18n } from '#lib/i18n/index.svelte.ts';
	import { authState } from '#lib/state/auth.svelte.ts';
	import { toastState } from '#lib/state/toast.svelte.ts';
	import { handlePinKeydown, isValidSixDigitPin, PIN_LENGTH, PIN_PATTERN, sanitizePinInput } from '#lib/utils/pin.ts';
	import { supportsAndroidBiometric } from '#lib/utils/platform.ts';
	import AppShell from '#lib/components/layout/AppShell.svelte';

	const PREF_REQUIRE_PW_AFTER_SIGNOUT = 'require_password_after_sign_out';

	let nickname = $state(authState.currentUser?.nickname ?? '');
	let email = $state(authState.currentUser?.email ?? '');
	let description = $state(authState.currentUser?.description ?? '');
	let newPassword = $state('');
	let confirmNewPassword = $state('');

	let newPin = $state('');
	let confirmNewPin = $state('');
	let biometricEnabled = $state(authState.lockConfig?.biometricEnabled ?? false);
	let requireUnlockOnSwitch = $state(authState.lockConfig?.requireUnlockOnSwitch ?? false);
	let requirePasswordAfterSignOut = $state(authState.requirePasswordAfterSignOut);
	let saving = $state(false);
	let togglingSignOutLock = $state(false);

	const canManageLocalLock = $derived(authState.isAdminOrHost);
	const trimmedPassword = $derived(newPassword.trim());
	const trimmedConfirm = $derived(confirmNewPassword.trim());
	const passwordTooShort = $derived(trimmedPassword.length > 0 && trimmedPassword.length < 4);
	const passwordMismatch = $derived(
		(trimmedPassword.length > 0 || trimmedConfirm.length > 0) &&
			trimmedPassword !== trimmedConfirm
	);

	const hasPinInput = $derived(newPin.length > 0 || confirmNewPin.length > 0);
	const pinInvalidLength = $derived(newPin.length > 0 && !isValidSixDigitPin(newPin));
	const pinMismatch = $derived(hasPinInput && newPin !== confirmNewPin);
	const pinFormInvalid = $derived(hasPinInput && (!isValidSixDigitPin(newPin) || pinMismatch));

	$effect(() => {
		requirePasswordAfterSignOut = authState.requirePasswordAfterSignOut;
		if (authState.currentUser) {
			nickname = authState.currentUser.nickname;
			email = authState.currentUser.email;
			description = authState.currentUser.description;
		}
		if (authState.lockConfig) {
			biometricEnabled = authState.lockConfig.biometricEnabled;
			requireUnlockOnSwitch = authState.lockConfig.requireUnlockOnSwitch;
		}
	});

	function onNewPinInput(e: Event): void {
		const input = e.currentTarget as HTMLInputElement;
		const clean = sanitizePinInput(input.value);
		if (input.value !== clean) {
			input.value = clean;
		}
		newPin = clean;
	}

	function onConfirmPinInput(e: Event): void {
		const input = e.currentTarget as HTMLInputElement;
		const clean = sanitizePinInput(input.value);
		if (input.value !== clean) {
			input.value = clean;
		}
		confirmNewPin = clean;
	}

	async function saveProfile(e?: SubmitEvent): Promise<void> {
		e?.preventDefault();
		if (!authState.isAuthenticated) return;
		if (trimmedPassword.length > 0 || trimmedConfirm.length > 0) {
			if (trimmedPassword.length < 4) {
				toastState.show(i18n.t('account.passwordTooShort'), 'error');
				return;
			}
			if (trimmedPassword !== trimmedConfirm) {
				toastState.show(i18n.t('account.passwordMismatch'), 'error');
				return;
			}
		}

		saving = true;
		try {
			await userApi.update({
				nickname,
				email,
				description,
				password: trimmedPassword || undefined
			});
			newPassword = '';
			confirmNewPassword = '';
			await authState.refresh();
			toastState.show(i18n.t('account.profileUpdated'), 'success');
		} catch (err) {
			toastState.show(err instanceof Error ? err.message : String(err), 'error');
		} finally {
			saving = false;
		}
	}

	async function toggleRequirePasswordAfterSignOut(): Promise<void> {
		if (!authState.isAuthenticated || togglingSignOutLock) return;
		if (!canManageLocalLock) return;
		const next = !requirePasswordAfterSignOut;
		requirePasswordAfterSignOut = next;
		togglingSignOutLock = true;
		try {
			await systemApi.setLocalPref(PREF_REQUIRE_PW_AFTER_SIGNOUT, next ? 'true' : 'false');
			await authState.refresh();
			toastState.show(
				i18n.t(next ? 'account.requirePasswordOn' : 'account.requirePasswordOff'),
				'info'
			);
		} catch (err) {
			requirePasswordAfterSignOut = !next;
			toastState.show(err instanceof Error ? err.message : String(err), 'error');
		} finally {
			togglingSignOutLock = false;
		}
	}

	async function saveLock(clearPin = false, e?: SubmitEvent): Promise<void> {
		e?.preventDefault();
		if (!authState.isAuthenticated || saving) return;
		if (!canManageLocalLock) return;

		const cleanPin = sanitizePinInput(newPin);
		const cleanConfirm = sanitizePinInput(confirmNewPin);
		newPin = cleanPin;
		confirmNewPin = cleanConfirm;

		if (!clearPin && (cleanPin.length > 0 || cleanConfirm.length > 0)) {
			if (!isValidSixDigitPin(cleanPin)) {
				toastState.show(i18n.t('account.pinInvalidLength'), 'error');
				return;
			}
			if (cleanPin !== cleanConfirm) {
				toastState.show(i18n.t('account.pinMismatch'), 'error');
				return;
			}
		}

		saving = true;
		try {
			await authApi.setLockConfig({
				pin: clearPin ? null : cleanPin || null,
				clearPin,
				biometricEnabled: supportsAndroidBiometric() ? biometricEnabled : false,
				requireUnlockOnSwitch
			});
			newPin = '';
			confirmNewPin = '';
			await authState.refresh();
			toastState.show(i18n.t(clearPin ? 'account.pinCleared' : 'account.lockUpdated'), 'success');
		} catch (err) {
			toastState.show(err instanceof Error ? err.message : String(err), 'error');
		} finally {
			saving = false;
		}
	}
</script>

<AppShell title={i18n.t('account.title')} backHref="/settings" backLabel={i18n.t('nav.settings')} showFab={false}>
	{#if !authState.isAuthenticated}
		<div class="glass-surface mx-auto max-w-md space-y-4 rounded-3xl p-6 text-center">
			<div
				class="mx-auto flex h-12 w-12 items-center justify-center rounded-2xl bg-[var(--xm-accent-soft)] xm-text-accent"
				aria-hidden="true"
			>
				<Lock class="h-6 w-6" />
			</div>
			<div class="space-y-1.5">
				<h2 class="text-base font-bold xm-text-primary">{i18n.t('guest.accountTitle')}</h2>
				<p class="text-xs leading-relaxed xm-text-secondary">
					{i18n.t('guest.accountDesc')}
				</p>
			</div>
			<Button rounded onclick={() => goto('/auth')}>{i18n.t('auth.signIn')}</Button>
		</div>
	{:else}
		<div class="space-y-4">
			<form class="glass-surface space-y-3.5 rounded-2xl p-4" onsubmit={saveProfile}>
				<h2 class="text-sm font-bold xm-text-primary">{i18n.t('account.profileDetails')}</h2>
				<div>
					<label for="acc-nickname" class="mb-1 block text-xs font-semibold xm-text-secondary">
						{i18n.t('account.nickname')}
					</label>
					<input
						id="acc-nickname"
						type="text"
						bind:value={nickname}
						placeholder={i18n.t('auth.nicknamePlaceholder')}
						class="xm-input"
					/>
				</div>
				<div>
					<label for="acc-email" class="mb-1 block text-xs font-semibold xm-text-secondary">
						{i18n.t('account.email')}
					</label>
					<input
						id="acc-email"
						type="email"
						bind:value={email}
						placeholder="you@example.com"
						class="xm-input"
					/>
				</div>
				<div>
					<label for="acc-bio" class="mb-1 block text-xs font-semibold xm-text-secondary">
						{i18n.t('account.bio')}
					</label>
					<textarea
						id="acc-bio"
						rows="3"
						bind:value={description}
						placeholder={i18n.t('account.bioPlaceholder')}
						class="xm-input resize-y"
					></textarea>
				</div>
				<div>
					<label for="acc-password" class="mb-1 block text-xs font-semibold xm-text-secondary">
						{i18n.t('account.newPassword')}
					</label>
					<input
						id="acc-password"
						type="password"
						autocomplete="new-password"
						bind:value={newPassword}
						placeholder={i18n.t('account.newPasswordPlaceholder')}
						class="xm-input"
					/>
					{#if passwordTooShort}
						<p class="mt-1 text-xs font-medium text-[#d70015] dark:text-[#ff453a]">
							{i18n.t('account.passwordTooShort')}
						</p>
					{/if}
				</div>
				<div>
					<label for="acc-password-confirm" class="mb-1 block text-xs font-semibold xm-text-secondary">
						{i18n.t('account.confirmPassword')}
					</label>
					<input
						id="acc-password-confirm"
						type="password"
						autocomplete="new-password"
						bind:value={confirmNewPassword}
						placeholder={i18n.t('account.confirmPasswordPlaceholder')}
						class="xm-input"
					/>
					{#if trimmedConfirm.length > 0 && passwordMismatch}
						<p class="mt-1 text-xs font-medium text-[#d70015] dark:text-[#ff453a]">
							{i18n.t('account.passwordMismatch')}
						</p>
					{/if}
				</div>
				<Button rounded type="submit" disabled={saving}>{i18n.t('account.saveProfile')}</Button>
			</form>

			{#if canManageLocalLock}
				<form
					class="glass-surface space-y-3.5 rounded-2xl p-4"
					aria-labelledby="lock-settings-heading"
					novalidate
					onsubmit={(e) => saveLock(false, e)}
				>
				<div class="flex items-center justify-between gap-2">
					<h2 id="lock-settings-heading" class="text-sm font-bold xm-text-primary">
						{i18n.t('account.lockTitle')}
					</h2>
					{#if authState.lockConfig?.hasPin}
						<div class="flex items-center gap-2">
							<span
								class="rounded-full bg-[var(--xm-accent-soft)] px-2.5 py-0.5 text-[11px] font-semibold xm-text-accent"
							>
								{i18n.t('account.pinActiveBadge')}
							</span>
							<button
								type="button"
								class="rounded-full border border-[var(--xm-divider)] px-2.5 py-0.5 text-[11px] font-semibold xm-text-secondary transition-colors hover:bg-[var(--xm-surface-subtle)]"
								onclick={() => authState.lock()}
							>
								{i18n.t('account.lockNow')}
							</button>
						</div>
					{/if}
				</div>
				<p class="text-xs leading-relaxed xm-text-secondary">
					{i18n.t('account.lockDesc')}
				</p>
				<div>
					<div class="mb-1 flex items-center justify-between gap-2">
						<label for="acc-pin" class="block text-xs font-semibold xm-text-secondary">
							{i18n.t('account.pinLabel')}
						</label>
						<span class="text-[11px] xm-text-muted tabular-nums">
							{newPin.length}/{PIN_LENGTH}
						</span>
					</div>
					<input
						id="acc-pin"
						type="password"
						inputmode="numeric"
						pattern={PIN_PATTERN}
						autocomplete="one-time-code"
						maxlength={PIN_LENGTH}
						value={newPin}
						onkeydown={handlePinKeydown}
						oninput={onNewPinInput}
						placeholder={i18n.t('account.pinPlaceholder')}
						aria-invalid={pinInvalidLength}
						class="xm-input tabular-nums"
					/>
					{#if pinInvalidLength}
						<p class="mt-1 text-xs font-medium text-[#d70015] dark:text-[#ff453a]">
							{i18n.t('account.pinInvalidLength')}
						</p>
					{/if}
				</div>
				<div>
					<div class="mb-1 flex items-center justify-between gap-2">
						<label for="acc-pin-confirm" class="block text-xs font-semibold xm-text-secondary">
							{i18n.t('account.confirmPin')}
						</label>
						<span class="text-[11px] xm-text-muted tabular-nums">
							{confirmNewPin.length}/{PIN_LENGTH}
						</span>
					</div>
					<input
						id="acc-pin-confirm"
						type="password"
						inputmode="numeric"
						pattern={PIN_PATTERN}
						autocomplete="one-time-code"
						maxlength={PIN_LENGTH}
						value={confirmNewPin}
						onkeydown={handlePinKeydown}
						oninput={onConfirmPinInput}
						placeholder={i18n.t('account.confirmPinPlaceholder')}
						aria-invalid={confirmNewPin.length > 0 && pinMismatch}
						class="xm-input tabular-nums"
					/>
					{#if confirmNewPin.length > 0 && pinMismatch}
						<p class="mt-1 text-xs font-medium text-[#d70015] dark:text-[#ff453a]">
							{i18n.t('account.pinMismatch')}
						</p>
					{/if}
				</div>
				<List strong inset class="!my-0 !mx-0">
					{#if supportsAndroidBiometric()}
						<ListItem title={i18n.t('account.biometric')}>
							{#snippet after()}
								<Toggle
									checked={biometricEnabled}
									onchange={() => (biometricEnabled = !biometricEnabled)}
								/>
							{/snippet}
						</ListItem>
					{/if}
					<ListItem title={i18n.t('account.requireUnlockSwitch')}>
						{#snippet after()}
							<Toggle
								checked={requireUnlockOnSwitch}
								onchange={() => (requireUnlockOnSwitch = !requireUnlockOnSwitch)}
							/>
						{/snippet}
					</ListItem>
					<ListItem
						title={i18n.t('account.requirePasswordAfterSignOut')}
						subtitle={i18n.t('account.requirePasswordAfterSignOutSub')}
					>
						{#snippet after()}
							<Toggle
								checked={requirePasswordAfterSignOut}
								disabled={togglingSignOutLock}
								onchange={toggleRequirePasswordAfterSignOut}
							/>
						{/snippet}
					</ListItem>
				</List>
				<div class="grid gap-2.5 {authState.lockConfig?.hasPin ? 'grid-cols-2' : 'grid-cols-1'}">
					<Button rounded type="submit" disabled={saving || pinFormInvalid}>
						{i18n.t('account.saveLock')}
					</Button>
					{#if authState.lockConfig?.hasPin}
						<Button
							tonal
							rounded
							type="button"
							class="!bg-[#d70015]/12 dark:!bg-[#ff453a]/16 !text-[#d70015] dark:!text-[#ff453a]"
							disabled={saving}
							onclick={() => saveLock(true)}
						>
							{i18n.t('account.clearPin')}
						</Button>
					{/if}
				</div>
				</form>
			{/if}
		</div>
	{/if}
</AppShell>
