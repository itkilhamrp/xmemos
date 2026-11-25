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

export function handleError({
	error,
	message
}: {
	error: unknown;
	message: string;
}): { message: string } {
	const detail =
		error instanceof Error
			? error.message || message
			: typeof error === 'string' && error.trim()
				? error
				: message;
	console.error('[Xmemos client error]', error);
	return {
		message: detail || 'An unexpected application error occurred.'
	};
}
