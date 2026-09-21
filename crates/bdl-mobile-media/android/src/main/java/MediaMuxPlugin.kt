package com.yueli.bdl.media

import android.app.Activity
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import java.io.File
import java.util.concurrent.Executors

@InvokeArg
class MuxArgs {
  var videoPath: String? = null
  var audioPath: String? = null
  lateinit var outputPath: String
  lateinit var formatName: String
}

@TauriPlugin
class MediaMuxPlugin(private val activity: Activity) : Plugin(activity) {
  private val executor = Executors.newSingleThreadExecutor()

  @Command
  fun mux(invoke: Invoke) {
    val args = try {
      invoke.parseArgs(MuxArgs::class.java)
    } catch (error: Exception) {
      invoke.reject(error.message ?: "媒体合并参数无效")
      return
    }

    executor.execute {
      try {
        muxMedia(args)
        invoke.resolve(JSObject())
      } catch (error: Exception) {
        invoke.reject(error.message ?: "媒体合并失败")
      }
    }
  }

  private fun muxMedia(args: MuxArgs) {
    if (args.videoPath == null && args.audioPath == null) {
      throw IllegalArgumentException("缺少视频或音频输入")
    }

    val output = File(args.outputPath)
    output.parentFile?.let { parent ->
      if (!parent.exists() && !parent.mkdirs()) {
        throw IllegalStateException("无法创建媒体输出目录")
      }
    }
    val temporary = File(output.parentFile, "${output.name}.bdl-mux.tmp")
    val backup = File(output.parentFile, "${output.name}.bdl-mux.backup")
    if (temporary.exists() && !temporary.delete()) {
      throw IllegalStateException("无法清理上次未完成的媒体文件")
    }
    if (backup.exists()) {
      if (output.exists()) {
        if (!backup.delete()) {
          throw IllegalStateException("无法清理上次遗留的媒体备份")
        }
      } else if (!backup.renameTo(output)) {
        throw IllegalStateException("无法恢复上次媒体合并留下的备份")
      }
    }

    try {
      try {
        FfmpegMuxer.mux(args.videoPath, args.audioPath, temporary.absolutePath, args.formatName)
      } catch (error: Exception) {
        throw IllegalStateException(
          "Android 内置 FFmpeg 合并失败：${error.message ?: error.javaClass.simpleName}",
          error,
        )
      }

      if (output.exists() && !output.renameTo(backup)) {
        throw IllegalStateException("无法保护已有媒体文件")
      }
      if (!temporary.renameTo(output)) {
        if (backup.exists()) {
          backup.renameTo(output)
        }
        throw IllegalStateException("无法提交合并后的媒体文件")
      }
      backup.delete()
    } catch (error: Exception) {
      temporary.delete()
      if (!output.exists() && backup.exists()) {
        backup.renameTo(output)
      }
      throw error
    }
  }
}
