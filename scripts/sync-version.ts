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

function run(): void {
	const pkgPath = resolve(rootDir, 'package.json');
	const tauriConfPath = resolve(rootDir, 'src-tauri/tauri.conf.json');
	const cargoTomlPath = resolve(rootDir, 'src-tauri/Cargo.toml');
	const gradlePropsPath = resolve(rootDir, 'src-tauri/gen/android/app/tauri.properties');

	const pkg = JSON.parse(readFileSync(pkgPath, 'utf8')) as PackageJson;
	const appVersion = pkg.version;
	const versionCode = computeVersionCode(appVersion);
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

	if (errors.length > 0) {
		console.error('Version check failed:\n' + errors.map((e) => `  - ${e}`).join('\n'));
		process.exit(1);
	}

	console.log(`Version check OK: v${appVersion} (Android versionCode=${versionCode})`);
}

run();
