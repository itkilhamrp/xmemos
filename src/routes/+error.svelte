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
	import { i18n } from '#lib/i18n/index.svelte.ts';
	import { goto } from '$app/navigation';
	import { page } from '$app/state';
	import { Button, Page } from 'konsta/svelte';
	import { AlertTriangle, RotateCcw } from 'lucide-svelte';
</script>

<Page class="!bg-transparent flex min-h-dvh items-center justify-center p-4">
	<main class="glass-elevated w-full max-w-md space-y-4 rounded-3xl p-6 text-center">
		<div
			class="mx-auto flex h-14 w-14 items-center justify-center rounded-2xl bg-[var(--xm-danger-soft)] text-[var(--xm-danger)]"
		>
			<AlertTriangle class="h-7 w-7" aria-hidden="true" />
		</div>
		<div class="space-y-1">
			<h1 class="text-xl font-bold xm-text-primary">{i18n.t('error.title')} ({page.status})</h1>
			<p class="text-xs leading-relaxed xm-text-secondary">
				{page.error?.message || 'An unexpected application error occurred.'}
			</p>
		</div>
		<div class="flex flex-col gap-2 pt-1 sm:flex-row">
			<Button large rounded onclick={() => goto('/')}>{i18n.t('error.backHome')}</Button>
			<Button
				large
				rounded
				tonal
				onclick={() => {
					if (typeof window !== 'undefined') window.location.reload();
				}}
			>
				<RotateCcw class="mr-1.5 h-4 w-4" aria-hidden="true" /> Reload
			</Button>
		</div>
	</main>
</Page>
