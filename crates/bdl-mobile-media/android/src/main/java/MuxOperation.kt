package com.yueli.bdl.media

// Native library loading can throw LinkageError rather than Exception. Convert it at
// the native boundary so the existing cleanup and Tauri rejection paths both run.
internal fun muxWithNativeErrorHandling(operation: () -> Unit) {
  try {
    operation()
  } catch (error: LinkageError) {
    throw IllegalStateException(
      "Android 内置 FFmpeg 加载失败：${error.message ?: error.javaClass.simpleName}",
      error,
    )
  }
}
