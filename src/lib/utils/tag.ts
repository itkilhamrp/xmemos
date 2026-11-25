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

import type { TagCount } from '#lib/types/memo.ts';

export interface TagTreeNode {
	segment: string;
	fullPath: string;
	count: number;
	children: TagTreeNode[];
}

export function buildTagTree(tags: TagCount[]): TagTreeNode[] {
	const root: TagTreeNode[] = [];

	for (const { name, count } of tags) {
		const parts = name.split('/').filter(Boolean);
		let level = root;
		let pathSoFar = '';

		for (let i = 0; i < parts.length; i++) {
			const segment = parts[i]!;
			pathSoFar = pathSoFar ? `${pathSoFar}/${segment}` : segment;
			let node = level.find((n) => n.segment === segment);
			if (!node) {
				node = { segment, fullPath: pathSoFar, count: 0, children: [] };
				level.push(node);
			}
			node.count += count;
			level = node.children;
		}
	}

	const sortNodes = (nodes: TagTreeNode[]): void => {
		nodes.sort((a, b) => b.count - a.count || a.segment.localeCompare(b.segment));
		for (const n of nodes) sortNodes(n.children);
	};
	sortNodes(root);

	return root;
}
