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
	import { toastState } from '#lib/state/toast.svelte.ts';
	import { highlightHtmlMatches, renderMemoMarkdown } from '#lib/utils/markdown.ts';

	interface Props {
		content: string;
		canToggleTask?: boolean;
		highlightTerms?: string[];
		onToggleTask?: (taskIndex: number) => void;
		onSelectTag?: (tag: string) => void;
	}

	let {
		content,
		canToggleTask = false,
		highlightTerms = [],
		onToggleTask,
		onSelectTag
	}: Props = $props();

	let renderedHtml = $derived.by(() => {
		const base = renderMemoMarkdown(content);
		return highlightTerms.length > 0 ? highlightHtmlMatches(base, highlightTerms) : base;
	});

	function handleClick(event: MouseEvent): void {
		const target = event.target as HTMLElement | null;
		if (!target) return;

		const copyBtn = target.closest<HTMLButtonElement>('[data-copy-code]');
		if (copyBtn) {
			event.preventDefault();
			event.stopPropagation();
			const codeEl = copyBtn.closest('.xm-code-block')?.querySelector('pre code');
			const rawCode = codeEl?.textContent ?? '';
			if (rawCode) {
				void navigator.clipboard.writeText(rawCode).then(() => {
					toastState.show(i18n.t('memo.copiedCode'), 'success');
				});
			}
			return;
		}

		if (target instanceof HTMLInputElement && target.dataset.taskIndex !== undefined) {
			event.stopPropagation();
			if (!canToggleTask) {
				event.preventDefault();
				return;
			}
			const idx = Number(target.dataset.taskIndex);
			if (!Number.isNaN(idx) && onToggleTask) {
				onToggleTask(idx);
			}
			return;
		}

		const tagBtn = target.closest<HTMLElement>('[data-tag]');
		if (tagBtn?.dataset.tag && onSelectTag) {
			event.preventDefault();
			event.stopPropagation();
			onSelectTag(tagBtn.dataset.tag);
		}
	}
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="memo-prose text-[15px]" onclick={handleClick}>
	{@html renderedHtml}
</div>
