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
	interface Props {
		variant?: 'memo' | 'media';
		count?: number;
		label?: string;
	}

	let { variant = 'memo', count = 3, label = 'Loading...' }: Props = $props();
	let indices = $derived(Array.from({ length: count }, (_, i) => i));
</script>

{#if variant === 'media'}
	<div
		class="grid grid-cols-2 gap-3 sm:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5"
		role="status"
		aria-busy="true"
		aria-label={label}
	>
		{#each indices as idx (idx)}
			<div class="glass-surface overflow-hidden rounded-2xl" aria-hidden="true">
				<div class="xm-skeleton aspect-square w-full"></div>
				<div class="space-y-1.5 p-2.5">
					<div class="xm-skeleton h-3 w-4/5 rounded-md"></div>
					<div class="xm-skeleton h-2.5 w-2/5 rounded-md"></div>
				</div>
			</div>
		{/each}
	</div>
{:else}
	<div class="space-y-3" role="status" aria-busy="true" aria-label={label}>
		{#each indices as idx (idx)}
			<div class="glass-surface space-y-3 rounded-2xl p-4" aria-hidden="true">
				<div class="flex items-center justify-between gap-2.5">
					<div class="flex items-center gap-2.5">
						<div class="xm-skeleton h-9 w-9 shrink-0 rounded-full"></div>
						<div class="space-y-1.5">
							<div class="xm-skeleton h-3.5 w-28 rounded-md"></div>
							<div class="xm-skeleton h-2.5 w-20 rounded-md"></div>
						</div>
					</div>
					<div class="xm-skeleton h-6 w-6 rounded-full"></div>
				</div>
				<div class="space-y-2 pt-0.5">
					<div class="xm-skeleton h-3.5 w-11/12 rounded-md"></div>
					<div class="xm-skeleton h-3.5 w-4/5 rounded-md"></div>
					{#if idx % 2 === 0}
						<div class="xm-skeleton h-3.5 w-3/5 rounded-md"></div>
					{/if}
				</div>
				<div class="flex items-center justify-between border-t border-[var(--xm-divider)] pt-2.5">
					<div class="flex items-center gap-1.5">
						<div class="xm-skeleton h-6 w-12 rounded-full"></div>
						<div class="xm-skeleton h-6 w-7 rounded-full"></div>
					</div>
					<div class="xm-skeleton h-5 w-10 rounded-full"></div>
				</div>
			</div>
		{/each}
	</div>
{/if}
