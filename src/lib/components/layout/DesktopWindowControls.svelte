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

	import { i18n } from '#lib/i18n/index.svelte.ts';
	import {
		closeDesktopWindow,
		isDesktopTauri,
		minimizeDesktopWindow,
		subscribeDesktopWindowState,
		toggleMaximizeDesktopWindow
	} from '#lib/utils/platform.ts';

	interface Props {
		compact?: boolean;
	}

	let { compact = false }: Props = $props();

	const enabled = isDesktopTauri();
	let maximized = $state(false);
	let focused = $state(true);

	onMount(() => {
		if (!enabled) return;
		let cleanup: (() => void) | undefined;
		void subscribeDesktopWindowState((next) => {
			maximized = next.maximized;
			focused = next.focused;
		}).then((fn) => {
			cleanup = fn;
		});
		return () => {
			cleanup?.();
		};
	});
</script>

{#if enabled}
	<div
		class="xm-traffic-lights group/lights inline-flex shrink-0 items-center select-none {compact
			? 'gap-[7px]'
			: 'gap-[8px]'}"
		class:xm-window-unfocused={!focused}
		role="group"
		aria-label="Window controls"
		data-no-drag
	>
		<!-- Close (Red) -->
		<button
			type="button"
			data-no-drag
			class="xm-traffic-btn xm-traffic-close"
			aria-label={i18n.t('window.close')}
			title={i18n.t('window.close')}
			onclick={() => void closeDesktopWindow().catch(() => {})}
		>
			<svg viewBox="0 0 8 8" class="xm-traffic-glyph" aria-hidden="true">
				<path
					d="M1.6 1.6L6.4 6.4M6.4 1.6L1.6 6.4"
					stroke="currentColor"
					stroke-width="1.25"
					stroke-linecap="round"
				/>
			</svg>
		</button>

		<!-- Minimize (Yellow) -->
		<button
			type="button"
			data-no-drag
			class="xm-traffic-btn xm-traffic-minimize"
			aria-label={i18n.t('window.minimize')}
			title={i18n.t('window.minimize')}
			onclick={() => void minimizeDesktopWindow().catch(() => {})}
		>
			<svg viewBox="0 0 8 8" class="xm-traffic-glyph" aria-hidden="true">
				<path d="M1.4 4H6.6" stroke="currentColor" stroke-width="1.25" stroke-linecap="round" />
			</svg>
		</button>

		<!-- Zoom / Maximize / Restore (Green) -->
		<button
			type="button"
			data-no-drag
			class="xm-traffic-btn xm-traffic-maximize"
			aria-label={maximized ? i18n.t('window.restore') : i18n.t('window.maximize')}
			title={maximized ? i18n.t('window.restore') : i18n.t('window.maximize')}
			onclick={() => void toggleMaximizeDesktopWindow().catch(() => {})}
		>
			{#if maximized}
				<svg viewBox="0 0 8 8" class="xm-traffic-glyph" aria-hidden="true">
					<path
						d="M1.3 4.7H3.3V6.7M6.7 3.3H4.7V1.3"
						fill="none"
						stroke="currentColor"
						stroke-width="1.15"
						stroke-linecap="round"
						stroke-linejoin="round"
					/>
				</svg>
			{:else}
				<svg viewBox="0 0 8 8" class="xm-traffic-glyph" aria-hidden="true">
					<path
						d="M1.7 4.4V6.3H3.6M6.3 3.6V1.7H4.4"
						fill="none"
						stroke="currentColor"
						stroke-width="1.15"
						stroke-linecap="round"
						stroke-linejoin="round"
					/>
				</svg>
			{/if}
		</button>
	</div>
{/if}
