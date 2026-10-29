<script setup lang="ts">
import SettingsCard from './SettingsCard.vue';
import ErrorDetails from '../../ui/ErrorDetails.vue';
import { recentFailureReport, clearRecentFailures } from '../../utils/recentFailures';
import ClipboardPreference from '../../ui/ClipboardPreference.vue';
import UiButton from '../../ui/Button.vue';
import UiSelect from '../../ui/Select.vue';
import UiTextField from '../../ui/TextField.vue';
import { logLevelOptions, proxyModeOptions } from './settingsCatalog';
import type { SettingsForm } from './useSettingsForm';
import { useFileRemoval } from '../transfer/useFileRemoval';
import FileRemovalDialog from '../transfer/FileRemovalDialog.vue';
import { maintenanceTempPreview } from '../../api/tauri';
import LogMaintenance from './LogMaintenance.vue';

const { form, desktopPaths = true } = defineProps<{ form: SettingsForm; desktopPaths?: boolean }>();
const { settings, settingsProxyMode, settingsProxyDescription, settingsProxyUrl, settingsLogLevel, settingsDataDir } = form;
const cleanup = useFileRemoval('temp', maintenanceTempPreview, (token) => settings.cleanupTemp(token));
</script>

<template>
  <section class="settings-block">
    <SettingsCard title="剪贴板识别">
      <ClipboardPreference />
    </SettingsCard>
    <SettingsCard title="网络与日志">
      <UiSelect v-model="settingsProxyMode" label="下载代理" :options="proxyModeOptions" :helper="settingsProxyDescription" />
      <UiTextField v-if="settingsProxyMode === 'custom'" v-model="settingsProxyUrl" label="代理地址" placeholder="例如 http://127.0.0.1:7890" :error="settings.proxySettingsError ?? undefined" />
      <p class="settings-note">用于任务资源下载{{ desktopPaths ? '与桌面应用更新' : '' }}，保存后新启动的下载或检查生效。</p>
      <UiSelect v-model="settingsLogLevel" label="任务日志级别" :options="logLevelOptions" />
    </SettingsCard>
    <SettingsCard v-if="desktopPaths" title="应用数据">
      <div class="directory-row">
        <UiTextField v-model="settingsDataDir" label="数据目录" placeholder="留空时使用当前工作目录下的 .bdl" />
        <UiButton variant="secondary" :disabled="settings.loading || settings.saving" @click="settings.chooseDataDir"
          >选择</UiButton
        >
      </div>
      <div v-if="desktopPaths && settings.draft.data_dir" class="settings-actions">
        <UiButton variant="ghost" :disabled="settings.loading || settings.saving" @click="settings.clearDataDir"
          >使用默认数据目录</UiButton
        >
      </div>
      <p v-if="desktopPaths" class="settings-note">数据目录影响任务库、账户摘要和维护文件，修改后下次启动生效。</p>
    </SettingsCard>
    <SettingsCard title="清理与诊断">
      <LogMaintenance />
      <details v-if="recentFailureReport" class="min-w-0">
        <summary class="cursor-pointer">最近的操作错误</summary>
        <div class="grid min-w-0 gap-3 py-3">
          <p class="settings-note">保留本次运行最近 20 条错误，退出应用后清空。</p>
          <ErrorDetails :detail="recentFailureReport" :show-advice="false" />
          <UiButton variant="ghost" @click="clearRecentFailures">清空错误记录</UiButton>
        </div>
      </details>
      <div class="maintenance-actions">
        <UiButton
          class="maintenance-action"
          variant="secondary"
          :disabled="settings.loading || settings.saving"
          @click="settings.cleanupCache"
          ><UIcon name="i-tabler-database-off" class="size-4" aria-hidden="true" />清理缓存</UiButton
        >
        <UiButton
          class="maintenance-action"
          variant="secondary"
          :disabled="settings.loading || settings.saving"
          @click="cleanup.show()"
          ><UIcon name="i-tabler-eraser" class="size-4" aria-hidden="true" />清理临时文件</UiButton
        >
        <UiButton
          class="maintenance-action"
          variant="secondary"
          :disabled="settings.loading || settings.saving"
          @click="settings.exportDiagnostics"
          ><UIcon name="i-tabler-file-export" class="size-4" aria-hidden="true" />导出全部诊断</UiButton
        >
      </div>
    </SettingsCard>
    <FileRemovalDialog :controller="cleanup" />
  </section>
</template>

<style scoped>
.maintenance-actions {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(9.25rem, 1fr));
  gap: var(--space-8);
}

.maintenance-action {
  width: 100%;
}
</style>
