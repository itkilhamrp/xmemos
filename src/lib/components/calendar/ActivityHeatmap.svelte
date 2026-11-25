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
	import { ChevronLeft, ChevronRight } from 'lucide-svelte';

	import { i18n } from '#lib/i18n/index.svelte.ts';
	import type { HeatmapDay } from '#lib/types/memo.ts';
	import { toLocalDateKey } from '#lib/utils/date.ts';

	interface Props {
		days: HeatmapDay[];
		selectedDate?: string | null;
		weeks?: number;
		onSelectDate?: (date: string | null) => void;
	}

	let { days, selectedDate = null, weeks: _weeks = 16, onSelectDate }: Props = $props();

	const now = new Date();
	const todayKey = toLocalDateKey(now);

	let viewYear = $state(now.getFullYear());
	let viewMonth = $state(now.getMonth()); // 0-indexed

	let countsMap = $derived(new Map(days.map((d) => [d.date, d.count])));

	let isCurrentMonth = $derived(viewYear === now.getFullYear() && viewMonth === now.getMonth());

	let monthLabel = $derived.by(() => {
		const dt = new Date(viewYear, viewMonth, 1);
		const formatted = new Intl.DateTimeFormat(i18n.bcp47, {
			month: 'long',
			year: 'numeric'
		}).format(dt);
		return formatted.charAt(0).toUpperCase() + formatted.slice(1);
	});

	let weekdayHeaders = $derived.by(() => {
		const fmt = new Intl.DateTimeFormat(i18n.bcp47, { weekday: 'narrow' });
		// 2026-11-01 is a Sunday — iterate Sun..Sat cleanly in local time
		return Array.from({ length: 7 }, (_, i) => fmt.format(new Date(2026, 10, 1 + i)));
	});

	interface CalendarCell {
		date: string;
		day: number;
		inCurrentMonth: boolean;
		isToday: boolean;
		count: number;
	}

	let calendarCells = $derived.by((): CalendarCell[] => {
		const firstOfMonth = new Date(viewYear, viewMonth, 1);
		const startWeekday = firstOfMonth.getDay(); // 0 = Sun
		const daysInMonth = new Date(viewYear, viewMonth + 1, 0).getDate();
		const totalSlots = Math.ceil((startWeekday + daysInMonth) / 7) * 7;

		const cells: CalendarCell[] = [];
		for (let i = 0; i < totalSlots; i++) {
			const dt = new Date(viewYear, viewMonth, 1 - startWeekday + i);
			const key = toLocalDateKey(dt);
			const inCurrentMonth = dt.getMonth() === viewMonth && dt.getFullYear() === viewYear;
			cells.push({
				date: key,
				day: dt.getDate(),
				inCurrentMonth,
				isToday: key === todayKey,
				count: countsMap.get(key) ?? 0
			});
		}
		return cells;
	});

	let monthMemoTotal = $derived(
		calendarCells.reduce((sum, cell) => (cell.inCurrentMonth ? sum + cell.count : sum), 0)
	);

	function prevMonth(): void {
		if (viewMonth === 0) {
			viewYear -= 1;
			viewMonth = 11;
		} else {
			viewMonth -= 1;
		}
	}

	function nextMonth(): void {
		if (viewMonth === 11) {
			viewYear += 1;
			viewMonth = 0;
		} else {
			viewMonth += 1;
		}
	}

	function jumpToCurrentMonth(): void {
		const current = new Date();
		viewYear = current.getFullYear();
		viewMonth = current.getMonth();
	}

	function handleDayClick(cell: CalendarCell): void {
		if (!cell.inCurrentMonth) {
			const [y, m] = cell.date.split('-').map(Number);
			if (y && m) {
				viewYear = y;
				viewMonth = m - 1;
			}
		}
		onSelectDate?.(selectedDate === cell.date ? null : cell.date);
	}

	function cellStyleClass(cell: CalendarCell): string {
		const isSelected = selectedDate === cell.date;
		const ring = isSelected
			? 'ring-2 ring-[var(--xm-accent)] ring-offset-1 ring-offset-white dark:ring-offset-[#1c1c1e]'
			: cell.isToday
				? 'ring-1.5 ring-[var(--xm-accent)]'
				: '';

		if (!cell.inCurrentMonth) {
			return `xm-text-muted opacity-35 hover:opacity-75 hover:bg-[var(--xm-surface-subtle)] ${ring}`;
		}

		if (cell.count === 0) {
			if (cell.isToday) {
				return `bg-[var(--xm-accent-soft)] xm-text-accent font-bold ${ring}`;
			}
			return `xm-text-secondary hover:bg-[var(--xm-surface-hover)] ${ring}`;
		}

		if (cell.count === 1) {
			return `bg-sky-500/20 text-sky-800 dark:bg-sky-400/24 dark:text-sky-100 font-semibold hover:bg-sky-500/30 dark:hover:bg-sky-400/34 ${ring}`;
		}
		if (cell.count <= 3) {
			return `bg-sky-500/42 text-sky-950 dark:bg-sky-400/46 dark:text-white font-semibold hover:bg-sky-500/52 dark:hover:bg-sky-400/56 ${ring}`;
		}
		if (cell.count <= 6) {
			return `bg-sky-600/80 text-white dark:bg-sky-400/75 dark:text-slate-950 font-bold ${ring}`;
		}
		return `bg-sky-600 text-white dark:bg-sky-300 dark:text-slate-950 font-bold ${ring}`;
	}
</script>

<section class="glass-surface rounded-2xl p-3.5" aria-label={i18n.t('heatmap.title')}>
	<!-- Official Memos 26.11 Monthly Calendar Header -->
	<div class="mb-2.5 flex items-center justify-between gap-2">
		<button
			type="button"
			title={isCurrentMonth ? monthLabel : i18n.t('heatmap.jumpToday')}
			class="rounded-lg px-1 py-0.5 text-[13px] font-semibold tracking-tight xm-text-primary transition-colors hover:bg-[var(--xm-surface-subtle)]"
			onclick={jumpToCurrentMonth}
		>
			{monthLabel}
		</button>
		<div class="flex items-center gap-0.5">
			<button
				type="button"
				aria-label={i18n.t('heatmap.prevMonth')}
				class="flex h-7 w-7 items-center justify-center rounded-lg xm-text-secondary transition-colors hover:bg-[var(--xm-surface-hover)] hover:text-[var(--xm-text-primary)]"
				onclick={prevMonth}
			>
				<ChevronLeft class="h-4 w-4" aria-hidden="true" />
			</button>
			<button
				type="button"
				aria-label={i18n.t('heatmap.nextMonth')}
				class="flex h-7 w-7 items-center justify-center rounded-lg xm-text-secondary transition-colors hover:bg-[var(--xm-surface-hover)] hover:text-[var(--xm-text-primary)]"
				onclick={nextMonth}
			>
				<ChevronRight class="h-4 w-4" aria-hidden="true" />
			</button>
			{#if !isCurrentMonth}
				<button
					type="button"
					class="ml-0.5 rounded-lg px-2 py-1 text-[11px] font-medium xm-text-secondary transition-colors hover:bg-[var(--xm-surface-hover)] hover:text-[var(--xm-text-primary)]"
					onclick={jumpToCurrentMonth}
				>
					{i18n.t('heatmap.today')}
				</button>
			{/if}
		</div>
	</div>

	<!-- Localized Weekday Labels (S M T W T F S) -->
	<div class="mb-1 grid grid-cols-7 gap-1 text-center" aria-hidden="true">
		{#each weekdayHeaders as wd, i (i)}
			<span class="py-0.5 text-[11px] font-medium xm-text-muted">{wd}</span>
		{/each}
	</div>

	<!-- Monthly Day-Number Matrix (7 columns × 5–6 weeks) -->
	<div class="grid grid-cols-7 gap-1" role="grid" aria-label={monthLabel}>
		{#each calendarCells as cell (cell.date)}
			{@const memoWord = i18n.t(cell.count === 1 ? 'heatmap.memoSingular' : 'heatmap.memoPlural')}
			<button
				type="button"
				title="{cell.date}: {cell.count} {memoWord}"
				aria-label="{cell.date}: {cell.count} {memoWord}"
				aria-pressed={selectedDate === cell.date}
				class="flex h-8 w-full items-center justify-center rounded-lg text-[12px] tabular-nums transition-colors active:scale-95 {cellStyleClass(cell)}"
				onclick={() => handleDayClick(cell)}
			>
				{cell.day}
			</button>
		{/each}
	</div>

	<!-- Compact Footer: Active Filter Clear / Month Total + Intensity Legend -->
	<div class="mt-2.5 flex items-center justify-between gap-2 text-[11px]">
		{#if selectedDate}
			<button
				type="button"
				class="rounded-md px-1.5 py-0.5 font-semibold xm-text-accent transition-colors hover:bg-[var(--xm-accent-soft)]"
				onclick={() => onSelectDate?.(null)}
			>
				{i18n.t('tags.clearFilter')} {selectedDate}
			</button>
		{:else}
			<span class="xm-text-muted tabular-nums">
				{i18n.t('heatmap.count', {
					count: monthMemoTotal,
					label: i18n.t(monthMemoTotal === 1 ? 'heatmap.memoSingular' : 'heatmap.memoPlural')
				})}
			</span>
		{/if}
		<div class="flex items-center gap-1 xm-text-muted" aria-hidden="true">
			<span>{i18n.t('heatmap.less')}</span>
			<span class="h-2.5 w-2.5 rounded-[3px] bg-slate-900/10 dark:bg-white/10"></span>
			<span class="h-2.5 w-2.5 rounded-[3px] bg-sky-500/25 dark:bg-sky-400/28"></span>
			<span class="h-2.5 w-2.5 rounded-[3px] bg-sky-500/50 dark:bg-sky-400/55"></span>
			<span class="h-2.5 w-2.5 rounded-[3px] bg-sky-600 dark:bg-sky-300"></span>
			<span>{i18n.t('heatmap.more')}</span>
		</div>
	</div>
</section>
