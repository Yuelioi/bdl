<script setup lang="ts">
import SettingsCard from './SettingsCard.vue';
import ParseRulesEditor from './ParseRulesEditor.vue';
import UiButton from '../../ui/Button.vue';
import UiCheckbox from '../../ui/Checkbox.vue';
import UiIconButton from '../../ui/IconButton.vue';
import UiSelect from '../../ui/Select.vue';
import UiTextField from '../../ui/TextField.vue';
import SettingsEnvironmentSummary from './SettingsEnvironmentSummary.vue';
import { concurrentTaskOptions, retryCountOptions, segmentCountOptions } from './settingsCatalog';
import type { SettingsForm } from './useSettingsForm';

const { form, desktopPaths = true } = defineProps<{ form: SettingsForm; desktopPaths?: boolean }>();
const {
  settings,
  settingsEnvironmentHealth,
  draftChecking,
  draftCheckError,
  checkDraftEnvironment,
  settingsDownloadDir,
  settingsConcurrentTasks,
  settingsRetryCount,
  settingsSegmentCount,
  settingsFfmpegPath,
  settingsGlobalSpeedLimitMb,
  settingsGlobalSpeedLimitError,
  updateGlobalSpeedLimit,
  settingsAutoRefreshExpiredUrls,
  settingsStartupAutoRecovery,
} = form;
</script>

<template>
  <section class="settings-block">
    <SettingsCard title="保存位置">
      <div v-if="!desktopPaths" class="directory-row">
        <UiTextField
          :model-value="settings.draft.document_tree_output?.display_name ?? ''"
          label="Android 导出目录"
          placeholder="尚未选择"
          disabled
        />
        <UiIconButton
          icon="folder-open"
          :label="settings.draft.document_tree_output ? '更换导出目录' : '选择导出目录'"
          :disabled="settings.loading || settings.saving"
          @click="settings.chooseDocumentTreeOutput"
        />
      </div>
      <div v-if="desktopPaths" class="directory-row">
        <UiTextField v-model="settingsDownloadDir" label="保存目录" placeholder="未设置时使用 downloads" />
        <UiButton
          variant="secondary"
          :disabled="settings.loading || settings.saving"
          @click="settings.chooseDownloadDir"
        >
          选择
        </UiButton>
      </div>
    </SettingsCard>
    <SettingsCard title="下载与恢复">
      <div class="settings-inline-grid">
        <UiSelect v-model="settingsConcurrentTasks" label="同时下载任务数" :options="concurrentTaskOptions" />
        <UiSelect v-model="settingsRetryCount" label="失败自动重试次数" :options="retryCountOptions" />
        <UiSelect v-model="settingsSegmentCount" label="单任务分段数" :options="segmentCountOptions" />
        <UiTextField
          :model-value="settingsGlobalSpeedLimitMb"
          label="全局下载限速（MB/s）"
          placeholder="留空时不限速"
          :error="settingsGlobalSpeedLimitError ?? undefined"
          @update:model-value="updateGlobalSpeedLimit"
        />
      </div>
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
    </SettingsCard>
    <ParseRulesEditor v-model="settings.draft.parse_rules" />
    <SettingsCard v-if="desktopPaths" title="FFmpeg">
      <div class="directory-row">
        <UiTextField v-model="settingsFfmpegPath" label="FFmpeg 路径" placeholder="留空时使用系统 FFmpeg" />
        <UiButton variant="secondary" :disabled="settings.loading || settings.saving" @click="settings.chooseFfmpegPath"
          >选择</UiButton
        >
      </div>
      <SettingsEnvironmentSummary
        :health="settingsEnvironmentHealth"
        :checking="draftChecking"
        @check="checkDraftEnvironment"
        @choose-ffmpeg="settings.chooseFfmpegPath"
        @use-system-ffmpeg="settings.clearFfmpegPath"
      />
      <p v-if="draftCheckError" class="settings-error">{{ draftCheckError }}</p>
    </SettingsCard>
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
