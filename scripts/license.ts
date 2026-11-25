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

import { spawnSync } from 'node:child_process';
import { readdirSync, readFileSync, statSync, writeFileSync } from 'node:fs';
import { extname, join, relative, resolve } from 'node:path';

const rootDir = process.cwd();
const headerFile = resolve(rootDir, '.github/license-header.txt');
const isCheck = process.argv.includes('--check');

const IGNORE_GLOBS = [
	'node_modules/**',
	'.svelte-kit/**',
	'build/**',
	'dist/**',
	'src-tauri/target/**',
	'src-tauri/gen/**',
	'plugins/**/target/**',
	'plugins/**/build/**',
	'**/.gradle/**',
	'**/*.json',
	'**/*.md',
	'**/*.lock',
	'pnpm-lock.yaml',
	'pnpm-workspace.yaml',
	'**/*.toml',
	'**/*.svg',
	'**/*.html',
	'**/*.txt',
	'**/*.pro',
	'**/*.properties',
	'**/*.xml',
	'LICENSE'
];

const IGNORE_FILES = new Set(['pnpm-lock.yaml', 'pnpm-workspace.yaml', 'LICENSE']);

const IGNORE_DIRS = new Set([
	'node_modules',
	'.git',
	'.svelte-kit',
	'build',
	'dist',
	'target',
	'.gradle',
	'gen'
]);

const COMMENT_STYLE: Record<string, { prefix: string; open?: string; close?: string }> = {
	'.ts': { prefix: '// ' },
	'.js': { prefix: '// ' },
	'.rs': { prefix: '// ' },
	'.kt': { prefix: '// ' },
	'.kts': { prefix: '// ' },
	'.css': { open: '/*', prefix: ' * ', close: ' */' },
	'.svelte': { open: '<!--', prefix: '  ', close: '-->' },
	'.sh': { prefix: '# ' },
	'.yml': { prefix: '# ' },
	'.yaml': { prefix: '# ' }
};

function tryAddlicenseBinary(): boolean {
	const checkBin = spawnSync('addlicense', ['-h'], { encoding: 'utf8' });
	if (checkBin.error) {
		return false;
	}
	const args = ['-f', headerFile];
	if (isCheck) {
		args.push('-check');
	}
	for (const pattern of IGNORE_GLOBS) {
		args.push('-ignore', pattern);
	}
	args.push('.');

	const res = spawnSync('addlicense', args, { stdio: 'inherit', cwd: rootDir });
	if (res.status !== 0) {
		process.exit(res.status ?? 1);
	}
	return true;
}

function collectFiles(dir: string, acc: string[]): void {
	for (const entry of readdirSync(dir)) {
		if (IGNORE_DIRS.has(entry)) continue;
		const full = join(dir, entry);
		const st = statSync(full);
		if (st.isDirectory()) {
			collectFiles(full, acc);
		} else if (st.isFile()) {
			if (IGNORE_FILES.has(entry)) continue;
			const ext = extname(entry).toLowerCase();
			if (COMMENT_STYLE[ext]) {
				acc.push(full);
			}
		}
	}
}

function formatHeader(rawHeader: string, ext: string): string {
	const style = COMMENT_STYLE[ext]!;
	const lines = rawHeader.trimEnd().split('\n');
	const body = lines.map((line) => (line.length === 0 ? style.prefix.trimEnd() : `${style.prefix}${line}`));
	const parts: string[] = [];
	if (style.open) parts.push(style.open);
	parts.push(...body);
	if (style.close) parts.push(style.close);
	return parts.join('\n') + '\n\n';
}

function runFallback(): void {
	const rawHeader = readFileSync(headerFile, 'utf8');
	const files: string[] = [];
	collectFiles(rootDir, files);

	const missing: string[] = [];
	for (const file of files) {
		const content = readFileSync(file, 'utf8');
		if (content.includes('SPDX-License-Identifier: AGPL-3.0-or-later')) {
			continue;
		}
		const rel = relative(rootDir, file);
		if (isCheck) {
			missing.push(rel);
		} else {
			const ext = extname(file).toLowerCase();
			const headerBlock = formatHeader(rawHeader, ext);
			if (content.startsWith('#!')) {
				const newlineIdx = content.indexOf('\n');
				const shebang = newlineIdx === -1 ? content + '\n' : content.slice(0, newlineIdx + 1);
				const rest = newlineIdx === -1 ? '' : content.slice(newlineIdx + 1);
				writeFileSync(file, shebang + headerBlock + rest, 'utf8');
			} else {
				writeFileSync(file, headerBlock + content, 'utf8');
			}
			console.log(`Applied AGPLv3 header: ${rel}`);
		}
	}

	if (isCheck && missing.length > 0) {
		console.error('Files missing AGPL-3.0-or-later license header:');
		for (const m of missing) {
			console.error(`  - ${m}`);
		}
		process.exit(1);
	}
	console.log(`License ${isCheck ? 'check' : 'apply'} completed (${files.length} source files scanned).`);
}

if (!tryAddlicenseBinary()) {
	runFallback();
}
