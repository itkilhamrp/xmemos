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
import type { SystemInfo, SystemSetting } from '#lib/types/setting.ts';

export const systemApi = {
	getInfo: (): Promise<SystemInfo> => callIpc<SystemInfo>('system_get_info'),

	listSettings: (): Promise<SystemSetting[]> => callIpc<SystemSetting[]>('system_list_settings'),

	upsertSetting: (name: string, value: string, description = ''): Promise<SystemSetting> =>
		callIpc<SystemSetting>('system_upsert_setting', { name, value, description }),

	getLocalPref: (key: string): Promise<string | null> =>
		callIpc<string | null>('system_get_local_pref', { key }),

	setLocalPref: (key: string, value: string): Promise<void> =>
		callIpc<void>('system_set_local_pref', { key, value }),

	showMainWindow: (): Promise<void> => callIpc<void>('app_show_main_window'),

	setWindowThemeBg: (isDark: boolean): Promise<void> =>
		callIpc<void>('system_set_window_theme_bg', { isDark })
};
