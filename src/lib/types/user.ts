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

export type UserRole = 'HOST' | 'ADMIN' | 'USER';
export type RowStatus = 'NORMAL' | 'ARCHIVED';

export interface User {
	id: number;
	createdTs: number;
	updatedTs: number;
	rowStatus: RowStatus;
	username: string;
	role: UserRole;
	email: string;
	nickname: string;
	avatarUrl: string;
	description: string;
}

export interface UserLockConfig {
	userId: number;
	hasPin: boolean;
	biometricEnabled: boolean;
	requireUnlockOnSwitch: boolean;
}

export interface AuthStatus {
	hasHost: boolean;
	disallowUserRegistration: boolean;
	currentUser: User | null;
	lockConfig: UserLockConfig | null;
	localUsers: User[];
	requirePasswordAfterSignOut: boolean;
}

export interface UserProfileStats {
	user: User;
	memoCount: number;
	tagCount: number;
	activeDays: number;
}
