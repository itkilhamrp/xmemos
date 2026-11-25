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

export const PIN_LENGTH = 6;
export const PIN_PATTERN = '[0-9]{6}';

const ALLOWED_CONTROL_KEYS = new Set([
	'Backspace',
	'Delete',
	'Tab',
	'Enter',
	'Escape',
	'ArrowLeft',
	'ArrowRight',
	'ArrowUp',
	'ArrowDown',
	'Home',
	'End'
]);

/**
 * Strips all non-ASCII-digit characters and truncates to 6 digits max.
 * Works identically for typing, paste, autofill, and IME commits.
 */
export function sanitizePinInput(raw: string): string {
	return raw.replace(/[^0-9]/g, '').slice(0, PIN_LENGTH);
}

/**
 * Returns true if and only if the value is strictly 6 ASCII numeric digits (0–9).
 */
export function isValidSixDigitPin(pin: string): boolean {
	return /^[0-9]{6}$/.test(pin);
}

/**
 * Keydown guard for 6-digit numeric PIN inputs:
 * Allows navigation/editing keys and Ctrl/Cmd shortcuts (copy/paste/select-all/undo),
 * allows ASCII digits 0–9 only while under 6 digits (or when replacing a selection),
 * and blocks all letters, symbols, spaces, and non-digit characters.
 */
export function handlePinKeydown(event: KeyboardEvent): void {
	if (event.ctrlKey || event.metaKey || event.altKey) return;
	if (event.isComposing || event.key === 'Unidentified' || event.key === 'Process') return;
	if (ALLOWED_CONTROL_KEYS.has(event.key)) return;

	if (!/^[0-9]$/.test(event.key)) {
		event.preventDefault();
		return;
	}

	const input = event.currentTarget as HTMLInputElement | null;
	if (!input) return;
	const selectionLen = Math.abs((input.selectionEnd ?? 0) - (input.selectionStart ?? 0));
	if (input.value.length >= PIN_LENGTH && selectionLen === 0) {
		event.preventDefault();
	}
}
