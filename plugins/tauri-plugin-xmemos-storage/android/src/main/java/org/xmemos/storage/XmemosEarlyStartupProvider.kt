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

package org.xmemos.storage

import android.content.ContentProvider
import android.content.ContentValues
import android.content.Context
import android.database.Cursor
import android.net.Uri
import android.os.Build
import android.webkit.WebView
import java.io.File
import java.util.concurrent.Executors

/**
 * Ultra-lightweight (<1ms on Main Thread) pre-Activity ContentProvider that runs before
 * MainActivity.onCreate(). Offloads two slow cold-start I/O steps to a background thread:
 * 1. Pre-resolves and caches the Android media root directory (`externalMediaDirs`) so Rust's
 *    `getMediaRoot()` IPC returns in 0ms without blocking on storage I/O.
 * 2. Touches `WebView.getCurrentWebViewPackage()` (API 26+) off the UI thread to warm the
 *    OS page cache for the system Chromium WebView package before Activity inflation.
 */
class XmemosEarlyStartupProvider : ContentProvider() {
    companion object {
        @Volatile
        internal var cachedMediaRoot: Pair<File, Boolean>? = null

        @Suppress("DEPRECATION")
        internal fun resolveAndCacheMediaRoot(context: Context): Pair<File, Boolean> {
            cachedMediaRoot?.let { return it }
            val mediaDir = context.externalMediaDirs?.firstOrNull { it != null }
            val result = if (mediaDir != null) {
                Pair(mediaDir, true)
            } else {
                val fallback = context.getExternalFilesDir(null) ?: context.filesDir
                Pair(fallback, false)
            }
            File(result.first, "assets/avatars").mkdirs()
            File(result.first, "backups").mkdirs()
            File(result.first, "temp").mkdirs()
            cachedMediaRoot = result
            return result
        }
    }

    override fun onCreate(): Boolean {
        val appContext = context?.applicationContext ?: return true
        val executor = Executors.newSingleThreadExecutor { runnable ->
            Thread(runnable, "xmemos-early-prewarm").apply {
                isDaemon = true
            }
        }
        executor.execute {
            try {
                resolveAndCacheMediaRoot(appContext)
                if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                    runCatching { WebView.getCurrentWebViewPackage() }
                }
            } catch (_: Throwable) {
                // Non-fatal prewarm
            } finally {
                executor.shutdown()
            }
        }
        return true
    }

    override fun query(
        uri: Uri,
        projection: Array<out String>?,
        selection: String?,
        selectionArgs: Array<out String>?,
        sortOrder: String?,
    ): Cursor? = null

    override fun getType(uri: Uri): String? = null
    override fun insert(uri: Uri, values: ContentValues?): Uri? = null
    override fun delete(uri: Uri, selection: String?, selectionArgs: Array<out String>?): Int = 0
    override fun update(
        uri: Uri,
        values: ContentValues?,
        selection: String?,
        selectionArgs: Array<out String>?,
    ): Int = 0
}
