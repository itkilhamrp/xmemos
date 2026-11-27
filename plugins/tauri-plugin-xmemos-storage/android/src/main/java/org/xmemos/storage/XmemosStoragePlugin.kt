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
import android.appwidget.AppWidgetManager
import android.content.ComponentName
import android.content.ContentValues
import android.content.Context
import android.content.Intent
import android.media.MediaScannerConnection
import android.os.Environment
import android.provider.MediaStore
import android.os.Bundle
import android.graphics.Bitmap
import android.graphics.Canvas
import android.graphics.Color
import android.graphics.LinearGradient
import android.graphics.Paint
import android.graphics.Path
import android.graphics.RectF
import android.graphics.Shader
import android.graphics.Typeface
import android.net.Uri
import android.os.Build
import android.provider.OpenableColumns
import android.webkit.WebView
import androidx.activity.result.ActivityResult
import androidx.biometric.BiometricManager
import androidx.biometric.BiometricPrompt
import androidx.core.content.ContextCompat
import androidx.core.content.pm.ShortcutInfoCompat
import androidx.core.content.pm.ShortcutManagerCompat
import androidx.core.graphics.drawable.IconCompat
import androidx.fragment.app.FragmentActivity
import app.tauri.annotation.ActivityCallback
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import java.io.File
import java.io.FileInputStream
import java.io.FileOutputStream
import java.util.UUID

@InvokeArg
class NomediaArgs {
    var enabled: Boolean = true
}

@InvokeArg
class PickDocumentArgs {
    var mimeTypes: Array<String> = arrayOf("*/*")
    var copyToTemp: Boolean = true
}

@InvokeArg
class SaveDocumentArgs {
    var sourcePath: String = ""
    var suggestedFilename: String = "export.bin"
    var mimeType: String = "application/octet-stream"
}

@InvokeArg
class BiometricArgs {
    var title: String = "Unlock Xmemos"
    var subtitle: String = "Verify your identity to continue"
    var negativeButtonText: String = "Use PIN"
}

@InvokeArg
class CreateMemoShortcutArgs {
    var memoUid: String = ""
    var title: String = ""
    var preview: String = ""
    var pinned: Boolean = false
    var color: String? = null
    var tags: Array<String> = emptyArray()
    var location: String? = null
    var attachmentCount: Int = 0
    var timestamp: String? = null
}

@InvokeArg
class UpdateMemoShortcutColorArgs {
    var memoUid: String = ""
    var color: String = "yellow"
    var title: String? = null
    var preview: String? = null
    var pinned: Boolean = false
}

@InvokeArg
class CompressVideoArgs {
    var sourcePath: String = ""
    var maxShortSide: Int? = null
    var targetBitrate: Int? = null
}

@InvokeArg
class SaveToDownloadsArgs {
    var sourcePath: String = ""
    var filename: String = "file"
    var mimeType: String = "application/octet-stream"
}

@TauriPlugin
class XmemosStoragePlugin(private val activity: Activity) : Plugin(activity) {
    companion object {
        internal const val ACTION_OPEN_MEMO_SHORTCUT = "org.xmemos.app.ACTION_OPEN_MEMO_SHORTCUT"
        internal const val EXTRA_MEMO_UID = "org.xmemos.app.EXTRA_MEMO_UID"
        private val MEMO_UID_REGEX = Regex("^[a-zA-Z0-9_-]{1,64}$")

        private data class ShortcutBitmapPalette(
            val bgStops: IntArray,
            val cardTop: Int,
            val cardBottom: Int,
            val accent: Int,
            val linePrimary: Int,
            val lineSecondary: Int
        )

        private fun resolveShortcutBitmapPalette(color: String?): ShortcutBitmapPalette {
            return when (XmemosMemoWidgetProvider.normalizeColor(color)) {
                "slate" -> ShortcutBitmapPalette(
                    bgStops = intArrayOf(
                        Color.parseColor("#3AC0FF"),
                        Color.parseColor("#0A84FF"),
                        Color.parseColor("#0056D2"),
                        Color.parseColor("#06287A")
                    ),
                    cardTop = Color.argb(252, 255, 255, 255),
                    cardBottom = Color.argb(238, 226, 236, 248),
                    accent = Color.parseColor("#0A84FF"),
                    linePrimary = Color.argb(145, 15, 23, 42),
                    lineSecondary = Color.argb(95, 15, 23, 42)
                )
                "blue" -> ShortcutBitmapPalette(
                    bgStops = intArrayOf(
                        Color.parseColor("#7DD3FC"),
                        Color.parseColor("#38BDF8"),
                        Color.parseColor("#0284C7"),
                        Color.parseColor("#075985")
                    ),
                    cardTop = Color.argb(252, 240, 249, 255),
                    cardBottom = Color.argb(242, 219, 239, 255),
                    accent = Color.parseColor("#0284C7"),
                    linePrimary = Color.argb(150, 23, 56, 84),
                    lineSecondary = Color.argb(95, 23, 56, 84)
                )
                "green" -> ShortcutBitmapPalette(
                    bgStops = intArrayOf(
                        Color.parseColor("#6EE7B7"),
                        Color.parseColor("#34D399"),
                        Color.parseColor("#10B981"),
                        Color.parseColor("#065F46")
                    ),
                    cardTop = Color.argb(252, 240, 253, 244),
                    cardBottom = Color.argb(242, 220, 252, 231),
                    accent = Color.parseColor("#059669"),
                    linePrimary = Color.argb(150, 20, 68, 43),
                    lineSecondary = Color.argb(95, 20, 68, 43)
                )
                "pink" -> ShortcutBitmapPalette(
                    bgStops = intArrayOf(
                        Color.parseColor("#F9A8D4"),
                        Color.parseColor("#F472B6"),
                        Color.parseColor("#EC4899"),
                        Color.parseColor("#9D174D")
                    ),
                    cardTop = Color.argb(252, 253, 242, 248),
                    cardBottom = Color.argb(242, 252, 231, 243),
                    accent = Color.parseColor("#DB2777"),
                    linePrimary = Color.argb(150, 86, 25, 58),
                    lineSecondary = Color.argb(95, 86, 25, 58)
                )
                "purple" -> ShortcutBitmapPalette(
                    bgStops = intArrayOf(
                        Color.parseColor("#C4B5FD"),
                        Color.parseColor("#A78BFA"),
                        Color.parseColor("#8B5CF6"),
                        Color.parseColor("#5B21B6")
                    ),
                    cardTop = Color.argb(252, 245, 243, 255),
                    cardBottom = Color.argb(242, 237, 233, 254),
                    accent = Color.parseColor("#7C3AED"),
                    linePrimary = Color.argb(150, 48, 30, 96),
                    lineSecondary = Color.argb(95, 48, 30, 96)
                )
                else -> ShortcutBitmapPalette(
                    bgStops = intArrayOf(
                        Color.parseColor("#FDE68A"),
                        Color.parseColor("#FBBF24"),
                        Color.parseColor("#F59E0B"),
                        Color.parseColor("#B45309")
                    ),
                    cardTop = Color.argb(252, 255, 251, 235),
                    cardBottom = Color.argb(242, 254, 243, 199),
                    accent = Color.parseColor("#D97706"),
                    linePrimary = Color.argb(155, 66, 48, 12),
                    lineSecondary = Color.argb(100, 66, 48, 12)
                )
            }
        }

        /**
         * Renders a crisp, Sticky Note-themed Liquid Glass adaptive shortcut bitmap (192x192 ARGB_8888)
         * matching the 6 Desktop Sticky Note palettes (yellow, blue, green, pink, purple, slate).
         */
        internal fun renderMemoShortcutBitmap(title: String, pinned: Boolean, color: String? = "yellow"): Bitmap {
            val size = 192
            val bmp = Bitmap.createBitmap(size, size, Bitmap.Config.ARGB_8888)
            val canvas = Canvas(bmp)
            val paint = Paint(Paint.ANTI_ALIAS_FLAG)
            val palette = resolveShortcutBitmapPalette(color)

            // Full-bleed sticky color gradient (safe for Android Adaptive Icon masks)
            paint.shader = LinearGradient(
                0f, 0f, size.toFloat(), size.toFloat(),
                palette.bgStops,
                floatArrayOf(0f, 0.34f, 0.72f, 1f),
                Shader.TileMode.CLAMP
            )
            canvas.drawRect(0f, 0f, size.toFloat(), size.toFloat(), paint)
            paint.shader = null

            // Centered frosted memo card inside Adaptive Icon 66% safe zone (36..156)
            val cardRect = RectF(42f, 38f, 150f, 154f)
            val cardRadius = 20f

            // Subtle card elevation shadow
            paint.color = Color.argb(56, 15, 23, 42)
            canvas.drawRoundRect(
                RectF(cardRect.left, cardRect.top + 4f, cardRect.right, cardRect.bottom + 5f),
                cardRadius, cardRadius, paint
            )

            // Tinted frosted Sticky Note card
            paint.shader = LinearGradient(
                cardRect.left, cardRect.top, cardRect.right, cardRect.bottom,
                palette.cardTop,
                palette.cardBottom,
                Shader.TileMode.CLAMP
            )
            canvas.drawRoundRect(cardRect, cardRadius, cardRadius, paint)
            paint.shader = null

            // Top color accent strip inside the sticky card
            paint.color = palette.accent
            canvas.drawRoundRect(RectF(56f, 48f, 92f, 54f), 3f, 3f, paint)

            // Optional top-right amber corner-fold pin indicator (1:1 MemoCard parity)
            if (pinned) {
                val saveCount = canvas.save()
                val clipPath = Path().apply {
                    addRoundRect(cardRect, cardRadius, cardRadius, Path.Direction.CW)
                }
                canvas.clipPath(clipPath)
                val foldPath = Path().apply {
                    moveTo(cardRect.right, cardRect.top)
                    lineTo(cardRect.right - 26f, cardRect.top)
                    lineTo(cardRect.right, cardRect.top + 26f)
                    close()
                }
                paint.color = Color.parseColor("#FBBF24")
                canvas.drawPath(foldPath, paint)
                canvas.restoreToCount(saveCount)
            }

            // Bold title initial badge in center of mini card
            val initial = title.trim().firstOrNull { it.isLetterOrDigit() }?.uppercaseChar()?.toString() ?: "X"
            paint.color = palette.accent
            paint.textAlign = Paint.Align.CENTER
            paint.typeface = Typeface.create(Typeface.SANS_SERIF, Typeface.BOLD)
            paint.textSize = 42f
            val fm = paint.fontMetrics
            val textCenterY = 86f - (fm.ascent + fm.descent) / 2f
            canvas.drawText(initial, size / 2f, textCenterY, paint)

            // Clean widget-like mini note lines below initial
            paint.color = palette.linePrimary
            canvas.drawRoundRect(RectF(62f, 116f, 130f, 124f), 4f, 4f, paint)
            paint.color = palette.lineSecondary
            canvas.drawRoundRect(RectF(62f, 131f, 114f, 139f), 4f, 4f, paint)

            return bmp
        }

        /**
         * Updates any existing pinned or dynamic Android Home-Screen shortcut for [memoUid]
         * using ShortcutManagerCompat.updateShortcuts() with a freshly colored adaptive bitmap.
         */
        internal fun updateShortcutIconColor(
            context: Context,
            memoUid: String,
            title: String?,
            preview: String?,
            pinned: Boolean,
            color: String
        ): Boolean {
            if (!MEMO_UID_REGEX.matches(memoUid)) return false
            val shortcutId = "xmemos-memo-$memoUid"
            return runCatching {
                val existing = ShortcutManagerCompat.getShortcuts(
                    context,
                    ShortcutManagerCompat.FLAG_MATCH_PINNED or ShortcutManagerCompat.FLAG_MATCH_DYNAMIC
                ).firstOrNull { it.id == shortcutId } ?: return false

                val cleanTitle = title?.replace(Regex("[\r\n\t]+"), " ")?.trim()
                    ?.ifEmpty { null }
                    ?: existing.shortLabel.toString().ifBlank { "Xmemos Memo" }
                val shortLabel = cleanTitle.take(24)
                val longLabel = preview?.replace(Regex("[\r\n\t]+"), " ")?.trim()
                    ?.ifEmpty { null }
                    ?: existing.longLabel?.toString()?.ifBlank { null }
                    ?: cleanTitle

                val launchIntent = (
                    context.packageManager.getLaunchIntentForPackage(context.packageName)
                        ?: existing.intent
                    ).apply {
                    action = ACTION_OPEN_MEMO_SHORTCUT
                    putExtra(EXTRA_MEMO_UID, memoUid)
                    flags = Intent.FLAG_ACTIVITY_NEW_TASK or
                        Intent.FLAG_ACTIVITY_CLEAR_TOP or
                        Intent.FLAG_ACTIVITY_SINGLE_TOP
                }

                val bitmap = renderMemoShortcutBitmap(shortLabel, pinned, color)
                val icon = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                    IconCompat.createWithAdaptiveBitmap(bitmap)
                } else {
                    IconCompat.createWithBitmap(bitmap)
                }

                val updatedInfo = ShortcutInfoCompat.Builder(context, shortcutId)
                    .setShortLabel(shortLabel)
                    .setLongLabel(longLabel.take(60))
                    .setIcon(icon)
                    .setIntent(launchIntent)
                    .build()

                ShortcutManagerCompat.updateShortcuts(context, listOf(updatedInfo))
            }.getOrDefault(false)
        }
    }

    private var pendingSaveSourcePath: String? = null
    private var pendingCopyToTemp: Boolean = true

    @Volatile
    private var pendingShortcutMemoUid: String? = null

    override fun load(webView: WebView) {
        super.load(webView)
        XmemosStartupOptimizer.applyOnLoad(activity, webView)
        extractMemoUidFromIntent(activity.intent)?.let { uid ->
            pendingShortcutMemoUid = uid
        }
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        extractMemoUidFromIntent(intent)?.let { uid ->
            pendingShortcutMemoUid = uid
            val payload = JSObject()
            payload.put("memoUid", uid)
            XmemosMemoWidgetProvider.getSavedMemoColor(activity, uid)?.let { savedColor ->
                payload.put("color", savedColor)
            }
            trigger("memoShortcutOpened", payload)
        }
    }

    private fun extractMemoUidFromIntent(intent: Intent?): String? {
        val raw = intent?.getStringExtra(EXTRA_MEMO_UID)?.trim() ?: return null
        return if (MEMO_UID_REGEX.matches(raw)) raw else null
    }

    @Command
    fun createMemoShortcut(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(CreateMemoShortcutArgs::class.java)
            val uid = args.memoUid.trim()
            if (!MEMO_UID_REGEX.matches(uid)) {
                invoke.reject("Invalid memo UID for shortcut")
                return
            }

            val cleanTitle = args.title.replace(Regex("[\\r\\n\\t]+"), " ").trim()
                .ifEmpty { "Xmemos Memo" }
            val shortLabel = cleanTitle.take(24)
            val longLabel = args.preview.replace(Regex("[\\r\\n\\t]+"), " ").trim()
                .ifEmpty { cleanTitle }
                .take(220)

            val launchIntent = (
                activity.packageManager.getLaunchIntentForPackage(activity.packageName)
                    ?: Intent(activity, activity::class.java)
                ).apply {
                action = ACTION_OPEN_MEMO_SHORTCUT
                putExtra(EXTRA_MEMO_UID, uid)
                flags = Intent.FLAG_ACTIVITY_NEW_TASK or
                    Intent.FLAG_ACTIVITY_CLEAR_TOP or
                    Intent.FLAG_ACTIVITY_SINGLE_TOP
            }

            val safeColor = XmemosMemoWidgetProvider.normalizeColor(args.color)
            val bitmap = renderMemoShortcutBitmap(shortLabel, args.pinned, safeColor)
            val icon = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                IconCompat.createWithAdaptiveBitmap(bitmap)
            } else {
                IconCompat.createWithBitmap(bitmap)
            }

            val shortcutId = "xmemos-memo-$uid"
            val shortcutInfo = ShortcutInfoCompat.Builder(activity, shortcutId)
                .setShortLabel(shortLabel)
                .setLongLabel(longLabel.take(60))
                .setIcon(icon)
                .setIntent(launchIntent)
                .build()

            activity.runOnUiThread {
                try {
                    // Publish dynamic shortcut for instant long-press app icon access
                    runCatching { ShortcutManagerCompat.pushDynamicShortcut(activity, shortcutInfo) }

                    // 1. Prefer real resizable MS Sticky Notes Home-Screen Widget on Android 8+ (API 26+)
                    if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
                        val appWidgetManager = AppWidgetManager.getInstance(activity)
                        if (appWidgetManager != null && appWidgetManager.isRequestPinAppWidgetSupported) {
                            val metaParts = mutableListOf<String>()
                            val cleanTags = args.tags.map { it.trim().trimStart('#') }.filter { it.isNotEmpty() }.take(3)
                            if (cleanTags.isNotEmpty()) {
                                metaParts.add(cleanTags.joinToString(" ") { "#$it" })
                            }
                            val cleanLoc = args.location?.trim().orEmpty()
                            if (cleanLoc.isNotEmpty()) {
                                metaParts.add("\uD83D\uDCCD ${cleanLoc.take(24)}")
                            }
                            if (args.attachmentCount > 0) {
                                metaParts.add("\uD83D\uDCCE ${args.attachmentCount}")
                            }
                            val richMeta = metaParts.joinToString(" · ")
                            val cleanTs = args.timestamp?.trim().orEmpty()
                            val richBadge = when {
                                args.pinned && cleanTs.isNotEmpty() -> "\uD83D\uDCCC $cleanTs"
                                args.pinned -> "\uD83D\uDCCC Xmemos"
                                cleanTs.isNotEmpty() -> cleanTs
                                else -> "Xmemos"
                            }

                            XmemosMemoWidgetProvider.savePendingWidgetMemo(
                                activity,
                                uid,
                                cleanTitle.take(48),
                                longLabel,
                                richMeta,
                                richBadge,
                                safeColor,
                                args.pinned
                            )
                            val provider = ComponentName(activity, XmemosMemoWidgetProvider::class.java)
                            val previewViews = XmemosMemoWidgetProvider.buildRemoteViews(
                                context = activity,
                                memoUid = uid,
                                title = cleanTitle.take(48),
                                preview = longLabel,
                                meta = richMeta,
                                badge = richBadge,
                                color = safeColor,
                                requestCode = uid.hashCode()
                            )
                            val extras = Bundle().apply {
                                putParcelable(AppWidgetManager.EXTRA_APPWIDGET_PREVIEW, previewViews)
                            }
                            if (appWidgetManager.requestPinAppWidget(provider, extras, null)) {
                                val ret = JSObject()
                                ret.put("created", true)
                                ret.put("unsupported", false)
                                invoke.resolve(ret)
                                return@runOnUiThread
                            }
                        }
                    }

                    // 2. Graceful fallback to pinned Home-Screen Shortcut on API 24-25 or restricted launchers
                    if (!ShortcutManagerCompat.isRequestPinShortcutSupported(activity)) {
                        val ret = JSObject()
                        ret.put("created", false)
                        ret.put("unsupported", true)
                        ret.put("message", "Current Android launcher does not support pinned widgets or shortcuts")
                        invoke.resolve(ret)
                        return@runOnUiThread
                    }

                    val requested = ShortcutManagerCompat.requestPinShortcut(activity, shortcutInfo, null)
                    val ret = JSObject()
                    ret.put("created", requested)
                    ret.put("unsupported", !requested)
                    if (!requested) {
                        ret.put("message", "Launcher declined pinned shortcut request")
                    }
                    invoke.resolve(ret)
                } catch (e: Exception) {
                    invoke.reject("Failed to pin Android memo sticky widget/shortcut: ${e.message}")
                }
            }
        } catch (e: Exception) {
            invoke.reject("Failed to create Android memo shortcut: ${e.message}")
        }
    }

    @Command
    fun updateMemoShortcutColor(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(UpdateMemoShortcutColorArgs::class.java)
            val uid = args.memoUid.trim()
            if (!MEMO_UID_REGEX.matches(uid)) {
                invoke.reject("Invalid memo UID for shortcut color update")
                return
            }
            val safeColor = XmemosMemoWidgetProvider.normalizeColor(args.color)
            val updatedWidgets = XmemosMemoWidgetProvider.updateWidgetsForMemoColor(
                context = activity,
                memoUid = uid,
                color = safeColor
            )
            val updatedShortcut = updateShortcutIconColor(
                context = activity,
                memoUid = uid,
                title = args.title,
                preview = args.preview,
                pinned = args.pinned,
                color = safeColor
            )
            val ret = JSObject()
            ret.put("updatedWidgets", updatedWidgets)
            ret.put("updatedShortcut", updatedShortcut)
            ret.put("color", safeColor)
            invoke.resolve(ret)
        } catch (e: Exception) {
            invoke.reject("Failed to update Android memo shortcut color: ${e.message}")
        }
    }

    @Command
    fun consumePendingMemoShortcut(invoke: Invoke) {
        val uid = pendingShortcutMemoUid
        pendingShortcutMemoUid = null
        val ret = JSObject()
        if (uid != null) {
            ret.put("memoUid", uid)
            XmemosMemoWidgetProvider.getSavedMemoColor(activity, uid)?.let { savedColor ->
                ret.put("color", savedColor)
            }
        }
        invoke.resolve(ret)
    }

    @Suppress("DEPRECATION")
    private fun resolveMediaRootDir(): Pair<File, Boolean> {
        return XmemosEarlyStartupProvider.resolveAndCacheMediaRoot(activity.applicationContext)
    }

    @Command
    fun saveToDownloads(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(SaveToDownloadsArgs::class.java)
            val srcFile = File(args.sourcePath)
            if (!srcFile.exists() || !srcFile.isFile) {
                invoke.reject("Source file does not exist: ${args.sourcePath}")
                return
            }

            val safeName = args.filename
                .replace(Regex("[/\\\\:*?\"<>|\\u0000]"), "_")
                .trim()
                .trimStart('.')
                .ifEmpty { "file" }
            val mime = args.mimeType.trim().ifEmpty { "application/octet-stream" }

            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
                val resolver = activity.contentResolver
                val values = ContentValues().apply {
                    put(MediaStore.MediaColumns.DISPLAY_NAME, safeName)
                    put(MediaStore.MediaColumns.MIME_TYPE, mime)
                    put(
                        MediaStore.MediaColumns.RELATIVE_PATH,
                        "${Environment.DIRECTORY_DOWNLOADS}/Xmemos"
                    )
                    put(MediaStore.MediaColumns.IS_PENDING, 1)
                }

                val uri = resolver.insert(MediaStore.Downloads.EXTERNAL_CONTENT_URI, values)
                if (uri == null) {
                    invoke.reject("Failed to create MediaStore.Downloads entry for $safeName")
                    return
                }

                var written = 0L
                try {
                    resolver.openOutputStream(uri)?.use { outStream ->
                        FileInputStream(srcFile).use { inStream ->
                            written = inStream.copyTo(outStream)
                        }
                    } ?: throw IllegalStateException("Unable to open MediaStore output stream")

                    val doneValues = ContentValues().apply {
                        put(MediaStore.MediaColumns.IS_PENDING, 0)
                    }
                    resolver.update(uri, doneValues, null, null)
                } catch (e: Exception) {
                    runCatching { resolver.delete(uri, null, null) }
                    throw e
                }

                val ret = JSObject()
                ret.put("displayPath", "Download/Xmemos/$safeName")
                ret.put("savedUri", uri.toString())
                ret.put("bytesWritten", written)
                invoke.resolve(ret)
            } else {
                @Suppress("DEPRECATION")
                val pubDir = File(
                    Environment.getExternalStoragePublicDirectory(Environment.DIRECTORY_DOWNLOADS),
                    "Xmemos"
                )
                pubDir.mkdirs()
                var dest = File(pubDir, safeName)
                var idx = 1
                val stem = dest.nameWithoutExtension.ifEmpty { "file" }
                val ext = dest.extension
                while (dest.exists() && idx < 9999) {
                    val nextName = if (ext.isEmpty()) "$stem ($idx)" else "$stem ($idx).$ext"
                    dest = File(pubDir, nextName)
                    idx++
                }
                var written = 0L
                FileInputStream(srcFile).use { inStream ->
                    FileOutputStream(dest).use { outStream ->
                        written = inStream.copyTo(outStream)
                    }
                }
                MediaScannerConnection.scanFile(activity, arrayOf(dest.absolutePath), arrayOf(mime), null)

                val ret = JSObject()
                ret.put("displayPath", "Download/Xmemos/${dest.name}")
                ret.put("savedUri", Uri.fromFile(dest).toString())
                ret.put("bytesWritten", written)
                invoke.resolve(ret)
            }
        } catch (e: Exception) {
            invoke.reject("Failed to save to Android Downloads: ${e.message}")
        }
    }

    @Command
    fun getMediaRoot(invoke: Invoke) {
        try {
            val (root, isAndroidMedia) = resolveMediaRootDir()
            val ret = JSObject()
            ret.put("rootPath", root.absolutePath)
            ret.put("assetsPath", File(root, "assets").absolutePath)
            ret.put("backupsPath", File(root, "backups").absolutePath)
            ret.put("tempPath", File(root, "temp").absolutePath)
            ret.put("isAndroidMediaDir", isAndroidMedia)
            invoke.resolve(ret)
        } catch (e: Exception) {
            invoke.reject("Failed to resolve Android media root: ${e.message}")
        }
    }

    @Command
    fun setNomedia(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(NomediaArgs::class.java)
            val (root, _) = resolveMediaRootDir()
            val nomediaFile = File(File(root, "assets"), ".nomedia")
            if (args.enabled) {
                if (!nomediaFile.exists()) {
                    nomediaFile.createNewFile()
                }
            } else if (nomediaFile.exists()) {
                nomediaFile.delete()
            }
            val ret = JSObject()
            ret.put("enabled", nomediaFile.exists())
            ret.put("path", nomediaFile.absolutePath)
            invoke.resolve(ret)
        } catch (e: Exception) {
            invoke.reject("Failed to toggle .nomedia: ${e.message}")
        }
    }

    @Command
    fun getNomediaStatus(invoke: Invoke) {
        try {
            val (root, _) = resolveMediaRootDir()
            val nomediaFile = File(File(root, "assets"), ".nomedia")
            val ret = JSObject()
            ret.put("enabled", nomediaFile.exists())
            ret.put("path", nomediaFile.absolutePath)
            invoke.resolve(ret)
        } catch (e: Exception) {
            invoke.reject("Failed to read .nomedia status: ${e.message}")
        }
    }

    @Command
    fun pickDocument(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(PickDocumentArgs::class.java)
            pendingCopyToTemp = args.copyToTemp
            val intent = Intent(Intent.ACTION_OPEN_DOCUMENT).apply {
                addCategory(Intent.CATEGORY_OPENABLE)
                type = if (args.mimeTypes.size == 1) args.mimeTypes[0] else "*/*"
                if (args.mimeTypes.size > 1) {
                    putExtra(Intent.EXTRA_MIME_TYPES, args.mimeTypes)
                }
            }
            startActivityForResult(invoke, intent, "handlePickDocumentResult")
        } catch (e: Exception) {
            invoke.reject("Failed to open Android document picker: ${e.message}")
        }
    }

    @ActivityCallback
    private fun handlePickDocumentResult(invoke: Invoke, result: ActivityResult) {
        if (result.resultCode != Activity.RESULT_OK || result.data?.data == null) {
            val ret = JSObject()
            ret.put("cancelled", true)
            ret.put("size", 0)
            invoke.resolve(ret)
            return
        }

        val uri: Uri = result.data!!.data!!
        try {
            val resolver = activity.contentResolver
            var displayName = "imported_${UUID.randomUUID()}"
            var sizeBytes = 0L
            resolver.query(uri, null, null, null, null)?.use { cursor ->
                val nameIndex = cursor.getColumnIndex(OpenableColumns.DISPLAY_NAME)
                val sizeIndex = cursor.getColumnIndex(OpenableColumns.SIZE)
                if (cursor.moveToFirst()) {
                    if (nameIndex >= 0) {
                        displayName = cursor.getString(nameIndex) ?: displayName
                    }
                    if (sizeIndex >= 0 && !cursor.isNull(sizeIndex)) {
                        sizeBytes = cursor.getLong(sizeIndex)
                    }
                }
            }
            val mimeType = resolver.getType(uri) ?: "application/octet-stream"
            val sanitizedName = displayName.replace(Regex("[^a-zA-Z0-9._-]"), "_")
            val (root, _) = resolveMediaRootDir()
            val tempDir = File(root, "temp")
            val destFile = File(tempDir, "${UUID.randomUUID()}_$sanitizedName")

            var copiedBytes = 0L
            if (pendingCopyToTemp) {
                resolver.openInputStream(uri)?.use { input ->
                    FileOutputStream(destFile).use { output ->
                        copiedBytes = input.copyTo(output)
                    }
                }
            }

            val ret = JSObject()
            ret.put("cancelled", false)
            ret.put("localPath", if (pendingCopyToTemp) destFile.absolutePath else uri.toString())
            ret.put("filename", displayName)
            ret.put("mimeType", mimeType)
            ret.put("size", if (copiedBytes > 0L) copiedBytes else sizeBytes)
            invoke.resolve(ret)
        } catch (e: Exception) {
            invoke.reject("Failed to read picked document: ${e.message}")
        }
    }

    @Command
    fun saveDocument(invoke: Invoke) {
        try {
            val args = invoke.parseArgs(SaveDocumentArgs::class.java)
            val sourceFile = File(args.sourcePath)
            if (!sourceFile.exists() || !sourceFile.isFile) {
                invoke.reject("Source file does not exist: ${args.sourcePath}")
                return
            }
            pendingSaveSourcePath = args.sourcePath
            val intent = Intent(Intent.ACTION_CREATE_DOCUMENT).apply {
                addCategory(Intent.CATEGORY_OPENABLE)
                type = args.mimeType
                putExtra(Intent.EXTRA_TITLE, args.suggestedFilename)
            }
            startActivityForResult(invoke, intent, "handleSaveDocumentResult")
        } catch (e: Exception) {
            invoke.reject("Failed to open Android document saver: ${e.message}")
        }
    }

    @ActivityCallback
    private fun handleSaveDocumentResult(invoke: Invoke, result: ActivityResult) {
        val sourcePath = pendingSaveSourcePath
        pendingSaveSourcePath = null

        if (result.resultCode != Activity.RESULT_OK || result.data?.data == null || sourcePath == null) {
            val ret = JSObject()
            ret.put("cancelled", true)
            ret.put("bytesWritten", 0)
            invoke.resolve(ret)
            return
        }

        val uri: Uri = result.data!!.data!!
        try {
            var written = 0L
            FileInputStream(File(sourcePath)).use { input ->
                activity.contentResolver.openOutputStream(uri, "wt")?.use { output ->
                    written = input.copyTo(output)
                }
            }
            val ret = JSObject()
            ret.put("cancelled", false)
            ret.put("savedUri", uri.toString())
            ret.put("bytesWritten", written)
            invoke.resolve(ret)
        } catch (e: Exception) {
            invoke.reject("Failed to save document via SAF: ${e.message}")
        }
    }

    @Command
    fun checkBiometricAvailable(invoke: Invoke) {
        val biometricManager = BiometricManager.from(activity)
        val status = biometricManager.canAuthenticate(
            BiometricManager.Authenticators.BIOMETRIC_STRONG or
                BiometricManager.Authenticators.BIOMETRIC_WEAK
        )
        val ret = JSObject()
        when (status) {
            BiometricManager.BIOMETRIC_SUCCESS -> {
                ret.put("available", true)
            }
            BiometricManager.BIOMETRIC_ERROR_NO_HARDWARE -> {
                ret.put("available", false)
                ret.put("reason", "No biometric hardware available")
            }
            BiometricManager.BIOMETRIC_ERROR_HW_UNAVAILABLE -> {
                ret.put("available", false)
                ret.put("reason", "Biometric hardware currently unavailable")
            }
            BiometricManager.BIOMETRIC_ERROR_NONE_ENROLLED -> {
                ret.put("available", false)
                ret.put("reason", "No biometrics enrolled on this device")
            }
            else -> {
                ret.put("available", false)
                ret.put("reason", "Biometric authentication unavailable (code $status)")
            }
        }
        invoke.resolve(ret)
    }

    @Command
    fun authenticateBiometric(invoke: Invoke) {
        val fragmentActivity = activity as? FragmentActivity
        if (fragmentActivity == null) {
            val ret = JSObject()
            ret.put("authenticated", false)
            ret.put("message", "Activity does not support BiometricPrompt")
            invoke.resolve(ret)
            return
        }

        val args = invoke.parseArgs(BiometricArgs::class.java)
        val executor = ContextCompat.getMainExecutor(activity)

        activity.runOnUiThread {
            val callback = object : BiometricPrompt.AuthenticationCallback() {
                override fun onAuthenticationSucceeded(result: BiometricPrompt.AuthenticationResult) {
                    val ret = JSObject()
                    ret.put("authenticated", true)
                    invoke.resolve(ret)
                }

                override fun onAuthenticationError(errorCode: Int, errString: CharSequence) {
                    val ret = JSObject()
                    ret.put("authenticated", false)
                    ret.put("message", errString.toString())
                    invoke.resolve(ret)
                }

                override fun onAuthenticationFailed() {
                    // System prompt remains open for retry until user cancels or locks out.
                }
            }

            val prompt = BiometricPrompt(fragmentActivity, executor, callback)
            val promptInfo = BiometricPrompt.PromptInfo.Builder()
                .setTitle(args.title)
                .setSubtitle(args.subtitle)
                .setNegativeButtonText(args.negativeButtonText)
                .setAllowedAuthenticators(
                    BiometricManager.Authenticators.BIOMETRIC_STRONG or
                        BiometricManager.Authenticators.BIOMETRIC_WEAK
                )
                .build()

            prompt.authenticate(promptInfo)
        }
    }

    @Command
    fun compressVideo(invoke: Invoke) {
        val args = invoke.parseArgs(CompressVideoArgs::class.java)
        XmemosVideoCompressor.compress(
            context = activity,
            sourcePath = args.sourcePath,
            maxShortSideArg = args.maxShortSide,
            targetBitrateArg = args.targetBitrate,
        ) { outcome ->
            val ret = JSObject()
            ret.put("compressed", outcome.compressed)
            ret.put("outputPath", outcome.outputPath)
            ret.put("mimeType", outcome.mimeType)
            ret.put("originalBytes", outcome.originalBytes)
            ret.put("outputBytes", outcome.outputBytes)
            invoke.resolve(ret)
        }
    }
}
