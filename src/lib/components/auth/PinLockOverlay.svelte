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
	import { Button } from 'konsta/svelte';
	import { Fingerprint, Lock } from 'lucide-svelte';

	import { authApi } from '#lib/api/auth.ts';
	import { i18n } from '#lib/i18n/index.svelte.ts';
	import { authState } from '#lib/state/auth.svelte.ts';
	import { toastState } from '#lib/state/toast.svelte.ts';
	import { handlePinKeydown, isValidSixDigitPin, PIN_LENGTH, PIN_PATTERN, sanitizePinInput } from '#lib/utils/pin.ts';
	import { supportsAndroidBiometric } from '#lib/utils/platform.ts';

	let pin = $state('');
	let checking = $state(false);
	let errorMsg = $state('');

	function handlePinInput(e: Event): void {
		const input = e.currentTarget as HTMLInputElement;
		const clean = sanitizePinInput(input.value);
		if (input.value !== clean) {
			input.value = clean;
		}
		pin = clean;
		if (errorMsg) errorMsg = '';
	}

	async function unlockWithPin(e?: SubmitEvent): Promise<void> {
		e?.preventDefault();
		if (checking) return;
		const cleanPin = sanitizePinInput(pin);
		pin = cleanPin;
		if (!isValidSixDigitPin(cleanPin)) {
			errorMsg = i18n.t('account.pinInvalidLength');
			toastState.show(errorMsg, 'info');
			return;
		}
		checking = true;
		errorMsg = '';
		try {
			const ok = await authApi.verifyPin(cleanPin);
			if (ok) {
				pin = '';
				errorMsg = '';
				authState.unlock();
			} else {
				errorMsg = i18n.t('pin.invalid');
				toastState.show(errorMsg, 'error');
				pin = '';
			}
		} catch (err) {
			errorMsg = err instanceof Error ? err.message : i18n.t('pin.invalid');
			toastState.show(errorMsg, 'error');
		} finally {
			checking = false;
		}
	}

	async function unlockWithBiometric(): Promise<void> {
		if (!supportsAndroidBiometric()) return;
		try {
			const res = await authApi.authenticateBiometric({
				title: i18n.t('pin.title'),
				subtitle: `@${authState.currentUser?.username ?? ''}`,
				negativeButtonText: i18n.t('pin.placeholder')
			});
			if (res.authenticated) {
				pin = '';
				errorMsg = '';
				authState.unlock();
			}
		} catch (err) {
			toastState.show(err instanceof Error ? err.message : 'Biometric unavailable', 'error');
		}
	}
</script>

{#if authState.isLocked}
	<div
		class="fixed inset-0 z-[60] flex items-center justify-center bg-black/75 p-6 backdrop-blur-2xl"
		role="dialog"
		aria-modal="true"
		aria-labelledby="pin-lock-heading"
	>
		<div class="glass-elevated w-full max-w-sm space-y-4 rounded-3xl p-6 text-center">
			<div
				class="mx-auto flex h-14 w-14 items-center justify-center rounded-2xl border border-[var(--xm-accent-border)] bg-[var(--xm-accent-soft)] text-[var(--xm-accent)]"
			>
				<Lock class="h-7 w-7" aria-hidden="true" />
			</div>
			<div>
				<h2 id="pin-lock-heading" class="text-lg font-bold xm-text-primary">{i18n.t('pin.title')}</h2>
				<p class="mt-0.5 text-xs xm-text-muted">@{authState.currentUser?.username}</p>
			</div>

			{#if authState.lockConfig?.hasPin}
				<form class="space-y-3" novalidate onsubmit={unlockWithPin}>
					<label for="pin-lock-input" class="sr-only">{i18n.t('account.pinLabel')}</label>
					<input
						id="pin-lock-input"
						type="password"
						inputmode="numeric"
						pattern={PIN_PATTERN}
						autocomplete="one-time-code"
						maxlength={PIN_LENGTH}
						value={pin}
						onkeydown={handlePinKeydown}
						oninput={handlePinInput}
						placeholder={i18n.t('pin.placeholder')}
						aria-invalid={Boolean(errorMsg)}
						class="xm-input text-center text-xl tracking-[0.4em] tabular-nums"
					/>
					{#if errorMsg}
						<p class="text-xs font-medium text-[#d70015] dark:text-[#ff453a]" role="alert">
							{errorMsg}
						</p>
					{/if}
					<Button large rounded type="submit" disabled={checking || pin.length !== PIN_LENGTH}>
						{i18n.t('pin.unlock')}
					</Button>
				</form>
			{/if}

			{#if supportsAndroidBiometric() && authState.lockConfig?.biometricEnabled}
				<Button outline large rounded onclick={unlockWithBiometric}>
					<Fingerprint class="mr-2 h-5 w-5" aria-hidden="true" /> {i18n.t('pin.useBiometric')}
				</Button>
			{/if}
		</div>
	</div>
{/if}
