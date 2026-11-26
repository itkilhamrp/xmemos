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
import android.content.Context
import android.content.Intent
import android.view.View
import android.widget.RemoteViews

/**
 * Lightweight, zero-background-service MS Sticky Notes-style AppWidgetProvider for Xmemos.
 * Renders rich memo metadata (pin status, formatted checklists/bullets, tags, location,
 * attachment count, and timestamp) directly via RemoteViews backed by SharedPreferences.
 */
class XmemosMemoWidgetProvider : AppWidgetProvider() {
    companion object {
        private const val PREFS_NAME = "xmemos_sticky_widgets"
        private const val KEY_PENDING_UID = "pending_uid"
        private const val KEY_PENDING_TITLE = "pending_title"
        private const val KEY_PENDING_PREVIEW = "pending_preview"
        private const val KEY_PENDING_META = "pending_meta"
        private const val KEY_PENDING_BADGE = "pending_badge"

        fun savePendingWidgetMemo(
            context: Context,
            memoUid: String,
            title: String,
            preview: String,
            meta: String = "",
            badge: String = "Xmemos"
        ) {
            // Use commit() so the pending memo is guaranteed persisted before the launcher invokes onUpdate()
            context.getSharedPreferences(PREFS_NAME, Context.MODE_PRIVATE)
                .edit()
                .putString(KEY_PENDING_UID, memoUid)
                .putString(KEY_PENDING_TITLE, title)
                .putString(KEY_PENDING_PREVIEW, preview)
                .putString(KEY_PENDING_META, meta)
                .putString(KEY_PENDING_BADGE, badge)
                .commit()
        }

        fun buildRemoteViews(
            context: Context,
            memoUid: String,
            title: String,
            preview: String,
            meta: String = "",
            badge: String = "Xmemos",
            requestCode: Int
        ): RemoteViews {
            val views = RemoteViews(context.packageName, R.layout.widget_memo_sticky)
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

        for (widgetId in appWidgetIds) {
            val uidKey = "uid_$widgetId"
            val titleKey = "title_$widgetId"
            val previewKey = "preview_$widgetId"
            val metaKey = "meta_$widgetId"
            val badgeKey = "badge_$widgetId"

            var uid = prefs.getString(uidKey, null)
            var title = prefs.getString(titleKey, null)
            var preview = prefs.getString(previewKey, null)
            var meta = prefs.getString(metaKey, null)
            var badge = prefs.getString(badgeKey, null)

            if (uid == null && pendingUid != null) {
                uid = pendingUid
                title = pendingTitle ?: "Xmemos"
                preview = pendingPreview ?: ""
                meta = pendingMeta ?: ""
                badge = pendingBadge ?: "Xmemos"
                prefs.edit()
                    .putString(uidKey, uid)
                    .putString(titleKey, title)
                    .putString(previewKey, preview)
                    .putString(metaKey, meta)
                    .putString(badgeKey, badge)
                    .commit()
            }

            val views = buildRemoteViews(
                context = context,
                memoUid = uid.orEmpty(),
                title = title ?: "Xmemos",
                preview = preview ?: "Tap to open memo in Xmemos",
                meta = meta.orEmpty(),
                badge = badge ?: "Xmemos",
                requestCode = widgetId
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
        }
        editor.apply()
    }
}
