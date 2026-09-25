# JavaCPP loads presets by reflection and its prebuilt JNI libraries look up Java
# classes/members by name. Keeping native method names alone is insufficient.
# Ship these rules with the plugin so regenerated Tauri projects also consume them.
-keepattributes *Annotation*,InnerClasses,EnclosingMethod
# Only runtime classes, not the desktop-only org.bytedeco.javacpp.tools package.
-keep class org.bytedeco.javacpp.* { *; }
-keep class org.bytedeco.javacpp.annotation.** { *; }
-keep class org.bytedeco.ffmpeg.** { *; }
# Optional OSGi package metadata is not used by the Android runtime.
-dontwarn org.osgi.annotation.bundle.Export
