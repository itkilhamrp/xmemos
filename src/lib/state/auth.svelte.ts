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

import { authApi } from '#lib/api/auth.ts';
import type { User, UserLockConfig } from '#lib/types/user.ts';
import { supportsAndroidBiometric } from '#lib/utils/platform.ts';

const AUTH_SNAPSHOT_KEY = 'xmemos.auth_snapshot.v1';
const MEMO_SNAPSHOT_PREFIX = 'xmemos.memo_snapshot.v1.';

interface AuthSnapshot {
	hasHost: boolean;
	disallowUserRegistration: boolean;
	currentUser: User | null;
	requirePasswordAfterSignOut: boolean;
}

function clearAllCachedSnapshots(): void {
	if (typeof window === 'undefined') return;
	try {
		localStorage.removeItem(AUTH_SNAPSHOT_KEY);
		const keysToRemove: string[] = [];
		for (let i = 0; i < localStorage.length; i++) {
			const k = localStorage.key(i);
			if (k && k.startsWith(MEMO_SNAPSHOT_PREFIX)) {
				keysToRemove.push(k);
			}
		}
		for (const k of keysToRemove) {
			localStorage.removeItem(k);
		}
	} catch {
		// Ignore storage access errors
	}
}

class AuthState {
	initialized = $state(false);
	hasHost = $state(false);
	disallowUserRegistration = $state(false);
	currentUser = $state<User | null>(null);
	lockConfig = $state<UserLockConfig | null>(null);
	localUsers = $state<User[]>([]);
	requirePasswordAfterSignOut = $state(false);
	isLocked = $state(false);
	private initialLockChecked = false;

	isAuthenticated = $derived(this.currentUser !== null);
	isHost = $derived(this.currentUser?.role === 'HOST');
	isAdminOrHost = $derived(
		this.currentUser?.role === 'HOST' || this.currentUser?.role === 'ADMIN'
	);

	/**
	 * Synchronously hydrates last verified session summary from localStorage on frame 0
	 * so the UI shell and timeline render immediately while Tauri IPC revalidates in parallel.
	 */
	hydrateFromCache(): void {
		if (typeof window === 'undefined' || this.initialized) return;
		try {
			const raw = localStorage.getItem(AUTH_SNAPSHOT_KEY);
			if (!raw) return;
			const snap = JSON.parse(raw) as Partial<AuthSnapshot>;
			if (typeof snap.hasHost === 'boolean') this.hasHost = snap.hasHost;
			if (typeof snap.disallowUserRegistration === 'boolean') {
				this.disallowUserRegistration = snap.disallowUserRegistration;
			}
			if (typeof snap.requirePasswordAfterSignOut === 'boolean') {
				this.requirePasswordAfterSignOut = snap.requirePasswordAfterSignOut;
			}
			if (snap.currentUser && typeof snap.currentUser.id === 'number') {
				this.currentUser = snap.currentUser;
				this.initialized = true;
			}
		} catch {
			// Ignore malformed snapshot
		}
	}

	private persistSnapshot(): void {
		if (typeof window === 'undefined') return;
		try {
			if (!this.currentUser) {
				clearAllCachedSnapshots();
				return;
			}
			const snap: AuthSnapshot = {
				hasHost: this.hasHost,
				disallowUserRegistration: this.disallowUserRegistration,
				currentUser: this.currentUser,
				requirePasswordAfterSignOut: this.requirePasswordAfterSignOut
			};
			localStorage.setItem(AUTH_SNAPSHOT_KEY, JSON.stringify(snap));
		} catch {
			// Ignore storage quota errors
		}
	}

	async refresh(): Promise<void> {
		try {
			const status = await authApi.getStatus();
			this.hasHost = status.hasHost;
			this.disallowUserRegistration = status.disallowUserRegistration;
			this.currentUser = status.currentUser;
			this.lockConfig = status.lockConfig;
			this.localUsers = status.localUsers;
			this.requirePasswordAfterSignOut = status.requirePasswordAfterSignOut;
			if (!this.initialLockChecked) {
				this.initialLockChecked = true;
				const canBiometric = supportsAndroidBiometric() && Boolean(status.lockConfig?.biometricEnabled);
				if (status.currentUser && (status.lockConfig?.hasPin || canBiometric)) {
					this.isLocked = true;
				}
			}
			this.persistSnapshot();
		} catch {
			// Browser preview fallback when running outside Tauri runtime
			this.hasHost = false;
			this.currentUser = null;
			this.localUsers = [];
			this.requirePasswordAfterSignOut = false;
			clearAllCachedSnapshots();
		} finally {
			this.initialized = true;
		}
	}

	async setupHost(username: string, password: string, nickname?: string): Promise<User> {
		const user = await authApi.setupHost({ username, password, nickname });
		await this.refresh();
		return user;
	}

	async signIn(username: string, password: string): Promise<User> {
		const user = await authApi.signIn({ username, password });
		this.isLocked = false;
		await this.refresh();
		return user;
	}

	async signUp(username: string, password: string, nickname?: string): Promise<User> {
		const user = await authApi.signUp({ username, password, nickname });
		this.isLocked = false;
		await this.refresh();
		return user;
	}

	async signOut(): Promise<void> {
		clearAllCachedSnapshots();
		await authApi.signOut();
		this.isLocked = false;
		await this.refresh();
	}

	async switchAccount(payload: {
		userId: number;
		password?: string;
		pin?: string;
		biometricVerified?: boolean;
	}): Promise<User> {
		clearAllCachedSnapshots();
		const user = await authApi.switchAccount(payload);
		this.isLocked = false;
		await this.refresh();
		return user;
	}

	unlock(): void {
		this.isLocked = false;
	}

	lock(): void {
		const canBiometric = supportsAndroidBiometric() && Boolean(this.lockConfig?.biometricEnabled);
		if (this.lockConfig?.hasPin || canBiometric) {
			this.isLocked = true;
		}
	}
}

export const authState = new AuthState();
