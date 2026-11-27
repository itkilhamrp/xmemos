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

import android.app.Activity
import android.content.Context
import android.content.res.Configuration
import android.graphics.Color
import android.os.Build
import android.webkit.WebSettings
import android.webkit.WebView
import java.io.File
import java.util.concurrent.Executors
import java.util.concurrent.atomic.AtomicBoolean

/**
 * Native Kotlin first-frame startup optimizer for Android:
 * 1. Matches the native Activity window background to Light/Dark mode before first WebView paint
 *    (eliminating white/black cold-start flashes).
 * 2. Tunes Android WebView caching & GPU pre-rasterization (`LOAD_DEFAULT`, `offscreenPreRaster`,
 *    hardware layer) for instant warm/cold reopen.
 * 3. Runs a lightweight background bounded-cache pruner (48 MiB LRU cap on `cacheDir/thumbs` and
 *    6-hour TTL on ephemeral `video_opt`/`export_tmp` staging folders) without blocking the UI thread.
 */
internal object XmemosStartupOptimizer {
    private const val MAX_THUMBS_CACHE_BYTES = 48L * 1024L * 1024L
    private const val STALE_TEMP_AGE_MS = 6L * 60L * 60L * 1000L
    private val maintenanceScheduled = AtomicBoolean(false)

    fun applyOnLoad(activity: Activity, webView: WebView) {
        runCatching {
            val isNight = (activity.resources.configuration.uiMode and Configuration.UI_MODE_NIGHT_MASK) ==
                Configuration.UI_MODE_NIGHT_YES
            val bgColor = if (isNight) Color.BLACK else Color.rgb(242, 242, 247)
            activity.window?.decorView?.setBackgroundColor(bgColor)
            webView.setBackgroundColor(bgColor)
        }

        runCatching {
            val settings = webView.settings
            settings.cacheMode = WebSettings.LOAD_DEFAULT
            settings.domStorageEnabled = true
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
                settings.offscreenPreRaster = true
            }
        }

        scheduleBoundedCachePruning(activity.applicationContext)
    }

    private fun scheduleBoundedCachePruning(appContext: Context) {
        if (!maintenanceScheduled.compareAndSet(false, true)) return
        val executor = Executors.newSingleThreadExecutor { runnable ->
            Thread(runnable, "xmemos-cache-prune").apply {
                isDaemon = true
                priority = Thread.MIN_PRIORITY
            }
        }
        executor.execute {
            try {
                // Yield briefly so first-frame rendering gets 100% of CPU & I/O bandwidth
                Thread.sleep(1500L)
                val cacheDir = appContext.cacheDir ?: return@execute
                pruneStaleDir(File(cacheDir, "video_opt"), STALE_TEMP_AGE_MS)
                pruneStaleDir(File(cacheDir, "export_tmp"), STALE_TEMP_AGE_MS)
                pruneLruDir(File(cacheDir, "thumbs"), MAX_THUMBS_CACHE_BYTES)
            } catch (_: Throwable) {
                // Non-fatal background cache maintenance
            } finally {
                executor.shutdown()
            }
        }
    }

    private fun pruneStaleDir(dir: File, maxAgeMs: Long) {
        if (!dir.exists() || !dir.isDirectory) return
        val cutoff = System.currentTimeMillis() - maxAgeMs
        dir.listFiles()?.forEach { child ->
            runCatching {
                if (child.lastModified() < cutoff) {
                    if (child.isDirectory) child.deleteRecursively() else child.delete()
                }
            }
        }
    }

    private fun pruneLruDir(dir: File, maxBytes: Long) {
        if (!dir.exists() || !dir.isDirectory) return
        val files = dir.listFiles()?.filter { it.isFile } ?: return
        var total = files.sumOf { it.length() }
        if (total <= maxBytes) return
        val sortedOldestFirst = files.sortedBy { it.lastModified() }
        for (file in sortedOldestFirst) {
            if (total <= maxBytes) break
            val len = file.length()
            if (file.delete()) {
                total -= len
            }
        }
    }
}
