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

import { describe, expect, it } from 'vitest';
import {
	handlePinKeydown,
	isValidSixDigitPin,
	PIN_PATTERN,
	sanitizePinInput
} from './pin';

describe('6-digit PIN sanitization & validation', () => {
	it('strips non-numeric characters and caps length at 6 digits', () => {
		expect(sanitizePinInput('12a3b4!5 6789')).toBe('123456');
		expect(sanitizePinInput('abcdef')).toBe('');
		expect(sanitizePinInput('007')).toBe('007');
		expect(sanitizePinInput('9876543210')).toBe('987654');
	});

	it('validates strictly 6 ASCII digits and exports a valid HTML pattern regex', () => {
		const htmlRegex = new RegExp(`^(?:${PIN_PATTERN})$`);
		expect(PIN_PATTERN).toBe('[0-9]{6}');
		expect(htmlRegex.test('123456')).toBe(true);
		expect(htmlRegex.test('000000')).toBe(true);
		expect(htmlRegex.test('16')).toBe(false);
		expect(isValidSixDigitPin('000000')).toBe(true);
		expect(isValidSixDigitPin('123456')).toBe(true);
		expect(isValidSixDigitPin('12345')).toBe(false);
		expect(isValidSixDigitPin('1234567')).toBe(false);
		expect(isValidSixDigitPin('12345a')).toBe(false);
		expect(isValidSixDigitPin(' 12345')).toBe(false);
		expect(isValidSixDigitPin('')).toBe(false);
	});

	it('allows digits, control keys, and Android/IME Unidentified keys while blocking letters', () => {
		const makeEvent = (key: string, value = '', isComposing = false) => {
			let prevented = false;
			const ev = {
				key,
				ctrlKey: false,
				metaKey: false,
				altKey: false,
				isComposing,
				currentTarget: { value, selectionStart: value.length, selectionEnd: value.length },
				preventDefault: () => {
					prevented = true;
				}
			} as unknown as KeyboardEvent;
			handlePinKeydown(ev);
			return prevented;
		};

		expect(makeEvent('5', '1234')).toBe(false);
		expect(makeEvent('5', '123456')).toBe(true);
		expect(makeEvent('a', '123')).toBe(true);
		expect(makeEvent('Backspace', '123456')).toBe(false);
		expect(makeEvent('Unidentified', '123')).toBe(false);
		expect(makeEvent('Process', '123', true)).toBe(false);
	});
});
