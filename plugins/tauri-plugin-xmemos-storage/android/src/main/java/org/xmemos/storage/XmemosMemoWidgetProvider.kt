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

import android.app.PendingIntent
import android.appwidget.AppWidgetManager
import android.appwidget.AppWidgetProvider
import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.view.View
import android.widget.RemoteViews
import androidx.core.content.ContextCompat

/**
 * Lightweight, zero-background-service MS Sticky Notes-style AppWidgetProvider for Xmemos.
 * Renders rich memo metadata (pin status, formatted checklists/bullets, tags, location,
 * attachment count, and timestamp) with 6 Day/Night Sticky Note color themes and an instant
 * 1-tap color cycler directly on the Home Screen widget via foreground explicit broadcast.
 */
class XmemosMemoWidgetProvider : AppWidgetProvider() {
    data class WidgetColorTheme(
        val id: String,
        val bgDrawableRes: Int,
        val accentColorRes: Int,
        val badgeColorRes: Int,
        val dotDrawableRes: Int
    )

    companion object {
        private const val PREFS_NAME = "xmemos_sticky_widgets"
        private const val KEY_PENDING_UID = "pending_uid"
        private const val KEY_PENDING_TITLE = "pending_title"
        private const val KEY_PENDING_PREVIEW = "pending_preview"
        private const val KEY_PENDING_META = "pending_meta"
        private const val KEY_PENDING_BADGE = "pending_badge"
        private const val KEY_PENDING_COLOR = "pending_color"
        private const val KEY_PENDING_PINNED = "pending_pinned"

        const val ACTION_CYCLE_WIDGET_COLOR = "org.xmemos.storage.ACTION_CYCLE_WIDGET_COLOR"

        private val COLOR_ORDER = listOf("yellow", "blue", "green", "pink", "purple", "slate")

        fun normalizeColor(raw: String?): String {
            val clean = raw?.trim()?.lowercase()
            return if (clean != null && clean in COLOR_ORDER) clean else "yellow"
        }

        fun nextColor(current: String): String {
            val normalized = normalizeColor(current)
            val idx = COLOR_ORDER.indexOf(normalized)
            return COLOR_ORDER[(idx + 1) % COLOR_ORDER.size]
        }

        fun resolveColorTheme(color: String?): WidgetColorTheme {
            return when (normalizeColor(color)) {
                "slate" -> WidgetColorTheme(
                    id = "slate",
                    bgDrawableRes = R.drawable.bg_sticky_widget_slate,
                    accentColorRes = R.color.xmemos_widget_accent_slate,
                    badgeColorRes = R.color.xmemos_widget_badge_slate,
                    dotDrawableRes = R.drawable.dot_sticky_color_slate
                )
                "blue" -> WidgetColorTheme(
                    id = "blue",
                    bgDrawableRes = R.drawable.bg_sticky_widget_blue,
                    accentColorRes = R.color.xmemos_widget_accent_blue,
                    badgeColorRes = R.color.xmemos_widget_badge_blue,
                    dotDrawableRes = R.drawable.dot_sticky_color_blue
                )
                "green" -> WidgetColorTheme(
                    id = "green",
                    bgDrawableRes = R.drawable.bg_sticky_widget_green,
                    accentColorRes = R.color.xmemos_widget_accent_green,
                    badgeColorRes = R.color.xmemos_widget_badge_green,
                    dotDrawableRes = R.drawable.dot_sticky_color_green
                )
                "pink" -> WidgetColorTheme(
                    id = "pink",
                    bgDrawableRes = R.drawable.bg_sticky_widget_pink,
                    accentColorRes = R.color.xmemos_widget_accent_pink,
                    badgeColorRes = R.color.xmemos_widget_badge_pink,
                    dotDrawableRes = R.drawable.dot_sticky_color_pink
                )
                "purple" -> WidgetColorTheme(
                    id = "purple",
                    bgDrawableRes = R.drawable.bg_sticky_widget_purple,
                    accentColorRes = R.color.xmemos_widget_accent_purple,
                    badgeColorRes = R.color.xmemos_widget_badge_purple,
                    dotDrawableRes = R.drawable.dot_sticky_color_purple
                )
                else -> WidgetColorTheme(
                    id = "yellow",
                    bgDrawableRes = R.drawable.bg_sticky_widget_yellow,
                    accentColorRes = R.color.xmemos_widget_accent_yellow,
                    badgeColorRes = R.color.xmemos_widget_badge_yellow,
                    dotDrawableRes = R.drawable.dot_sticky_color_yellow
                )
            }
        }

        fun getSavedMemoColor(context: Context, memoUid: String): String? {
            if (memoUid.isBlank()) return null
            val raw = context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
                .getString("memo_color_$memoUid", null)
                ?: return null
            return normalizeColor(raw)
        }

        fun savePendingWidgetMemo(
            context: Context,
            memoUid: String,
            title: String,
            preview: String,
            meta: String = "",
            badge: String = "Xmemos",
            color: String = "yellow",
            pinned: Boolean = false
        ) {
            val safeColor = normalizeColor(color)
            // Use commit() so the pending memo is guaranteed persisted before the launcher invokes onUpdate()
            context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
                .edit()
                .putString(KEY_PENDING_UID, memoUid)
                .putString(KEY_PENDING_TITLE, title)
                .putString(KEY_PENDING_PREVIEW, preview)
                .putString(KEY_PENDING_META, meta)
                .putString(KEY_PENDING_BADGE, badge)
                .putString(KEY_PENDING_COLOR, safeColor)
                .putBoolean(KEY_PENDING_PINNED, pinned)
                .putString("memo_color_$memoUid", safeColor)
                .commit()
        }

        fun buildRemoteViews(
            context: Context,
            memoUid: String,
            title: String,
            preview: String,
            meta: String = "",
            badge: String = "Xmemos",
            color: String = "yellow",
            requestCode: Int,
            appWidgetId: Int = AppWidgetManager.INVALID_APPWIDGET_ID
        ): RemoteViews {
            val theme = resolveColorTheme(color)
            val views = RemoteViews(context.packageName, R.layout.widget_memo_sticky)
            views.setInt(R.id.sticky_widget_root, "setBackgroundResource", theme.bgDrawableRes)
            views.setInt(R.id.sticky_widget_accent, "setBackgroundResource", theme.accentColorRes)
            views.setImageViewResource(R.id.sticky_widget_color_dot, theme.dotDrawableRes)

            val badgeColorInt = ContextCompat.getColor(context, theme.badgeColorRes)
            views.setTextColor(R.id.sticky_widget_badge, badgeColorInt)
            views.setTextColor(R.id.sticky_widget_meta, badgeColorInt)

            views.setTextViewText(R.id.sticky_widget_title, title.ifBlank { "Xmemos" })
            views.setTextViewText(R.id.sticky_widget_badge, badge.ifBlank { "Xmemos" })
            views.setTextViewText(
                R.id.sticky_widget_body,
                preview.ifBlank { title.ifBlank { "Tap to open memo in Xmemos" } }
            )

            val cleanMeta = meta.trim()
            if (cleanMeta.isNotEmpty()) {
                views.setTextViewText(R.id.sticky_widget_meta, cleanMeta)
                views.setViewVisibility(R.id.sticky_widget_meta, View.VISIBLE)
            } else {
                views.setViewVisibility(R.id.sticky_widget_meta, View.GONE)
            }

            if (appWidgetId != AppWidgetManager.INVALID_APPWIDGET_ID) {
                runCatching {
                    val cycleIntent = Intent(context, XmemosMemoWidgetProvider::class.java).apply {
                        action = ACTION_CYCLE_WIDGET_COLOR
                        putExtra(AppWidgetManager.EXTRA_APPWIDGET_ID, appWidgetId)
                        addFlags(Intent.FLAG_RECEIVER_FOREGROUND)
                    }
                    val cyclePending = PendingIntent.getBroadcast(
                        context,
                        appWidgetId * 31 + 7,
                        cycleIntent,
                        PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
                    )
                    views.setOnClickPendingIntent(R.id.sticky_widget_color_btn, cyclePending)
                }
            }

            runCatching {
                val launchIntent = context.packageManager.getLaunchIntentForPackage(context.packageName)
                if (launchIntent != null) {
                    if (memoUid.isNotBlank()) {
                        launchIntent.action = XmemosStoragePlugin.ACTION_OPEN_MEMO_SHORTCUT
                        launchIntent.putExtra(XmemosStoragePlugin.EXTRA_MEMO_UID, memoUid)
                    }
                    launchIntent.flags = Intent.FLAG_ACTIVITY_NEW_TASK or
                        Intent.FLAG_ACTIVITY_CLEAR_TOP or
                        Intent.FLAG_ACTIVITY_SINGLE_TOP
                    val pendingIntent = PendingIntent.getActivity(
                        context,
                        requestCode,
                        launchIntent,
                        PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
                    )
                    views.setOnClickPendingIntent(R.id.sticky_widget_root, pendingIntent)
                }
            }
            return views
        }

        /**
         * Updates the sticky note color for all pinned Home Screen widgets bound to [memoUid]
         * using full AppWidgetManager.updateAppWidget() calls per official Android guidance.
         */
        fun updateWidgetsForMemoColor(
            context: Context,
            memoUid: String,
            color: String
        ): Int {
            val safeColor = normalizeColor(color)
            val prefs = context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
            prefs.edit().putString("memo_color_$memoUid", safeColor).apply()

            val appWidgetManager = AppWidgetManager.getInstance(context) ?: return 0
            val component = ComponentName(context, XmemosMemoWidgetProvider::class.java)
            val widgetIds = runCatching { appWidgetManager.getAppWidgetIds(component) }
                .getOrDefault(IntArray(0))

            var updated = 0
            val editor = prefs.edit()
            for (widgetId in widgetIds) {
                val storedUid = prefs.getString("uid_$widgetId", null)
                if (storedUid == memoUid) {
                    editor.putString("color_$widgetId", safeColor)
                    val title = prefs.getString("title_$widgetId", null) ?: "Xmemos"
                    val preview = prefs.getString("preview_$widgetId", null) ?: "Tap to open memo in Xmemos"
                    val meta = prefs.getString("meta_$widgetId", null).orEmpty()
                    val badge = prefs.getString("badge_$widgetId", null) ?: "Xmemos"
                    val views = buildRemoteViews(
                        context = context,
                        memoUid = storedUid,
                        title = title,
                        preview = preview,
                        meta = meta,
                        badge = badge,
                        color = safeColor,
                        requestCode = widgetId,
                        appWidgetId = widgetId
                    )
                    appWidgetManager.updateAppWidget(widgetId, views)
                    updated += 1
                }
            }
            if (updated > 0) {
                editor.commit()
            }
            return updated
        }
    }

    override fun onReceive(context: Context, intent: Intent) {
        if (intent.action == ACTION_CYCLE_WIDGET_COLOR) {
            val widgetId = intent.getIntExtra(
                AppWidgetManager.EXTRA_APPWIDGET_ID,
                AppWidgetManager.INVALID_APPWIDGET_ID
            )
            if (widgetId != AppWidgetManager.INVALID_APPWIDGET_ID) {
                runCatching {
                    val prefs = context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
                    val uid = prefs.getString("uid_$widgetId", null).orEmpty()
                    val currentColor = prefs.getString("color_$widgetId", null)
                        ?: if (uid.isNotBlank()) prefs.getString("memo_color_$uid", null) else null
                    val next = nextColor(currentColor ?: "yellow")

                    val editor = prefs.edit().putString("color_$widgetId", next)
                    if (uid.isNotBlank()) {
                        editor.putString("memo_color_$uid", next)
                    }
                    editor.commit()

                    val title = prefs.getString("title_$widgetId", null) ?: "Xmemos"
                    val preview = prefs.getString("preview_$widgetId", null) ?: "Tap to open memo in Xmemos"
                    val meta = prefs.getString("meta_$widgetId", null).orEmpty()
                    val badge = prefs.getString("badge_$widgetId", null) ?: "Xmemos"
                    val pinned = prefs.getBoolean("pinned_$widgetId", false)

                    if (uid.isNotBlank()) {
                        updateWidgetsForMemoColor(context, uid, next)
                        XmemosStoragePlugin.updateShortcutIconColor(
                            context = context,
                            memoUid = uid,
                            title = title,
                            preview = preview,
                            pinned = pinned,
                            color = next
                        )
                    } else {
                        val appWidgetManager = AppWidgetManager.getInstance(context)
                        val views = buildRemoteViews(
                            context = context,
                            memoUid = uid,
                            title = title,
                            preview = preview,
                            meta = meta,
                            badge = badge,
                            color = next,
                            requestCode = widgetId,
                            appWidgetId = widgetId
                        )
                        appWidgetManager?.updateAppWidget(widgetId, views)
                    }
                }
            }
            return
        }
        super.onReceive(context, intent)
    }

    override fun onUpdate(
        context: Context,
        appWidgetManager: AppWidgetManager,
        appWidgetIds: IntArray
    ) {
        val prefs = context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
        val pendingUid = prefs.getString(KEY_PENDING_UID, null)
        val pendingTitle = prefs.getString(KEY_PENDING_TITLE, null)
        val pendingPreview = prefs.getString(KEY_PENDING_PREVIEW, null)
        val pendingMeta = prefs.getString(KEY_PENDING_META, null)
        val pendingBadge = prefs.getString(KEY_PENDING_BADGE, null)
        val pendingColor = prefs.getString(KEY_PENDING_COLOR, null)
        val pendingPinned = prefs.getBoolean(KEY_PENDING_PINNED, false)

        for (widgetId in appWidgetIds) {
            val uidKey = "uid_$widgetId"
            val titleKey = "title_$widgetId"
            val previewKey = "preview_$widgetId"
            val metaKey = "meta_$widgetId"
            val badgeKey = "badge_$widgetId"
            val colorKey = "color_$widgetId"
            val pinnedKey = "pinned_$widgetId"

            var uid = prefs.getString(uidKey, null)
            var title = prefs.getString(titleKey, null)
            var preview = prefs.getString(previewKey, null)
            var meta = prefs.getString(metaKey, null)
            var badge = prefs.getString(badgeKey, null)
            var color = prefs.getString(colorKey, null)

            if (uid == null && pendingUid != null) {
                uid = pendingUid
                title = pendingTitle ?: "Xmemos"
                preview = pendingPreview ?: ""
                meta = pendingMeta ?: ""
                badge = pendingBadge ?: "Xmemos"
                color = normalizeColor(pendingColor)
                prefs.edit()
                    .putString(uidKey, uid)
                    .putString(titleKey, title)
                    .putString(previewKey, preview)
                    .putString(metaKey, meta)
                    .putString(badgeKey, badge)
                    .putString(colorKey, color)
                    .putBoolean(pinnedKey, pendingPinned)
                    .commit()
            } else if (color == null && uid != null) {
                color = prefs.getString("memo_color_$uid", null) ?: "yellow"
            }

            val views = buildRemoteViews(
                context = context,
                memoUid = uid.orEmpty(),
                title = title ?: "Xmemos",
                preview = preview ?: "Tap to open memo in Xmemos",
                meta = meta.orEmpty(),
                badge = badge ?: "Xmemos",
                color = normalizeColor(color),
                requestCode = widgetId,
                appWidgetId = widgetId
            )
            appWidgetManager.updateAppWidget(widgetId, views)
        }
    }

    override fun onDeleted(context: Context, appWidgetIds: IntArray) {
        val editor = context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE).edit()
        for (widgetId in appWidgetIds) {
            editor.remove("uid_$widgetId")
            editor.remove("title_$widgetId")
            editor.remove("preview_$widgetId")
            editor.remove("meta_$widgetId")
            editor.remove("badge_$widgetId")
            editor.remove("color_$widgetId")
            editor.remove("pinned_$widgetId")
        }
        editor.apply()
    }
}
