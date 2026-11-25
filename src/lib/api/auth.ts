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
import type { AuthStatus, User, UserLockConfig } from '#lib/types/user.ts';

export interface BiometricAvailability {
	available: boolean;
	reason?: string | null;
}

export interface BiometricAuthResponse {
	authenticated: boolean;
	message?: string | null;
}

export const authApi = {
	getStatus: (): Promise<AuthStatus> => callIpc<AuthStatus>('auth_get_status'),

	setupHost: (payload: {
		username: string;
		password: string;
		nickname?: string;
		email?: string;
	}): Promise<User> => callIpc<User>('auth_setup_host', { payload }),

	signUp: (payload: {
		username: string;
		password: string;
		nickname?: string;
		email?: string;
	}): Promise<User> => callIpc<User>('auth_sign_up', { payload }),

	signIn: (payload: { username: string; password: string }): Promise<User> =>
		callIpc<User>('auth_sign_in', { payload }),

	signOut: (): Promise<void> => callIpc<void>('auth_sign_out'),

	switchAccount: (payload: {
		userId: number;
		password?: string;
		pin?: string;
		biometricVerified?: boolean;
	}): Promise<User> => callIpc<User>('auth_switch_account', { payload }),

	setLockConfig: (payload: {
		pin?: string | null;
		clearPin: boolean;
		biometricEnabled: boolean;
		requireUnlockOnSwitch: boolean;
	}): Promise<UserLockConfig> => callIpc<UserLockConfig>('auth_set_lock_config', { payload }),

	verifyPin: (pin: string): Promise<boolean> => callIpc<boolean>('auth_verify_pin', { pin }),

	checkBiometricAvailable: (): Promise<BiometricAvailability> =>
		callIpc<BiometricAvailability>('plugin:xmemos-storage|check_biometric_available'),

	authenticateBiometric: (payload: {
		title: string;
		subtitle: string;
		negativeButtonText: string;
	}): Promise<BiometricAuthResponse> =>
		callIpc<BiometricAuthResponse>('plugin:xmemos-storage|authenticate_biometric', { payload })
};
