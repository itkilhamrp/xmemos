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

import { callIpc } from './invoke';
import type { User, UserProfileStats, UserRole } from '#lib/types/user.ts';
import type { UserSetting } from '#lib/types/setting.ts';

export const userApi = {
	list: (includeArchived = false): Promise<User[]> =>
		callIpc<User[]>('user_list', { includeArchived }),

	getProfile: (username: string): Promise<UserProfileStats> =>
		callIpc<UserProfileStats>('user_get_profile', { username }),

	adminCreate: (payload: {
		username: string;
		password: string;
		role: Exclude<UserRole, 'HOST'>;
		nickname?: string;
		email?: string;
	}): Promise<User> => callIpc<User>('user_admin_create', { payload }),

	update: (payload: {
		userId?: number;
		username?: string;
		nickname?: string;
		email?: string;
		description?: string;
		avatarUrl?: string;
		password?: string;
		role?: UserRole;
		rowStatus?: 'NORMAL' | 'ARCHIVED';
	}): Promise<User> => callIpc<User>('user_update', { payload }),

	delete: (userId: number): Promise<void> => callIpc<void>('user_delete', { userId }),

	listSettings: (): Promise<UserSetting[]> => callIpc<UserSetting[]>('user_list_settings'),

	upsertSetting: (key: string, value: string): Promise<UserSetting> =>
		callIpc<UserSetting>('user_upsert_setting', { key, value })
};
