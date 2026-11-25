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

import Prism from 'prismjs';

// Load PrismJS grammars in dependency-safe order (all bundled locally, zero network calls)
import 'prismjs/components/prism-markup-templating';
import 'prismjs/components/prism-c';
import 'prismjs/components/prism-cpp';
import 'prismjs/components/prism-csharp';
import 'prismjs/components/prism-java';
import 'prismjs/components/prism-kotlin';
import 'prismjs/components/prism-swift';
import 'prismjs/components/prism-dart';
import 'prismjs/components/prism-go';
import 'prismjs/components/prism-rust';
import 'prismjs/components/prism-typescript';
import 'prismjs/components/prism-jsx';
import 'prismjs/components/prism-tsx';
import 'prismjs/components/prism-scss';
import 'prismjs/components/prism-python';
import 'prismjs/components/prism-ruby';
import 'prismjs/components/prism-php';
import 'prismjs/components/prism-bash';
import 'prismjs/components/prism-shell-session';
import 'prismjs/components/prism-powershell';
import 'prismjs/components/prism-docker';
import 'prismjs/components/prism-json';
import 'prismjs/components/prism-yaml';
import 'prismjs/components/prism-toml';
import 'prismjs/components/prism-sql';
import 'prismjs/components/prism-ini';
import 'prismjs/components/prism-markdown';
import 'prismjs/components/prism-diff';

// Enhance bash grammar so modern developer CLI tools (pnpm, cargo, docker, kubectl, etc.) highlight cleanly
if (Prism.languages.bash) {
	Prism.languages.insertBefore('bash', 'function', {
		'cli-tool': {
			pattern:
				/(^|[\s;|&])(?:git|pnpm|npm|npx|yarn|bun|cargo|rustc|tauri|docker|kubectl|helm|terraform|gh|curl|wget|ssh|scp|rsync|systemctl|journalctl|apt|apt-get|dnf|pacman|brew|pip|pip3|uv|python|python3|node|deno|go|make|cmake)(?=$|[\s;|&])/,
			lookbehind: true,
			alias: 'function'
		}
	});
}

const LANGUAGE_ALIASES: Record<string, string> = {
	sh: 'bash',
	shell: 'bash',
	zsh: 'bash',
	console: 'shell-session',
	py: 'python',
	py3: 'python',
	js: 'javascript',
	mjs: 'javascript',
	cjs: 'javascript',
	ts: 'typescript',
	mts: 'typescript',
	rs: 'rust',
	golang: 'go',
	rb: 'ruby',
	kt: 'kotlin',
	kts: 'kotlin',
	cs: 'csharp',
	'c#': 'csharp',
	'c++': 'cpp',
	cc: 'cpp',
	cxx: 'cpp',
	hpp: 'cpp',
	h: 'c',
	yml: 'yaml',
	dockerfile: 'docker',
	ps: 'powershell',
	ps1: 'powershell',
	pwsh: 'powershell',
	md: 'markdown',
	html: 'markup',
	xml: 'markup',
	svg: 'markup',
	svelte: 'markup',
	vue: 'markup',
	patch: 'diff',
	conf: 'ini',
	cfg: 'ini',
	env: 'ini',
	sqlite: 'sql',
	postgres: 'sql',
	postgresql: 'sql',
	mysql: 'sql',
	plaintext: 'text',
	txt: 'text',
	plain: 'text'
};

export function escapeHtml(raw: string): string {
	return raw
		.replace(/&/g, '&amp;')
		.replace(/</g, '&lt;')
		.replace(/>/g, '&gt;')
		.replace(/"/g, '&quot;');
}

export function normalizeLanguage(rawLang?: string | null): string {
	const cleaned = (rawLang ?? '').trim().toLowerCase().replace(/^language-/, '');
	if (!cleaned) return '';
	return LANGUAGE_ALIASES[cleaned] ?? cleaned;
}

/**
 * Intelligently infers programming language for unlabeled fenced code blocks.
 * Runs in O(n) with lightweight regex heuristics and zero heavy dependencies.
 */
export function detectCodeLanguage(code: string): string {
	const trimmed = code.trim();
	if (!trimmed) return 'text';

	// 1. Shebang detection
	const firstLine = trimmed.split('\n', 1)[0] ?? '';
	if (firstLine.startsWith('#!')) {
		if (/python/.test(firstLine)) return 'python';
		if (/node|deno|bun/.test(firstLine)) return 'javascript';
		if (/ruby/.test(firstLine)) return 'ruby';
		if (/php/.test(firstLine)) return 'php';
		if (/bash|sh|zsh/.test(firstLine)) return 'bash';
	}

	// 2. Diff / Patch
	if (/^(?:diff --git |@@ -\d+,\d+ \+\d+,\d+ @@|--- a\/|\+\+\+ b\/)/m.test(trimmed)) {
		return 'diff';
	}

	// 3. JSON
	if (
		(trimmed.startsWith('{') && trimmed.endsWith('}')) ||
		(trimmed.startsWith('[') && trimmed.endsWith(']'))
	) {
		try {
			JSON.parse(trimmed);
			return 'json';
		} catch {
			// Not strict JSON — continue heuristic checks
		}
	}

	// 4. HTML / XML / Svelte
	if (/^<!doctype\s+html/i.test(trimmed) || /^<(?:html|head|body|div|section|main|script|style|svg)\b/i.test(trimmed)) {
		return 'markup';
	}

	// 5. Dockerfile
	if (/^(?:FROM|RUN|CMD|COPY|ADD|WORKDIR|ENV|EXPOSE|ENTRYPOINT)\s+/m.test(trimmed) && /^FROM\s+/m.test(trimmed)) {
		return 'docker';
	}

	// 6. SQL
	if (
		/\b(?:SELECT\s+.+\s+FROM|INSERT\s+INTO|UPDATE\s+\w+\s+SET|DELETE\s+FROM|CREATE\s+TABLE|ALTER\s+TABLE|PRAGMA\s+\w+)\b/i.test(
			trimmed
		)
	) {
		return 'sql';
	}

	// 7. Rust
	if (
		/\b(?:pub\s+fn|fn\s+\w+\s*\(|let\s+mut\s+|impl\s+|match\s+\w+|println!|use\s+\w+::|Result<|Option<)\b/.test(
			trimmed
		)
	) {
		return 'rust';
	}

	// 8. Go
	if (/\b(?:package\s+\w+|func\s+(?:\([^)]+\)\s*)?\w+\(|fmt\.Print|:=)\b/.test(trimmed)) {
		return 'go';
	}

	// 9. Python
	if (
		/(?:^\s*def\s+\w+\s*\(|^\s*class\s+\w+.*:|^\s*from\s+[\w.]+\s+import\s+|^\s*elif\s+|__name__\s*==\s*['"]__main__['"])/m.test(
			trimmed
		)
	) {
		return 'python';
	}

	// 10. C / C++
	if (/^\s*#include\s*[<"]/m.test(trimmed)) {
		return /\b(?:std::|cout|cin|vector<|namespace\s+|template\s*<|class\s+)\b/.test(trimmed)
			? 'cpp'
			: 'c';
	}

	// 11. PHP
	if (/^<\?php\b/i.test(trimmed)) return 'php';

	// 12. Java / Kotlin
	if (/\b(?:public\s+class|private\s+static\s+final|System\.out\.println)\b/.test(trimmed)) {
		return 'java';
	}
	if (/\b(?:fun\s+\w+\s*\(|val\s+\w+\s*[:=]|data\s+class)\b/.test(trimmed)) {
		return 'kotlin';
	}

	// 13. TypeScript / JavaScript
	if (
		/\b(?:interface\s+\w+|type\s+\w+\s*=|:\s*(?:string|number|boolean|void|Promise<)|\$state<|\$derived)\b/.test(
			trimmed
		)
	) {
		return 'typescript';
	}
	if (
		/\b(?:const|let|var)\s+\w+\s*=|\bfunction\s+\w+\s*\(|=>\s*\{|import\s+.+\s+from\s+['"]|console\.log\(/.test(
			trimmed
		)
	) {
		return 'javascript';
	}

	// 14. CSS / SCSS
	if (/(?:^|\n)\s*(?:@media|@keyframes|:root|\.[a-z_-][\w-]*\s*\{|#[a-z_-][\w-]*\s*\{)/i.test(trimmed)) {
		return 'css';
	}

	// 15. Bash / Shell commands
	if (
		/(?:^|\n)\s*(?:\$\s+)?(?:git|pnpm|npm|npx|yarn|bun|cargo|docker|kubectl|sudo|curl|wget|apt|apt-get|brew|cd|ls|mkdir|rm|cp|mv|chmod|chown|export|echo|grep|sed|awk|cat|ssh|systemctl)\s+/m.test(
			trimmed
		)
	) {
		return 'bash';
	}

	// 16. TOML
	if (/^\[[\w.-]+\]\s*$/m.test(trimmed) && /^\s*[\w.-]+\s*=\s*(?:"|'|true|false|\d|\[)/m.test(trimmed)) {
		return 'toml';
	}

	// 17. YAML
	if (/^(?:[a-zA-Z_][\w.-]*:\s+.+\n?){2,}$/m.test(trimmed) && !/[{};]/.test(trimmed)) {
		return 'yaml';
	}

	return 'text';
}

/**
 * Highlights comments cleanly even for plain/unknown languages so notes inside code blocks
 * remain visually distinct.
 */
function highlightGenericComments(code: string): string {
	const escaped = escapeHtml(code);
	return escaped.replace(
		/(\/\*[\s\S]*?\*\/|&lt;!--[\s\S]*?--&gt;|(?:^|(?<=\s))\/\/[^\n]*|(?:^|(?<=\s))--[^\n]*|(?:^|(?<=\s))#[^\n]*)/gm,
		'<span class="token comment">$1</span>'
	);
}

export interface HighlightedCodeResult {
	language: string;
	displayLabel: string;
	html: string;
}

export function highlightCodeBlock(code: string, rawLang?: string | null): HighlightedCodeResult {
	const explicit = normalizeLanguage(rawLang);
	const language = explicit || detectCodeLanguage(code);
	const displayLabel = language === 'markup' && rawLang?.trim() ? rawLang.trim().toLowerCase() : language;

	const grammar = language && language !== 'text' ? Prism.languages[language] : null;
	if (grammar) {
		return {
			language,
			displayLabel,
			html: Prism.highlight(code, grammar, language)
		};
	}

	return {
		language: 'text',
		displayLabel: explicit || 'text',
		html: highlightGenericComments(code)
	};
}
