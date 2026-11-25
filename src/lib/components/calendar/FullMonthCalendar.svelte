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
	import {
		ChevronLeft,
		ChevronRight,
		ChevronsUpDown,
		Image as ImageIcon
	} from 'lucide-svelte';

	import { getMediaUrl } from '#lib/api/invoke.ts';
	import { i18n } from '#lib/i18n/index.svelte.ts';
	import type { MemoView } from '#lib/types/memo.ts';
	import { toLocalDateKey } from '#lib/utils/date.ts';
	import { stripMarkdownToPlainText } from '#lib/utils/markdown.ts';

	interface Props {
		memos: MemoView[];
		selectedDate?: string | null;
		onSelectDate?: (date: string | null) => void;
	}

	let { memos, selectedDate = null, onSelectDate }: Props = $props();

	const today = new Date();
	const todayKey = toLocalDateKey(today);

	let viewYear = $state(today.getFullYear());
	let viewMonth = $state(today.getMonth()); // 0-indexed
	let pickerOpened = $state(false);
	let pickerYear = $state(today.getFullYear());

	// Single O(N) grouping pass: local YYYY-MM-DD -> MemoView[], plus YYYY-MM counts & active years
	let indexed = $derived.by(() => {
		const byDay = new Map<string, MemoView[]>();
		const byMonth = new Map<string, number>();
		const yearsSet = new Set<number>([today.getFullYear()]);

		for (const item of memos) {
			const dt = new Date(item.memo.createdTs * 1000);
			if (Number.isNaN(dt.getTime())) continue;
			const dayKey = toLocalDateKey(dt);
			const monthKey = dayKey.slice(0, 7);
			yearsSet.add(dt.getFullYear());

			const list = byDay.get(dayKey);
			if (list) {
				list.push(item);
			} else {
				byDay.set(dayKey, [item]);
			}
			byMonth.set(monthKey, (byMonth.get(monthKey) ?? 0) + 1);
		}

		const years = Array.from(yearsSet).sort((a, b) => b - a);
		return { byDay, byMonth, years };
	});

	let monthLabel = $derived.by(() => {
		const dt = new Date(viewYear, viewMonth, 1);
		const formatted = new Intl.DateTimeFormat(i18n.bcp47, {
			month: 'long',
			year: 'numeric'
		}).format(dt);
		return formatted.charAt(0).toUpperCase() + formatted.slice(1);
	});

	let shortMonths = $derived.by(() => {
		const fmt = new Intl.DateTimeFormat(i18n.bcp47, { month: 'short' });
		return Array.from({ length: 12 }, (_, m) => fmt.format(new Date(2026, m, 1)));
	});

	let weekdayHeaders = $derived.by(() => {
		const fmt = new Intl.DateTimeFormat(i18n.bcp47, { weekday: 'short' });
		// 2026-11-01 is a Sunday
		return Array.from({ length: 7 }, (_, i) => fmt.format(new Date(2026, 10, 1 + i)));
	});

	let isCurrentMonth = $derived(
		viewYear === today.getFullYear() && viewMonth === today.getMonth()
	);

	interface PreviewLine {
		id: number;
		initial: string;
		excerpt: string;
	}

	interface DayCell {
		date: string;
		day: number;
		inCurrentMonth: boolean;
		isToday: boolean;
		totalCount: number;
		previews: PreviewLine[];
		overflowCount: number;
		thumbUrl: string | null;
	}

	function extractExcerpt(content: string): string {
		const plain = stripMarkdownToPlainText(content).replace(/\s+/g, ' ').trim();
		if (!plain) return '…';
		return plain.length > 68 ? `${plain.slice(0, 67)}…` : plain;
	}

	let calendarCells = $derived.by((): DayCell[] => {
		const firstOfMonth = new Date(viewYear, viewMonth, 1);
		const startWeekday = firstOfMonth.getDay(); // 0 = Sun
		const daysInMonth = new Date(viewYear, viewMonth + 1, 0).getDate();
		const totalSlots = Math.ceil((startWeekday + daysInMonth) / 7) * 7;

		const cells: DayCell[] = [];
		for (let i = 0; i < totalSlots; i++) {
			const dt = new Date(viewYear, viewMonth, 1 - startWeekday + i);
			const date = toLocalDateKey(dt);
			const inCurrentMonth = dt.getMonth() === viewMonth && dt.getFullYear() === viewYear;
			const dayMemos = indexed.byDay.get(date) ?? [];
			const maxPreviews = dayMemos.length > 2 ? 2 : dayMemos.length;
			const previews: PreviewLine[] = dayMemos.slice(0, maxPreviews).map((m) => ({
				id: m.memo.id,
				initial: (m.creator?.nickname || m.creator?.username || 'U').slice(0, 1).toUpperCase(),
				excerpt: extractExcerpt(m.memo.content)
			}));

			let thumbUrl: string | null = null;
			for (const m of dayMemos) {
				const img = m.resources.find((r) => r.type.startsWith('image/'));
				if (img) {
					thumbUrl = getMediaUrl(img.uid, true);
					break;
				}
			}

			cells.push({
				date,
				day: dt.getDate(),
				inCurrentMonth,
				isToday: date === todayKey,
				totalCount: dayMemos.length,
				previews,
				overflowCount: Math.max(0, dayMemos.length - previews.length),
				thumbUrl
			});
		}
		return cells;
	});

	let weekRows = $derived(Math.max(4, Math.ceil(calendarCells.length / 7)));

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

	function jumpToToday(): void {
		const now = new Date();
		viewYear = now.getFullYear();
		viewMonth = now.getMonth();
		pickerOpened = false;
		onSelectDate?.(toLocalDateKey(now));
	}

	function togglePicker(): void {
		if (!pickerOpened) {
			pickerYear = viewYear;
		}
		pickerOpened = !pickerOpened;
	}

	function selectMonth(monthIndex: number): void {
		viewYear = pickerYear;
		viewMonth = monthIndex;
		pickerOpened = false;
	}

	function handleDayClick(cell: DayCell): void {
		if (!cell.inCurrentMonth) {
			const [y, m] = cell.date.split('-').map(Number);
			if (y && m) {
				viewYear = y;
				viewMonth = m - 1;
			}
		}
		pickerOpened = false;
		onSelectDate?.(selectedDate === cell.date ? null : cell.date);
	}

	function handleKeydown(event: KeyboardEvent): void {
		const tag = (event.target as HTMLElement | null)?.tagName;
		if (tag === 'INPUT' || tag === 'TEXTAREA') return;
		if (event.key === 'Escape' && pickerOpened) {
			pickerOpened = false;
			return;
		}
		if (pickerOpened) {
			if (event.key === 'ArrowLeft') {
				event.preventDefault();
				pickerYear -= 1;
			} else if (event.key === 'ArrowRight') {
				event.preventDefault();
				pickerYear += 1;
			}
			return;
		}
		if (event.altKey && event.key === 'ArrowLeft') {
			event.preventDefault();
			prevMonth();
		} else if (event.altKey && event.key === 'ArrowRight') {
			event.preventDefault();
			nextMonth();
		}
	}

	function monthMemoCount(year: number, monthIndex: number): number {
		const key = `${year}-${String(monthIndex + 1).padStart(2, '0')}`;
		return indexed.byMonth.get(key) ?? 0;
	}
</script>

<svelte:window onkeydown={handleKeydown} />

<section class="relative flex flex-col gap-1.5 md:h-full md:min-h-0" aria-label={i18n.t('calendar.tabCalendar')}>
	<!-- Official Memos 26.11 Top Calendar Header -->
	<div class="flex shrink-0 flex-wrap items-center justify-between gap-2 px-0.5">
		<div class="relative">
			<button
				type="button"
				aria-expanded={pickerOpened}
				aria-label={i18n.t('calendar.pickMonthYear')}
				class="inline-flex items-center gap-1.5 rounded-xl px-2 py-0.5 text-base font-bold tracking-tight xm-text-primary transition-colors hover:bg-[var(--xm-surface-subtle)] sm:text-lg"
				onclick={togglePicker}
			>
				<span>{monthLabel}</span>
				<ChevronsUpDown class="h-4 w-4 xm-text-muted" aria-hidden="true" />
			</button>

			<!-- Konsta iOS Glass Month/Year Picker Popover -->
			{#if pickerOpened}
				<div
					role="dialog"
					aria-label={i18n.t('calendar.pickMonthYear')}
					class="glass-surface absolute top-full left-0 z-30 mt-1.5 w-[268px] rounded-2xl border border-[var(--xm-divider)] p-3 shadow-xl sm:w-[292px]"
				>
					<div class="mb-2.5 flex items-center justify-between">
						<button
							type="button"
							aria-label="Previous year"
							class="flex h-8 w-8 items-center justify-center rounded-full xm-text-secondary transition-colors hover:bg-[var(--xm-surface-hover)]"
							onclick={() => (pickerYear -= 1)}
						>
							<ChevronLeft class="h-4 w-4" aria-hidden="true" />
						</button>
						<span class="text-sm font-bold xm-text-primary tabular-nums">{pickerYear}</span>
						<button
							type="button"
							aria-label="Next year"
							class="flex h-8 w-8 items-center justify-center rounded-full xm-text-secondary transition-colors hover:bg-[var(--xm-surface-hover)]"
							onclick={() => (pickerYear += 1)}
						>
							<ChevronRight class="h-4 w-4" aria-hidden="true" />
						</button>
					</div>

					{#if indexed.years.length > 1}
						<div class="mb-2.5 flex flex-wrap gap-1">
							{#each indexed.years as yr (yr)}
								<button
									type="button"
									class="rounded-full px-2.5 py-0.5 text-[11px] font-semibold tabular-nums transition-colors {pickerYear === yr
										? 'bg-[var(--xm-accent)] text-white'
										: 'bg-[var(--xm-surface-subtle)] xm-text-secondary hover:bg-[var(--xm-surface-hover)]'}"
									onclick={() => (pickerYear = yr)}
								>
									{yr}
								</button>
							{/each}
						</div>
					{/if}

					<div class="grid grid-cols-3 gap-1.5">
						{#each shortMonths as mLabel, mIdx (mIdx)}
							{@const mCount = monthMemoCount(pickerYear, mIdx)}
							{@const isSelected = pickerYear === viewYear && mIdx === viewMonth}
							{@const isNowMonth =
								pickerYear === today.getFullYear() && mIdx === today.getMonth()}
							<button
								type="button"
								class="relative flex flex-col items-center justify-center rounded-xl py-2 text-xs font-medium transition-colors {isSelected
									? 'bg-[var(--xm-accent)] font-semibold text-white'
									: isNowMonth
										? 'bg-[var(--xm-accent-soft)] font-semibold xm-text-accent'
										: 'xm-text-primary hover:bg-[var(--xm-surface-hover)]'}"
								onclick={() => selectMonth(mIdx)}
							>
								<span>{mLabel}</span>
								{#if mCount > 0}
									<span
										class="mt-0.5 text-[10px] tabular-nums {isSelected
											? 'text-white/85'
											: 'xm-text-accent font-semibold'}"
									>
										• {mCount}
									</span>
								{/if}
							</button>
						{/each}
					</div>
				</div>
			{/if}
		</div>

		<div class="flex items-center gap-1">
			<button
				type="button"
				aria-label={i18n.t('heatmap.prevMonth')}
				class="flex h-8 w-8 items-center justify-center rounded-full xm-text-secondary transition-colors hover:bg-[var(--xm-surface-subtle)] hover:text-[var(--xm-text-primary)]"
				onclick={prevMonth}
			>
				<ChevronLeft class="h-4 w-4" aria-hidden="true" />
			</button>
			<button
				type="button"
				aria-label={i18n.t('heatmap.nextMonth')}
				class="flex h-8 w-8 items-center justify-center rounded-full xm-text-secondary transition-colors hover:bg-[var(--xm-surface-subtle)] hover:text-[var(--xm-text-primary)]"
				onclick={nextMonth}
			>
				<ChevronRight class="h-4 w-4" aria-hidden="true" />
			</button>
			<button
				type="button"
				aria-label={i18n.t('heatmap.jumpToday')}
				class="ml-0.5 rounded-full px-3 py-1 text-xs font-semibold transition-colors {isCurrentMonth &&
				selectedDate === todayKey
					? 'bg-[var(--xm-accent-soft)] xm-text-accent'
					: 'xm-text-secondary hover:bg-[var(--xm-surface-subtle)] hover:text-[var(--xm-text-primary)]'}"
				onclick={jumpToToday}
			>
				{i18n.t('heatmap.today')}
			</button>
		</div>
	</div>

	<!-- Weekday Header Axis (Sun..Sat) -->
	<div class="grid shrink-0 grid-cols-7 px-0.5" aria-hidden="true">
		{#each weekdayHeaders as wd, idx (idx)}
			<div class="px-1.5 text-[11px] font-medium tracking-wide xm-text-muted sm:px-2">
				{wd}
			</div>
		{/each}
	</div>

	<!-- Viewport-Locked Monthly Grid on md+ (zero vertical scrollbar on tablet/desktop), natural touch grid on mobile -->
	<div
		class="glass-surface flex flex-col overflow-hidden rounded-2xl border border-[var(--xm-divider)] md:min-h-0 md:flex-1"
		role="grid"
		aria-label={monthLabel}
	>
		<div
			class="xm-cal-grid grid grid-cols-7 gap-px bg-[var(--xm-divider)] md:min-h-0 md:flex-1"
			style="--xm-cal-weeks: {weekRows};"
		>
			{#each calendarCells as cell (cell.date)}
				{@const isSelected = selectedDate === cell.date}
				{@const hasMemos = cell.totalCount > 0}
				<button
					type="button"
					aria-label="{cell.date}: {cell.totalCount} {cell.totalCount === 1
						? i18n.t('heatmap.memoSingular')
						: i18n.t('heatmap.memoPlural')}"
					aria-pressed={isSelected}
					class="group relative flex min-h-[56px] flex-col items-start justify-between overflow-hidden p-1.5 text-left transition-colors focus-visible:z-10 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--xm-accent)] sm:p-2 md:min-h-0 {isSelected
						? 'bg-[var(--xm-accent)]/10 ring-1.5 ring-inset ring-[var(--xm-accent)]/75 dark:bg-[var(--xm-accent)]/14'
						: hasMemos && cell.inCurrentMonth
							? 'bg-[var(--xm-surface-subtle)] hover:bg-[var(--xm-surface-hover)]'
							: 'bg-white dark:bg-[#161618] hover:bg-[var(--xm-surface-subtle)]'}"
					onclick={() => handleDayClick(cell)}
				>
					<!-- Top Row: Day Number Pill -->
					<div class="flex w-full shrink-0 items-center justify-between gap-1">
						<span
							class="inline-flex h-5 min-w-[22px] items-center justify-center rounded-md px-1 text-xs tabular-nums transition-colors {cell.isToday
								? 'bg-[var(--xm-accent)] font-bold text-white dark:bg-white dark:text-black'
								: isSelected
									? 'bg-[var(--xm-accent)]/20 font-bold xm-text-accent dark:bg-[var(--xm-accent)]/30 dark:text-white'
									: cell.inCurrentMonth
										? hasMemos
											? 'font-semibold xm-text-primary'
											: 'font-medium xm-text-secondary'
										: 'font-normal xm-text-muted opacity-35'}"
						>
							{cell.day}
						</span>

						<!-- Compact Mobile Count Badge (< md) -->
						{#if hasMemos}
							<span
								class="inline-flex items-center rounded-full bg-[var(--xm-accent-soft)] px-1.5 py-0.5 text-[10px] font-semibold xm-text-accent tabular-nums md:hidden"
							>
								{cell.totalCount}
							</span>
						{/if}
					</div>

					<!-- Desktop / Tablet Rich Memo Previews (md+) -->
					{#if hasMemos}
						<div class="mt-1 hidden min-h-0 w-full flex-1 flex-col gap-0.5 overflow-hidden md:flex">
							{#each cell.previews as prev (prev.id)}
								<div
									class="flex items-start gap-1 text-[11px] leading-[1.3] {cell.inCurrentMonth
										? 'xm-text-secondary'
										: 'xm-text-muted opacity-55'}"
								>
									<span
										class="mt-0.5 flex h-3.5 w-3.5 shrink-0 items-center justify-center rounded-full bg-[var(--xm-accent-soft)] text-[8px] font-bold xm-text-accent"
										aria-hidden="true"
									>
										{prev.initial}
									</span>
									<span
										class="min-w-0 flex-1 {cell.previews.length === 1
											? 'line-clamp-2'
											: 'truncate'}"
									>
										{prev.excerpt}
									</span>
								</div>
							{/each}

							{#if cell.overflowCount > 0 || cell.thumbUrl}
								<div class="mt-auto flex items-center justify-between gap-1 pt-0.5">
									{#if cell.overflowCount > 0}
										<span class="pl-4 text-[10px] font-medium xm-text-muted tabular-nums">
											{i18n.t('calendar.moreCount', { count: cell.overflowCount })}
										</span>
									{:else}
										<span></span>
									{/if}
									{#if cell.thumbUrl}
										<img
											src={cell.thumbUrl}
											alt=""
											loading="lazy"
											decoding="async"
											class="h-5 w-5 shrink-0 rounded border border-[var(--xm-divider)] object-cover"
										/>
									{/if}
								</div>
							{/if}
						</div>

						<!-- Mobile Dot & Photo Indicator (< md) -->
						<div class="mt-1.5 flex w-full items-center gap-1 md:hidden">
							{#each Array.from({ length: Math.min(cell.totalCount, 3) }) as _, dotIdx (dotIdx)}
								<span class="h-1.5 w-1.5 rounded-full bg-[var(--xm-accent)]"></span>
							{/each}
							{#if cell.thumbUrl}
								<ImageIcon class="ml-auto h-3 w-3 xm-text-muted" aria-hidden="true" />
							{/if}
						</div>
					{/if}
				</button>
			{/each}
		</div>
	</div>
</section>
