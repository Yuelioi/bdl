# WorkManager's Room database is instantiated by reflection during AndroidX Startup.
-keepclassmembers class androidx.work.impl.WorkDatabase_Impl {
    <init>();
}
