package com.yueli.bdl.execution

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.os.Build
import androidx.work.Worker
import androidx.work.WorkerParameters

class ScheduledTaskWorker(
  appContext: Context,
  workerParams: WorkerParameters,
) : Worker(appContext, workerParams) {
  override fun doWork(): Result {
    val manager = applicationContext.getSystemService(NotificationManager::class.java)
    createNotificationChannel(manager)
    val scheduledAt = inputData.getLong(INPUT_SCHEDULED_AT, 0L)
    manager.notify(notificationId(scheduledAt), buildNotification(scheduledAt))
    return Result.success()
  }

  private fun createNotificationChannel(manager: NotificationManager) {
    if (Build.VERSION.SDK_INT < Build.VERSION_CODES.O) return
    manager.createNotificationChannel(
      NotificationChannel(
        CHANNEL_ID,
        "定时下载",
        NotificationManager.IMPORTANCE_DEFAULT,
      ).apply {
        description = "BDL 定时下载到点提醒"
      },
    )
  }

  @Suppress("DEPRECATION")
  private fun buildNotification(scheduledAt: Long): Notification {
    val launchIntent = applicationContext.packageManager
      .getLaunchIntentForPackage(applicationContext.packageName)
      ?.apply {
        addFlags(Intent.FLAG_ACTIVITY_CLEAR_TOP or Intent.FLAG_ACTIVITY_SINGLE_TOP)
      }
    val contentIntent = launchIntent?.let {
      PendingIntent.getActivity(
        applicationContext,
        0,
        it,
        PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE,
      )
    }
    val builder = if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
      Notification.Builder(applicationContext, CHANNEL_ID)
    } else {
      Notification.Builder(applicationContext)
    }
    return builder
      .setSmallIcon(R.drawable.ic_bdl_download)
      .setContentTitle("BDL 定时任务已到时间")
      .setContentText("打开 BDL 后将按队列状态开始下载")
      .setWhen(if (scheduledAt > 0L) scheduledAt else System.currentTimeMillis())
      .setShowWhen(true)
      .setCategory(Notification.CATEGORY_REMINDER)
      .setAutoCancel(true)
      .setContentIntent(contentIntent)
      .build()
  }

  private fun notificationId(scheduledAt: Long): Int {
    if (scheduledAt <= 0L) return FALLBACK_NOTIFICATION_ID
    val folded = (scheduledAt xor (scheduledAt ushr 32)).toInt() and Int.MAX_VALUE
    return if (folded == 0) FALLBACK_NOTIFICATION_ID else folded
  }

  companion object {
    const val INPUT_SCHEDULED_AT = "scheduled_at_unix_millis"
    private const val CHANNEL_ID = "bdl_scheduled_tasks"
    private const val FALLBACK_NOTIFICATION_ID = 1202
  }
}
