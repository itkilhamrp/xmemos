// Copyright (C) 2026 Xmemos Contributors
//
// This file is part of Xmemos.
//
// Xmemos is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Xmemos is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with Xmemos. If not, see https://www.gnu.org/licenses/.
//
// SPDX-License-Identifier: AGPL-3.0-or-later

import { callIpc } from './invoke';
import type { HeatmapDay, MemoDraft, MemoLocation, MemoView, MemoVisibility, TagCount } from '#lib/types/memo.ts';
import type { Reaction } from '#lib/types/reaction.ts';

export interface ListMemosParams {
	creatorId?: number | null;
	rowStatus?: 'NORMAL' | 'ARCHIVED';
	visibilityList?: MemoVisibility[];
	tag?: string | null;
	query?: string | null;
	hasTaskList?: boolean;
	hasLink?: boolean;
	hasCode?: boolean;
	hasMedia?: boolean;
	startTs?: number | null;
	endTs?: number | null;
	orderBy?: 'created_ts' | 'updated_ts';
	orderDirection?: 'desc' | 'asc';
	limit?: number;
	offset?: number;
}

export const memoApi = {
	create: (payload: {
		content: string;
		visibility: MemoVisibility;
		pinned?: boolean;
		createdTs?: number | null;
		location?: MemoLocation | null;
		resourceIds?: number[];
		relationIds?: number[];
		parentMemoId?: number | null;
	}): Promise<MemoView> => callIpc<MemoView>('memo_create', { payload }),

	list: (payload: ListMemosParams = {}): Promise<MemoView[]> =>
		callIpc<MemoView[]>('memo_list', { payload }),

	getByUid: (uid: string): Promise<MemoView> => callIpc<MemoView>('memo_get_by_uid', { uid }),

	listComments: (memoId: number): Promise<MemoView[]> =>
		callIpc<MemoView[]>('memo_list_comments', { memoId }),

	update: (payload: {
		id: number;
		content?: string;
		visibility?: MemoVisibility;
		pinned?: boolean;
		rowStatus?: 'NORMAL' | 'ARCHIVED';
		createdTs?: number;
		location?: MemoLocation | null;
		clearLocation?: boolean;
		resourceIds?: number[];
	}): Promise<MemoView> => callIpc<MemoView>('memo_update', { payload }),

	toggleTask: (memoId: number, taskIndex: number): Promise<MemoView> =>
		callIpc<MemoView>('memo_toggle_task', { memoId, taskIndex }),

	delete: (memoId: number): Promise<void> => callIpc<void>('memo_delete', { memoId }),

	toggleReaction: (memoUid: string, reactionType: string): Promise<Reaction[]> =>
		callIpc<Reaction[]>('memo_toggle_reaction', { memoUid, reactionType }),

	listTags: (creatorId?: number | null): Promise<TagCount[]> =>
		callIpc<TagCount[]>('memo_list_tags', { creatorId: creatorId ?? null }),

	renameTag: (oldTag: string, newTag: string): Promise<number> =>
		callIpc<number>('memo_rename_tag', { oldTag, newTag }),

	deleteTag: (tag: string): Promise<number> => callIpc<number>('memo_delete_tag', { tag }),

	getHeatmap: (creatorId?: number | null, days = 0): Promise<HeatmapDay[]> =>
		callIpc<HeatmapDay[]>('memo_get_heatmap', { creatorId: creatorId ?? null, days }),

	saveDraft: (
		content: string,
		visibility: MemoVisibility,
		resourceIds: number[],
		relationIds: number[]
	): Promise<void> =>
		callIpc<void>('memo_save_draft', { content, visibility, resourceIds, relationIds }),

	loadDraft: (): Promise<MemoDraft | null> => callIpc<MemoDraft | null>('memo_load_draft')
};
