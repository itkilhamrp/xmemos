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
import androidx.webkit.WebViewCompat
import androidx.webkit.WebViewOutcomeReceiver
import androidx.webkit.WebViewStartUpConfig
import androidx.webkit.WebViewStartUpResult
import androidx.webkit.WebViewStartupException
import java.io.File
import java.util.concurrent.Executors

/**
 * Ultra-lightweight (<1ms on Main Thread) pre-Activity ContentProvider that runs before
 * MainActivity.onCreate(). Offloads two critical cold-start bottlenecks off the UI thread:
 * 1. Pre-resolves and caches the Android media root directory (`externalMediaDirs`) so Rust's
 *    `getMediaRoot()` IPC returns in 0ms without blocking on storage I/O.
 * 2. Triggers official Jetpack WebKit `WebViewCompat.startUpWebView()` (`androidx.webkit:webkit:1.16.0`)
 *    on a dedicated background executor so Chromium's heavy native initialization runs in parallel
 *    before `MainActivity` inflates the Tauri WebView.
 */
class XmemosEarlyStartupProvider : ContentProvider() {
    companion object {
        @Volatile
        internal var cachedMediaRoot: Pair<File, Boolean>? = null

        private fun ensureDir(dir: File) {
            if (!dir.exists()) {
                dir.mkdirs()
            }
        }

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
            ensureDir(File(result.first, "assets/avatars"))
            ensureDir(File(result.first, "backups"))
            ensureDir(File(result.first, "temp"))
            cachedMediaRoot = result
            return result
        }
    }

    override fun onCreate(): Boolean {
        val appContext = context?.applicationContext ?: return true

        // 1. Official Jetpack WebKit asynchronous WebView startup (Android Developers Oct 2026)
        runCatching {
            val webViewExecutor = Executors.newSingleThreadExecutor { runnable ->
                Thread(runnable, "xmemos-webview-init").apply {
                    isDaemon = true
                }
            }
            val startUpConfig = WebViewStartUpConfig.Builder(webViewExecutor).build()
            WebViewCompat.startUpWebView(
                appContext,
                startUpConfig,
                object : WebViewOutcomeReceiver<WebViewStartUpResult, WebViewStartupException> {
                    override fun onResult(result: WebViewStartUpResult) {
                        webViewExecutor.shutdown()
                    }

                    override fun onError(error: WebViewStartupException) {
                        webViewExecutor.shutdown()
                    }
                }
            )
        }

        // 2. Pre-warm Android MediaRoot off the UI thread
        val ioExecutor = Executors.newSingleThreadExecutor { runnable ->
            Thread(runnable, "xmemos-early-prewarm").apply {
                isDaemon = true
            }
        }
        ioExecutor.execute {
            try {
                resolveAndCacheMediaRoot(appContext)
            } catch (_: Throwable) {
                // Non-fatal prewarm
            } finally {
                ioExecutor.shutdown()
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
