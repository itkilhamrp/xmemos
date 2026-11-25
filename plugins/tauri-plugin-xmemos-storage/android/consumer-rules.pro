# Copyright (C) 2026 Xmemos Contributors
# SPDX-License-Identifier: AGPL-3.0-or-later
# Keep Xmemos Android Tauri plugin classes, @Command/@InvokeArg methods, and AppWidgetProvider intact under R8.

-keep class org.xmemos.storage.** { *; }
-keepclassmembers class org.xmemos.storage.** { *; }
-keepattributes *Annotation*, Signature, InnerClasses, EnclosingMethod
-keep @app.tauri.annotation.TauriPlugin class * { *; }
-keep @app.tauri.annotation.InvokeArg class * { *; }
-keepclassmembers class * {
    @app.tauri.annotation.Command <methods>;
}
-dontwarn java.beans.ConstructorProperties
-dontwarn java.beans.Transient
-dontwarn java.beans.**
-dontwarn javax.xml.**
-dontwarn org.w3c.dom.bootstrap.DOMImplementationRegistry
