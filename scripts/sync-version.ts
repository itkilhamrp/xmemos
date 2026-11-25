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

import { readFileSync, writeFileSync, existsSync } from 'node:fs';
import { resolve } from 'node:path';

interface PackageJson {
	version: string;
}

interface TauriConf {
	version: string;
}

const CHANGELOG_DETAILS_URL = 'https://github.com/xmemos/xmemos/blob/main/CHANGELOG.md';
const rootDir = process.cwd();
const mode = process.argv.includes('--sync') ? 'sync' : 'check';

function parseSemVer(version: string): { major: number; minor: number; patch: number } {
	const match = /^(\d+)\.(\d+)\.(\d+)(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$/.exec(version.trim());
	if (!match) {
		throw new Error(`Invalid SemVer in package.json: "${version}"`);
	}
	return {
		major: Number(match[1]),
		minor: Number(match[2]),
		patch: Number(match[3])
	};
}

export function computeVersionCode(version: string): number {
	const { major, minor, patch } = parseSemVer(version);
	if (minor >= 1000 || patch >= 1000) {
		throw new Error(`SemVer components minor/patch must be < 1000 for deterministic versionCode: ${version}`);
	}
	return major * 1_000_000 + minor * 1_000 + patch;
}

function escapeRegExp(value: string): string {
	return value.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

function resolveReleaseDateFromChangelog(changelogPath: string, appVersion: string): string {
	if (existsSync(changelogPath)) {
		const changelog = readFileSync(changelogPath, 'utf8');
		const exactPattern = new RegExp(
			`^##\\s+\\[${escapeRegExp(appVersion)}\\]\\s+-\\s+(\\d{4}-\\d{2}-\\d{2})`,
			'm'
		);
		const exactMatch = exactPattern.exec(changelog);
		if (exactMatch?.[1]) {
			return exactMatch[1];
		}

		const latestMatch = /^##\s+\[\d+\.\d+\.\d+[^\]]*\]\s+-\s+(\d{4}-\d{2}-\d{2})/m.exec(changelog);
		if (latestMatch?.[1]) {
			return latestMatch[1];
		}
	}
	return new Date().toISOString().slice(0, 10);
}

function buildExpectedMetainfoReleasesBlock(appVersion: string, releaseDate: string): string {
	return [
		'  <releases>',
		'    <!-- Auto-synced from package.json and CHANGELOG.md via `pnpm version:sync` -->',
		`    <release version="${appVersion}" date="${releaseDate}">`,
		`      <url type="details">${CHANGELOG_DETAILS_URL}</url>`,
		'    </release>',
		'  </releases>'
	].join('\n');
}

function run(): void {
	const pkgPath = resolve(rootDir, 'package.json');
	const tauriConfPath = resolve(rootDir, 'src-tauri/tauri.conf.json');
	const cargoTomlPath = resolve(rootDir, 'src-tauri/Cargo.toml');
	const cargoLockPath = resolve(rootDir, 'src-tauri/Cargo.lock');
	const gradlePropsPath = resolve(rootDir, 'src-tauri/gen/android/app/tauri.properties');
	const changelogPath = resolve(rootDir, 'CHANGELOG.md');
	const metainfoPath = resolve(rootDir, 'packaging/flatpak/org.xmemos.app.metainfo.xml');
	const aboutPagePath = resolve(rootDir, 'src/routes/settings/about/+page.svelte');

	const pkg = JSON.parse(readFileSync(pkgPath, 'utf8')) as PackageJson;
	const appVersion = pkg.version;
	const versionCode = computeVersionCode(appVersion);
	const releaseDate = resolveReleaseDateFromChangelog(changelogPath, appVersion);
	const errors: string[] = [];

	// 1. Validate src-tauri/tauri.conf.json points to ../package.json
	const tauriConf = JSON.parse(readFileSync(tauriConfPath, 'utf8')) as TauriConf;
	if (tauriConf.version !== '../package.json') {
		errors.push(
			`src-tauri/tauri.conf.json "version" must be "../package.json" (found "${tauriConf.version}").`
		);
	}

	// 2. Validate or sync src-tauri/Cargo.toml package version
	const cargoContent = readFileSync(cargoTomlPath, 'utf8');
	const cargoVersionMatch = /^version\s*=\s*"([^"]+)"/m.exec(cargoContent);
	if (!cargoVersionMatch) {
		errors.push('Could not find package version in src-tauri/Cargo.toml.');
	} else if (cargoVersionMatch[1] !== appVersion) {
		if (mode === 'sync') {
			const updated = cargoContent.replace(/^version\s*=\s*"([^"]+)"/m, `version = "${appVersion}"`);
			writeFileSync(cargoTomlPath, updated, 'utf8');
			console.log(`Synced src-tauri/Cargo.toml version -> ${appVersion}`);
		} else {
			errors.push(
				`src-tauri/Cargo.toml version "${cargoVersionMatch[1]}" != package.json "${appVersion}". Run "pnpm version:sync".`
			);
		}
	}

	// 2b. Validate or sync src-tauri/Cargo.lock root package version (required by `cargo --locked`)
	if (existsSync(cargoLockPath)) {
		const lockContent = readFileSync(cargoLockPath, 'utf8');
		const lockRegex = /(\[\[package\]\]\r?\nname = "xmemos"\r?\nversion = )"([^"]+)"/;
		const lockMatch = lockRegex.exec(lockContent);
		if (!lockMatch) {
			errors.push('Could not find "xmemos" package version in src-tauri/Cargo.lock.');
		} else if (lockMatch[2] !== appVersion) {
			if (mode === 'sync') {
				const updated = lockContent.replace(lockRegex, `$1"${appVersion}"`);
				writeFileSync(cargoLockPath, updated, 'utf8');
				console.log(`Synced src-tauri/Cargo.lock xmemos version -> ${appVersion}`);
			} else {
				errors.push(
					`src-tauri/Cargo.lock xmemos version "${lockMatch[2]}" != package.json "${appVersion}". Run "pnpm version:sync".`
				);
			}
		}
	}

	// 3. Validate or sync Android tauri.properties if generated
	if (existsSync(gradlePropsPath)) {
		const propsContent = readFileSync(gradlePropsPath, 'utf8');
		const expectedProps = `tauri.android.versionCode=${versionCode}\ntauri.android.versionName=${appVersion}\n`;
		if (
			!propsContent.includes(`tauri.android.versionCode=${versionCode}`) ||
			!propsContent.includes(`tauri.android.versionName=${appVersion}`)
		) {
			if (mode === 'sync') {
				writeFileSync(gradlePropsPath, expectedProps, 'utf8');
				console.log(`Synced Android tauri.properties -> ${appVersion} (${versionCode})`);
			} else {
				errors.push(
					`src-tauri/gen/android/app/tauri.properties is out of sync with ${appVersion} (${versionCode}).`
				);
			}
		}
	}

	// 4. Validate or sync Flatpak AppStream metainfo.xml <releases> from package.json + CHANGELOG.md
	if (existsSync(metainfoPath)) {
		const metainfoContent = readFileSync(metainfoPath, 'utf8');
		const releasesRegex = /[ \t]*<releases>[\s\S]*?<\/releases>/;
		const releasesMatch = releasesRegex.exec(metainfoContent);
		const expectedBlock = buildExpectedMetainfoReleasesBlock(appVersion, releaseDate);

		if (!releasesMatch) {
			errors.push('Could not find <releases> block in packaging/flatpak/org.xmemos.app.metainfo.xml.');
		} else if (releasesMatch[0] !== expectedBlock) {
			if (mode === 'sync') {
				const updated = metainfoContent.replace(releasesRegex, expectedBlock);
				writeFileSync(metainfoPath, updated, 'utf8');
				console.log(
					`Synced packaging/flatpak/org.xmemos.app.metainfo.xml -> ${appVersion} (${releaseDate})`
				);
			} else {
				errors.push(
					`packaging/flatpak/org.xmemos.app.metainfo.xml <releases> is out of sync with package.json "${appVersion}" (${releaseDate}) or contains manual changelog text. Run "pnpm version:sync".`
				);
			}
		}
	}

	// 5. Validate or sync About page browser-preview fallback version
	if (existsSync(aboutPagePath)) {
		const aboutContent = readFileSync(aboutPagePath, 'utf8');
		const aboutVersionRegex = /info\s*\?\s*`v\$\{info\.appVersion\}`\s*:\s*'v([^']+)'/;
		const aboutMatch = aboutVersionRegex.exec(aboutContent);
		if (aboutMatch && aboutMatch[1] !== appVersion) {
			if (mode === 'sync') {
				const updated = aboutContent.replace(
					aboutVersionRegex,
					`info ? \`v\${info.appVersion}\` : 'v${appVersion}'`
				);
				writeFileSync(aboutPagePath, updated, 'utf8');
				console.log(`Synced src/routes/settings/about/+page.svelte fallback -> v${appVersion}`);
			} else {
				errors.push(
					`src/routes/settings/about/+page.svelte fallback "v${aboutMatch[1]}" != package.json "v${appVersion}". Run "pnpm version:sync".`
				);
			}
		}
	}

	if (errors.length > 0) {
		console.error('Version check failed:\n' + errors.map((e) => `  - ${e}`).join('\n'));
		process.exit(1);
	}

	console.log(
		`Version check OK: v${appVersion} (${releaseDate}, Android versionCode=${versionCode})`
	);
}

run();
