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
import java.util.concurrent.TimeUnit
import java.util.concurrent.atomic.AtomicBoolean

/**
 * Native Kotlin first-frame startup optimizer and smart cache manager for Android:
 * 1. Matches the native Activity window & WebView background to Light/Dark mode before first paint
 *    (eliminating white/black cold-start flashes).
 * 2. Tunes Android WebView caching & GPU pre-rasterization (`LOAD_DEFAULT`, `offscreenPreRaster`)
 *    for instant warm/cold reopen.
 * 3. Uses a zero-sleep `ScheduledExecutorService` after first-frame commit (2000ms delay) to run
 *    smart bounded cache pruning (6h TTL on `video_opt`/`export_tmp`, 48 MiB LRU cap on `thumbs`,
 *    and a 160 MiB safety ceiling across `cacheDir` HTTP/code caches so storage never bloats).
 */
internal object XmemosStartupOptimizer {
    private const val MAX_THUMBS_CACHE_BYTES = 48L * 1024L * 1024L
    private const val MAX_TOTAL_CACHE_BYTES = 160L * 1024L * 1024L
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
        val scheduler = Executors.newSingleThreadScheduledExecutor { runnable ->
            Thread(runnable, "xmemos-cache-prune").apply {
                isDaemon = true
                priority = Thread.MIN_PRIORITY
            }
        }
        scheduler.schedule(
            {
                try {
                    val cacheDir = appContext.cacheDir ?: return@schedule
                    pruneStaleDir(File(cacheDir, "video_opt"), STALE_TEMP_AGE_MS)
                    pruneStaleDir(File(cacheDir, "export_tmp"), STALE_TEMP_AGE_MS)
                    pruneLruDir(File(cacheDir, "thumbs"), MAX_THUMBS_CACHE_BYTES)
                    enforceTotalCacheCeiling(cacheDir, MAX_TOTAL_CACHE_BYTES)
                } catch (_: Throwable) {
                    // Non-fatal background cache maintenance
                } finally {
                    scheduler.shutdown()
                }
            },
            2000L,
            TimeUnit.MILLISECONDS
        )
    }

    private fun dirSize(dir: File): Long {
        if (!dir.exists()) return 0L
        var total = 0L
        dir.walkTopDown().forEach { f ->
            if (f.isFile) total += f.length()
        }
        return total
    }

    private fun enforceTotalCacheCeiling(cacheDir: File, maxBytes: Long) {
        val total = dirSize(cacheDir)
        if (total <= maxBytes) return
        // Trim oldest WebView HTTP cache files first when total app cache exceeds 160 MiB
        val webViewHttpCache = File(cacheDir, "WebView/Default/HTTP Cache")
        if (webViewHttpCache.exists()) {
            pruneLruDir(webViewHttpCache, 40L * 1024L * 1024L)
        }
        val remaining = dirSize(cacheDir)
        if (remaining > maxBytes) {
            pruneLruDir(File(cacheDir, "thumbs"), 24L * 1024L * 1024L)
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
        val files = dir.walkTopDown().filter { it.isFile }.toList()
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
