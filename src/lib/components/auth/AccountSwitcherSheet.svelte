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
	import { tick } from 'svelte';
	import { goto } from '$app/navigation';
	import { Button, List, ListItem } from 'konsta/svelte';
	import { Check, LogOut, UserPlus, X } from 'lucide-svelte';

	import { authApi } from '#lib/api/auth.ts';
	import { i18n } from '#lib/i18n/index.svelte.ts';
	import { authState } from '#lib/state/auth.svelte.ts';
	import { toastState } from '#lib/state/toast.svelte.ts';
	import { scrollFieldIntoSafeView } from '#lib/utils/keyboard.svelte.ts';
	import { supportsAndroidBiometric } from '#lib/utils/platform.ts';
	import type { User } from '#lib/types/user.ts';

	interface Props {
		opened: boolean;
		onClose: () => void;
	}

	let { opened, onClose }: Props = $props();
	let pendingUser = $state<User | null>(null);
	let unlockSecret = $state('');
	let busy = $state(false);
	let secretInputEl = $state<HTMLInputElement | null>(null);

	const requiresPasswordOnly = $derived(
		!authState.isAuthenticated && authState.requirePasswordAfterSignOut
	);

	function handleClose(): void {
		pendingUser = null;
		unlockSecret = '';
		onClose();
	}

	async function focusSecretInput(): Promise<void> {
		await tick();
		if (secretInputEl) {
			secretInputEl.focus({ preventScroll: true });
			scrollFieldIntoSafeView(secretInputEl);
		}
	}

	function handleWindowKeydown(event: KeyboardEvent): void {
		if (!opened) return;
		if (event.key === 'Escape') {
			event.preventDefault();
			handleClose();
		}
	}

	async function trySwitch(target: User): Promise<void> {
		if (target.id === authState.currentUser?.id) {
			handleClose();
			return;
		}
		if (requiresPasswordOnly) {
			pendingUser = target;
			unlockSecret = '';
			void focusSecretInput();
			return;
		}
		busy = true;
		try {
			await authState.switchAccount({ userId: target.id });
			toastState.show(i18n.t('switcher.switched', { username: target.username }), 'success');
			handleClose();
		} catch {
			// Account requires PIN, biometric, or password verification
			pendingUser = target;
			if (supportsAndroidBiometric()) {
				try {
					const bio = await authApi.checkBiometricAvailable();
					if (bio.available) {
						const authRes = await authApi.authenticateBiometric({
							title: `Switch to @${target.username}`,
							subtitle: 'Verify your identity',
							negativeButtonText: 'Use PIN / Password'
						});
						if (authRes.authenticated) {
							await authState.switchAccount({ userId: target.id, biometricVerified: true });
							toastState.show(i18n.t('switcher.switched', { username: target.username }), 'success');
							handleClose();
							return;
						}
					}
				} catch {
					// Fall through to PIN/Password input
				}
			}
			void focusSecretInput();
		} finally {
			busy = false;
		}
	}

	async function submitUnlock(e?: SubmitEvent): Promise<void> {
		e?.preventDefault();
		if (!pendingUser || !unlockSecret.trim()) return;
		busy = true;
		try {
			const secret = unlockSecret.trim();
			if (requiresPasswordOnly) {
				await authState.switchAccount({
					userId: pendingUser.id,
					password: secret
				});
			} else {
				const isSixDigits = /^\d{6}$/.test(secret);
				await authState.switchAccount({
					userId: pendingUser.id,
					pin: isSixDigits ? secret : undefined,
					password: secret
				});
			}
			toastState.show(i18n.t('switcher.switched', { username: pendingUser.username }), 'success');
			handleClose();
		} catch (err) {
			toastState.show(err instanceof Error ? err.message : 'Unlock failed', 'error');
		} finally {
			busy = false;
		}
	}
</script>

<svelte:window onkeydown={handleWindowKeydown} />

{#if opened}
	<!-- Keyboard-safe visualViewport-locked modal/sheet container (never jumps or gets covered on Android) -->
	<div
		class="xm-kb-modal-backdrop fixed right-0 left-0 z-[70] flex justify-center {pendingUser
			? 'items-center px-4 py-4'
			: 'items-end sm:items-center sm:px-4 sm:py-6'}"
		style="top: var(--xm-vvt, 0px); height: var(--xm-vvh, 100dvh);"
		role="dialog"
		aria-modal="true"
		aria-labelledby="account-switcher-heading"
	>
		<!-- Lightweight opacity-only scrim (zero backdrop-blur overhead on Android WebView) -->
		<button
			type="button"
			aria-label="Close account switcher"
			class="absolute inset-0 bg-black/45"
			onclick={handleClose}
		></button>

		<div
			class="xm-kb-sheet-panel relative z-10 w-full max-w-lg overflow-y-auto overscroll-contain {pendingUser
				? 'rounded-[26px]'
				: 'rounded-t-[26px] sm:rounded-[26px]'}"
			style="max-height: calc(var(--xm-vvh, 100dvh) - 24px);"
		>
			<div
				class="space-y-3.5 px-4 pt-2.5 {pendingUser
					? 'pb-4'
					: 'pb-[max(1.25rem,env(safe-area-inset-bottom))] sm:pb-4'}"
			>
				{#if !pendingUser}
					<!-- Native iOS Sheet Grabber -->
					<div class="mx-auto h-1 w-9 rounded-full bg-[var(--xm-divider)] sm:hidden" aria-hidden="true"></div>
				{/if}

				<div class="flex items-center justify-between gap-2 pt-0.5">
					<h2 id="account-switcher-heading" class="text-[17px] font-semibold tracking-tight xm-text-primary">
						{i18n.t('switcher.title')}
					</h2>
					<button
						type="button"
						aria-label="Close account switcher"
						class="flex h-9 w-9 items-center justify-center rounded-full bg-[var(--xm-surface-subtle)] xm-text-secondary transition-colors hover:bg-[var(--xm-surface-hover)]"
						onclick={handleClose}
					>
						<X class="h-4 w-4" aria-hidden="true" />
					</button>
				</div>

				{#if pendingUser}
					<form
						class="space-y-3.5 rounded-2xl border border-[var(--xm-divider)] bg-[var(--xm-surface-subtle)] p-4"
						onsubmit={submitUnlock}
					>
						<label for="switch-unlock-secret" class="block text-sm font-medium xm-text-primary">
							{requiresPasswordOnly
								? i18n.t('switcher.enterPassword', { username: pendingUser.username })
								: i18n.t('switcher.enterSecret', { username: pendingUser.username })}
						</label>
						<input
							bind:this={secretInputEl}
							id="switch-unlock-secret"
							type="password"
							autocomplete="current-password"
							enterkeyhint="done"
							bind:value={unlockSecret}
							onfocus={(e) => scrollFieldIntoSafeView(e.currentTarget)}
							placeholder={requiresPasswordOnly
								? i18n.t('switcher.passwordPlaceholder')
								: i18n.t('switcher.secretPlaceholder')}
							class="xm-input"
						/>
						<div class="grid grid-cols-2 gap-2.5 pt-0.5">
							<Button
								tonal
								rounded
								type="button"
								class="!h-11 !text-[14px] !font-semibold !bg-black/8 dark:!bg-white/14 !text-[var(--xm-text-primary)]"
								onclick={() => {
									pendingUser = null;
									unlockSecret = '';
								}}
							>
								{i18n.t('nav.back')}
							</Button>
							<Button rounded type="submit" class="!h-11 !text-[14px] !font-semibold" disabled={busy}>
								{requiresPasswordOnly ? i18n.t('auth.signIn') : i18n.t('switcher.unlock')}
							</Button>
						</div>
					</form>
				{:else}
					<List strong inset class="!my-0 !mx-0">
						{#each authState.localUsers as u (u.id)}
							<ListItem
								link
								chevron={false}
								title={u.nickname || u.username}
								subtitle={`@${u.username} • ${u.role}`}
								onclick={() => trySwitch(u)}
							>
								{#snippet media()}
									<span
										class="flex h-9 w-9 items-center justify-center rounded-full border border-[var(--xm-accent-border)] bg-[var(--xm-accent-soft)] text-xs font-bold xm-text-accent"
										aria-hidden="true"
									>
										{(u.nickname || u.username).slice(0, 1).toUpperCase()}
									</span>
								{/snippet}
								{#snippet after()}
									{#if u.id === authState.currentUser?.id}
										<Check class="h-4 w-4 xm-text-accent" aria-label="Active account" />
									{/if}
								{/snippet}
							</ListItem>
						{/each}
					</List>

					<div class="grid gap-2.5 pt-1 {authState.isAuthenticated ? 'grid-cols-2' : 'grid-cols-1'}">
						<Button
							tonal
							rounded
							class="!h-11 !text-[14px] !font-semibold"
							onclick={() => {
								handleClose();
								goto('/auth');
							}}
						>
							<UserPlus class="mr-1.5 h-4 w-4 shrink-0" aria-hidden="true" />
							<span>{i18n.t('switcher.addAccount')}</span>
						</Button>
						{#if authState.isAuthenticated}
							<Button
								tonal
								rounded
								class="!h-11 !text-[14px] !font-semibold !bg-[#d70015]/12 dark:!bg-[#ff453a]/16 !text-[#d70015] dark:!text-[#ff453a]"
								onclick={async () => {
									await authState.signOut();
									toastState.show(i18n.t('switcher.signedOut'), 'info');
									handleClose();
									goto('/explore');
								}}
							>
								<LogOut class="mr-1.5 h-4 w-4 shrink-0" aria-hidden="true" />
								<span>{i18n.t('switcher.signOut')}</span>
							</Button>
						{/if}
					</div>
				{/if}
			</div>
		</div>
	</div>
{/if}
