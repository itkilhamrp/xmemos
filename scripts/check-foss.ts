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

import { readdirSync, readFileSync, statSync } from 'node:fs';
import { extname, join, relative } from 'node:path';

const rootDir = process.cwd();

const FORBIDDEN_PATTERNS: { label: string; regex: RegExp }[] = [
	{ label: 'Google Mobile Services (GMS)', regex: /com\.google\.android\.gms/i },
	{ label: 'Firebase SDK', regex: /com\.google\.firebase/i },
	{ label: 'Google Play Services', regex: /play-services-/i },
	{ label: 'Google Play Core', regex: /com\.google\.android\.play/i },
	{ label: 'Google ML Kit', regex: /com\.google\.mlkit/i },
	{ label: 'AdMob / Ads SDK', regex: /googleads|admob/i }
];

const SCAN_EXTENSIONS = new Set([
	'.ts',
	'.svelte',
	'.rs',
	'.kt',
	'.kts',
	'.toml',
	'.json',
	'.xml',
	'.gradle'
]);

const BINARY_EXTENSIONS = new Set(['.so', '.jar', '.aar', '.apk', '.exe', '.dll', '.dylib']);

const IGNORE_DIRS = new Set([
	'node_modules',
	'.git',
	'.svelte-kit',
	'build',
	'dist',
	'target',
	'.gradle',
	'scripts'
]);

function walk(dir: string, acc: string[]): void {
	for (const entry of readdirSync(dir)) {
		if (IGNORE_DIRS.has(entry)) continue;
		const full = join(dir, entry);
		const st = statSync(full);
		if (st.isDirectory()) {
			walk(full, acc);
		} else if (st.isFile()) {
			const ext = extname(entry).toLowerCase();
			if (BINARY_EXTENSIONS.has(ext)) {
				throw new Error(`Forbidden prebuilt binary in source tree: ${relative(rootDir, full)}`);
			}
			if (SCAN_EXTENSIONS.has(ext))
			acc.push(full);
		}
	}
}

function run(): void {
	const files: string[] = [];
	walk(rootDir, files);
	const violations: string[] = [];

	for (const file of files) {
		const content = readFileSync(file, 'utf8');
		for (const { label, regex } of FORBIDDEN_PATTERNS) {
			if (regex.test(content)) {
				violations.push(`${relative(rootDir, file)}: matched forbidden ${label}`);
			}
		}
	}

	if (violations.length > 0) {
		console.error('FOSS / GMS-free check FAILED:\n' + violations.map((v) => `  - ${v}`).join('\n'));
		process.exit(1);
	}

	console.log(`FOSS / GMS-free check OK: 0 proprietary/GMS references across ${files.length} files.`);
}

run();
