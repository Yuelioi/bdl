package com.yueli.bdl.execution

import android.Manifest
import android.app.Activity
import android.content.Intent
import android.os.Build
import android.webkit.WebView
import androidx.core.view.ViewCompat
import androidx.core.view.WindowInsetsCompat
import androidx.work.Data
import androidx.work.ExistingWorkPolicy
import androidx.work.OneTimeWorkRequest
import androidx.work.WorkManager
import app.tauri.annotation.Command
import app.tauri.annotation.InvokeArg
import app.tauri.annotation.Permission
import app.tauri.annotation.TauriPlugin
import app.tauri.plugin.Invoke
import app.tauri.plugin.JSObject
import app.tauri.plugin.Plugin
import java.util.concurrent.TimeUnit

@InvokeArg
class ExecutionArgs {
  var active: Boolean = false
}

@InvokeArg
class ScheduleWakeupsArgs {
  var scheduledAtUnixMillis: List<Long> = emptyList()
}

@TauriPlugin(
  permissions = [
    Permission(
      strings = [Manifest.permission.POST_NOTIFICATIONS],
      alias = "notifications",
    ),
  ],
)
class TaskExecutionPlugin(private val activity: Activity) : Plugin(activity) {
  override fun load(webView: WebView) {
    super.load(webView)
    webView.addOnLayoutChangeListener { view, _, _, _, _, _, _, _, _ ->
      syncSystemInsets(view as WebView)
    }
    webView.post {
      syncSystemInsets(webView)
    }
  }

  private fun syncSystemInsets(webView: WebView) {
    val rootInsets = ViewCompat.getRootWindowInsets(webView) ?: return
    val insets = rootInsets.getInsets(
      WindowInsetsCompat.Type.systemBars() or WindowInsetsCompat.Type.displayCutout(),
    )
    val density = webView.resources.displayMetrics.density.takeIf { it > 0f } ?: 1f
    val top = insets.top / density
    val right = insets.right / density
    val bottom = insets.bottom / density
    val left = insets.left / density
    val script = """
      (() => {
        const root = document.documentElement;
        root.style.setProperty('--bdl-safe-area-top', '${top}px');
        root.style.setProperty('--bdl-safe-area-right', '${right}px');
        root.style.setProperty('--bdl-safe-area-bottom', '${bottom}px');
        root.style.setProperty('--bdl-safe-area-left', '${left}px');
      })();
    """.trimIndent()
    webView.evaluateJavascript(script, null)
  }

  @Command
  fun setActive(invoke: Invoke) {
    val args = try {
      invoke.parseArgs(ExecutionArgs::class.java)
    } catch (error: Exception) {
      invoke.reject(error.message ?: "后台任务参数无效")
      return
    }

    try {
      val context = activity.applicationContext
      val intent = Intent(context, DownloadForegroundService::class.java)
      if (args.active) {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
          context.startForegroundService(intent)
        } else {
          context.startService(intent)
        }
      } else {
        context.stopService(intent)
      }
      invoke.resolve(JSObject())
    } catch (error: Exception) {
      invoke.reject(error.message ?: "无法切换 Android 后台下载服务")
    }
  }

  @Command
  fun syncScheduledWakeups(invoke: Invoke) {
    val args = try {
      invoke.parseArgs(ScheduleWakeupsArgs::class.java)
    } catch (error: Exception) {
      invoke.reject(error.message ?: "定时任务唤醒参数无效")
      return
    }

    try {
      val workManager = WorkManager.getInstance(activity.applicationContext)
      workManager.cancelAllWorkByTag(SCHEDULED_WAKEUP_TAG)
      args.scheduledAtUnixMillis.distinct().sorted().forEach { scheduledAt ->
        val delayMillis = (scheduledAt - System.currentTimeMillis()).coerceAtLeast(0L)
        val request = OneTimeWorkRequest.Builder(ScheduledTaskWorker::class.java)
          .setInitialDelay(delayMillis, TimeUnit.MILLISECONDS)
          .setInputData(
            Data.Builder()
              .putLong(ScheduledTaskWorker.INPUT_SCHEDULED_AT, scheduledAt)
              .build(),
          )
          .addTag(SCHEDULED_WAKEUP_TAG)
          .build()
        workManager.enqueueUniqueWork(
          "$SCHEDULED_WAKEUP_WORK_PREFIX$scheduledAt",
          ExistingWorkPolicy.REPLACE,
          request,
        )
      }
      invoke.resolve(JSObject())
    } catch (error: Exception) {
      invoke.reject(error.message ?: "无法同步 Android 定时任务唤醒")
    }
  }

  companion object {
    private const val SCHEDULED_WAKEUP_TAG = "bdl-scheduled-task-wakeup"
    private const val SCHEDULED_WAKEUP_WORK_PREFIX = "bdl-scheduled-task-wakeup-"
  }
}
