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

import { convertFileSrc, invoke, isTauri } from '@tauri-apps/api/core';

export interface IpcErrorPayload {
	code: string;
	message: string;
}

export class XmemosError extends Error {
	readonly code: string;

	constructor(code: string, message: string) {
		super(message);
		this.name = 'XmemosError';
		this.code = code;
	}
}

export async function callIpc<T>(command: string, args?: Record<string, unknown>): Promise<T> {
	try {
		return await invoke<T>(command, args);
	} catch (err: unknown) {
		if (typeof err === 'object' && err !== null && 'code' in err && 'message' in err) {
			const e = err as IpcErrorPayload;
			throw new XmemosError(String(e.code), String(e.message));
		}
		if (typeof err === 'string') {
			throw new XmemosError('IPC_ERROR', err);
		}
		throw new XmemosError('UNKNOWN_ERROR', err instanceof Error ? err.message : String(err));
	}
}

/**
 * Builds the permission-checked custom Tauri v2 protocol URL for a resource attachment.
 */
export function getMediaUrl(uid: string, thumb = false): string {
	const base = isTauri()
		? convertFileSrc(uid, 'xmemos-media')
		: `xmemos-media://localhost/${encodeURIComponent(uid)}`;
	return thumb ? `${base}?thumb=1` : base;
}
