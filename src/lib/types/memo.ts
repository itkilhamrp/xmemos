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

import type { User, RowStatus } from './user';
import type { Resource } from './resource';
import type { MemoRelation } from './relation';
import type { Reaction } from './reaction';

export type MemoVisibility = 'PRIVATE' | 'PROTECTED' | 'PUBLIC';

export interface MemoProperty {
	hasLink: boolean;
	hasTaskList: boolean;
	hasCode: boolean;
	hasIncompleteTasks: boolean;
}

export interface MemoLocation {
	placeholder: string;
	latitude: number;
	longitude: number;
}

export interface MemoPayload {
	tags: string[];
	property: MemoProperty;
	location?: MemoLocation | null;
}

export interface Memo {
	id: number;
	uid: string;
	creatorId: number;
	createdTs: number;
	updatedTs: number;
	rowStatus: RowStatus;
	content: string;
	visibility: MemoVisibility;
	pinned: boolean;
	payload: MemoPayload;
}

export interface MemoView {
	memo: Memo;
	creator: User | null;
	resources: Resource[];
	relations: MemoRelation[];
	reactions: Reaction[];
	commentCount: number;
}

export interface TagCount {
	name: string;
	count: number;
}

export interface HeatmapDay {
	date: string;
	count: number;
}

export interface MemoDraft {
	content: string;
	visibility: MemoVisibility;
	resourceIds: number[];
	relationIds: number[];
	updatedTs: number;
}
