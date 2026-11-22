<script setup lang="ts">
import SettingsCard from './SettingsCard.vue';
import SettingsOutputDefaults from './SettingsOutputDefaults.vue';
import ParseRulesEditor from './ParseRulesEditor.vue';
import UiButton from '../../ui/Button.vue';
import UiSelect from '../../ui/Select.vue';
import UiTextField from '../../ui/TextField.vue';
import SettingsEnvironmentSummary from './SettingsEnvironmentSummary.vue';
import FfmpegDownloadButton from '../../ui/FfmpegDownloadButton.vue';
import { concurrentTaskOptions, retryCountOptions, segmentCountOptions } from './settingsCatalog';
import type { SettingsForm } from './useSettingsForm';

const { form, desktopPaths = true, group } = defineProps<{
  form: SettingsForm;
  desktopPaths?: boolean;
  group?: 'saving' | 'tasks' | 'links';
}>();
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
    <SettingsCard v-if="!group || group === 'saving'" title="保存位置">
      <div v-if="!desktopPaths" class="mobile-directory-row">
        <span>{{ settings.draft.document_tree_output?.display_name ?? '尚未选择目录' }}</span>
        <UiButton
          variant="secondary"
          :disabled="settings.loading || settings.saving"
          @click="settings.chooseDocumentTreeOutput"
        >
          <UIcon name="i-tabler-folder-open" class="size-4" aria-hidden="true" />
          {{ settings.draft.document_tree_output ? '更换目录' : '选择目录' }}
        </UiButton>
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
    <SettingsOutputDefaults v-if="!group || group === 'saving'" :form="form" />
    <SettingsCard v-if="!group || group === 'tasks'" :title="group ? '速度与并发' : '下载与恢复'">
      <div class="settings-inline-grid settings-download-options">
        <UiSelect v-model="settingsConcurrentTasks" label="同时下载任务数" :options="concurrentTaskOptions" />
        <UiSelect v-model="settingsSegmentCount" label="单任务分段数" :options="segmentCountOptions" />
        <UiSelect v-if="!group" v-model="settingsRetryCount" label="失败自动重试次数" :options="retryCountOptions" />
        <UiTextField
          :model-value="settingsGlobalSpeedLimitMb"
          label="全局下载限速（MB/s）"
          placeholder="留空时不限速"
          :error="settingsGlobalSpeedLimitError ?? undefined"
          @update:model-value="updateGlobalSpeedLimit"
        />
      </div>
      <h3 v-if="group" class="settings-subheading">重试与续传</h3>
      <div v-if="group" class="settings-inline-grid settings-download-options">
        <UiSelect v-model="settingsRetryCount" label="失败自动重试次数" :options="retryCountOptions" />
      </div>
      <div class="settings-toggle-list">
        <label class="settings-toggle-row"
          ><span>链接过期时自动刷新</span
          ><USwitch
            v-model="settingsAutoRefreshExpiredUrls"
            aria-label="链接过期时自动刷新"
            :disabled="settings.loading || settings.saving"
        /></label>
        <label class="settings-toggle-row"
          ><span>启动时继续未完成任务</span
          ><USwitch
            v-model="settingsStartupAutoRecovery"
            aria-label="启动时自动继续未完成任务"
            :disabled="settings.loading || settings.saving"
        /></label>
      </div>
    </SettingsCard>
    <ParseRulesEditor v-if="!group || group === 'links'" v-model="settings.draft.parse_rules" />
    <SettingsCard v-if="desktopPaths && !group" title="FFmpeg">
      <details class="text-xs leading-5 text-(--color-muted)">
        <summary
          class="cursor-pointer rounded-sm text-(--color-text) border border-transparent focus-visible:border-(--color-accent) focus-visible:outline-none"
        >
          如何配置 FFmpeg
        </summary>
        <div class="mt-3 grid justify-items-start gap-2">
          <p class="m-0">FFmpeg 用于合并视频与音频、转换格式。可从月离软件站下载适合当前系统的版本。</p>
          <FfmpegDownloadButton />
          <ol class="m-0 grid list-decimal gap-2 pl-5">
            <li>
              下载后先解压到固定位置。点击“选择”，选中 bin 文件夹内的 ffmpeg.exe 文件（macOS / Linux 为
              ffmpeg），不要选择文件夹或压缩包。
            </li>
            <li>
              如果已安装 FFmpeg 并加入系统 PATH 环境变量，路径留空即可自动查找。刚修改 PATH 时，请重启 BDL 后再检查。
            </li>
            <li>点击下方“重新检查”，显示“就绪”后点击页面上的“保存”。保存后请保留 FFmpeg 文件及其所在目录。</li>
          </ol>
        </div>
      </details>
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

.mobile-directory-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: var(--space-8);
  overflow-wrap: anywhere;
}

.settings-subheading {
  margin: var(--space-8) 0 0;
  color: var(--color-text-strong);
  font-size: var(--font-14);
  font-weight: 600;
}
</style>
