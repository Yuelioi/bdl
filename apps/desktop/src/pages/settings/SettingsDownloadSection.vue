<script setup lang="ts">
import ParseRulesEditor from './ParseRulesEditor.vue';
import UiButton from '../../ui/Button.vue';
import UiCheckbox from '../../ui/Checkbox.vue';
import UiIconButton from '../../ui/IconButton.vue';
import UiSelect from '../../ui/Select.vue';
import UiTextField from '../../ui/TextField.vue';
import SettingsEnvironmentSummary from './SettingsEnvironmentSummary.vue';
import { concurrentTaskOptions, retryCountOptions } from './settingsCatalog';
import type { SettingsForm } from './useSettingsForm';

const { form, desktopPaths = true } = defineProps<{ form: SettingsForm; desktopPaths?: boolean }>();
const {
  settings,
  settingsDownloadDir,
  settingsConcurrentTasks,
  settingsRetryCount,
  settingsGlobalSpeedLimitMib,
  settingsGlobalSpeedLimitError,
  updateGlobalSpeedLimit,
  settingsAutoRefreshExpiredUrls,
  settingsStartupAutoRecovery,
} = form;
</script>

<template>
  <section class="settings-block">
    <div v-if="!desktopPaths" class="directory-row">
      <UiTextField :model-value="settings.draft.document_tree_output?.display_name ?? ''" label="Android 导出目录" placeholder="尚未选择" disabled />
      <UiIconButton
        icon="folder-open"
        :label="settings.draft.document_tree_output ? '更换导出目录' : '选择导出目录'"
        :disabled="settings.loading || settings.saving"
        @click="settings.chooseDocumentTreeOutput"
      />
    </div>
    <div v-if="desktopPaths" class="directory-row">
      <UiTextField v-model="settingsDownloadDir" label="保存目录" placeholder="未设置时使用 downloads" />
      <UiButton variant="secondary" :disabled="settings.loading || settings.saving" @click="settings.chooseDownloadDir">
        选择
      </UiButton>
    </div>
    <div class="settings-inline-grid">
      <UiSelect v-model="settingsConcurrentTasks" label="同时下载任务数" :options="concurrentTaskOptions" />
      <UiSelect v-model="settingsRetryCount" label="失败自动重试次数" :options="retryCountOptions" />
    </div>
    <UiTextField
      :model-value="settingsGlobalSpeedLimitMib"
      label="全局下载限速（MiB/s）"
      placeholder="留空时不限速"
      :error="settingsGlobalSpeedLimitError ?? undefined"
      @update:model-value="updateGlobalSpeedLimit"
    />
    <div class="settings-toggle-list">
      <UiCheckbox
        v-model="settingsAutoRefreshExpiredUrls"
        label="链接过期时自动刷新"
        :disabled="settings.loading || settings.saving"
      />
      <UiCheckbox
        v-model="settingsStartupAutoRecovery"
        label="启动时自动继续未完成任务"
        :disabled="settings.loading || settings.saving"
      />
    </div>
    <ParseRulesEditor v-model="settings.draft.parse_rules" />
    <SettingsEnvironmentSummary
      v-if="desktopPaths"
      :health="settings.environmentHealth"
      :checking="settings.environmentChecking"
      @check="settings.checkEnvironment"
      @choose-ffmpeg="settings.chooseFfmpegPath"
      @use-system-ffmpeg="settings.clearFfmpegPath"
    />
  </section>
</template>

<style scoped>
.directory-row {
  min-width: 0;
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  align-items: end;
  gap: var(--space-8);
}
</style>
