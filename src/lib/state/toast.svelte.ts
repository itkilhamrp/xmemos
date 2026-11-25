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

export type ToastKind = 'info' | 'success' | 'error';

class ToastState {
	opened = $state(false);
	message = $state('');
	kind = $state<ToastKind>('info');
	private timer: ReturnType<typeof setTimeout> | null = null;

	show(message: string, kind: ToastKind = 'info', durationMs = 2600): void {
		this.message = message;
		this.kind = kind;
		this.opened = true;
		if (this.timer) clearTimeout(this.timer);
		this.timer = setTimeout(() => {
			this.opened = false;
		}, durationMs);
	}

	dismiss(): void {
		this.opened = false;
		if (this.timer) clearTimeout(this.timer);
	}
}

export const toastState = new ToastState();
