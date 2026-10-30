<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import type { LogMaintenanceUsage } from '../../api/dto';
import { maintenanceLogUsage, maintenanceCleanupLogs, maintenanceCleanupDiagnostics, refreshOperationErrors } from '../../api/tauri';
import { useQueueStore } from '../../stores/queue';
import { useUiStore } from '../../stores/ui';
import UiButton from '../../ui/Button.vue';
import UiDialog from '../../ui/Dialog.vue';
import UiInlineNotice from '../../ui/InlineNotice.vue';

const usage = ref<LogMaintenanceUsage | null>(null);
const loading = ref(false);
const busy = ref(false);
const pending = ref<'logs' | 'diagnostics' | null>(null);
const confirmationOpen = computed({ get: () => pending.value !== null, set: (open) => { if (!open && !busy.value) pending.value = null; } });
const error = ref('');
const notice = ref('');
const errorMessage = (cause: unknown) => cause instanceof Error ? cause.message : String(cause);
const size = (bytes: number) => bytes < 1024 ? `${bytes} B` : bytes < 1024 * 1024 ? `${(bytes / 1024).toFixed(1)} KB` : `${(bytes / 1024 / 1024).toFixed(1)} MB`;

const refresh = async () => {
  loading.value = true;
  error.value = '';
  try { usage.value = await maintenanceLogUsage(); await refreshOperationErrors(); }
  catch (cause) { error.value = `读取日志用量失败：${errorMessage(cause)}`; }
  finally { loading.value = false; }
};
const clear = async () => {
  if (!pending.value || busy.value) return;
  busy.value = true;
  error.value = '';
  notice.value = '';
  try {
    if (pending.value === 'logs') {
      const result = await maintenanceCleanupLogs();
      const queue = useQueueStore();
      // Reload from storage instead of keeping cleared failures in the UI cache.
      queue.invalidateLogs(result.task_ids);
      notice.value = `已清理 ${result.removed_logs} 条日志`;
    } else {
      notice.value = `已清理 ${await maintenanceCleanupDiagnostics()} 份诊断文件`;
    }
    pending.value = null;
    await refresh();
  } catch (cause) {
    error.value = `清理失败：${errorMessage(cause)}。请刷新用量后重试。`;
    useUiStore().pushToast(error.value, 'danger');
  } finally { busy.value = false; }
};
onMounted(refresh);
</script>

<template>
  <div class="grid min-w-0 gap-3">
    <div class="flex flex-wrap items-center justify-between gap-2">
      <span>日志与诊断文件</span>
      <UiButton variant="ghost" :disabled="loading || busy" @click="refresh">{{ loading ? '读取中' : '刷新用量' }}</UiButton>
    </div>
    <UiInlineNotice v-if="error" tone="danger">{{ error }}</UiInlineNotice>
    <UiInlineNotice v-if="notice" tone="success">{{ notice }}</UiInlineNotice>
    <div v-if="usage" class="grid min-w-0 gap-3">
      <div class="flex flex-wrap items-center justify-between gap-2">
        <span class="text-(length:--font-13)">日志 {{ usage.log_count }} 条 · {{ size(usage.log_bytes) }}</span>
        <UiButton variant="secondary" :disabled="loading || busy || !usage.cleanable_log_count" @click="pending = 'logs'">清理日志</UiButton>
      </div>
      <p class="settings-note">日志保留 30 天，每个任务最多 1000 条，操作错误最多 200 条。清理会删除操作错误和已结束任务的日志，保留任务记录与下载文件。</p>
      <div class="flex flex-wrap items-center justify-between gap-2">
        <span class="text-(length:--font-13)">诊断文件 {{ usage.diagnostic_files }} 份 · {{ size(usage.diagnostic_bytes) }}</span>
        <UiButton variant="secondary" :disabled="loading || busy || !usage.diagnostic_files" @click="pending = 'diagnostics'">清理诊断文件</UiButton>
      </div>
      <p class="settings-note">导出时保留最近 10 份。只清理应用数据目录中的诊断导出文件。</p>
    </div>
    <UiDialog v-model="confirmationOpen" :title="pending === 'logs' ? '清理日志' : '清理诊断文件'" size="compact" :dismissible="!busy" :show-close="!busy">
      <p v-if="pending === 'logs'">将删除操作错误记录，以及已完成、失败和已取消任务的日志。排队、暂停和运行中的任务日志保留，任务记录和下载文件不受影响。</p>
      <p v-else>将删除应用保存的诊断导出文件。如需反馈问题，请先保留需要的报告。</p>
      <template #footer>
        <UiButton variant="secondary" :disabled="busy" @click="pending = null">取消</UiButton>
        <UiButton variant="danger" :disabled="busy" @click="clear">{{ busy ? '清理中' : '确认清理' }}</UiButton>
      </template>
    </UiDialog>
  </div>
</template>
