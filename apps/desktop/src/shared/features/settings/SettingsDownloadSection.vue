<script setup lang="ts">
import { computed, ref } from 'vue';
import SettingsCard from './SettingsCard.vue';
import SettingsOutputDefaults from './SettingsOutputDefaults.vue';
import ParseRulesEditor from './ParseRulesEditor.vue';
import UiButton from '../../ui/Button.vue';
import UiSelect from '../../ui/Select.vue';
import UiTextField from '../../ui/TextField.vue';
import SettingsEnvironmentSummary from './SettingsEnvironmentSummary.vue';
import FfmpegDownloadButton from '../../ui/FfmpegDownloadButton.vue';
import { concurrentTaskOptions, globalSpeedLimitOptions, retryCountOptions, segmentCountOptions, taskStartIntervalOptions } from './settingsCatalog';
import type { SettingsForm } from './useSettingsForm';

const { form, desktopPaths = true, group } = defineProps<{
  form: SettingsForm;
  desktopPaths?: boolean;
  group: 'saving' | 'tasks' | 'links';
}>();
const {
  settings,
  settingsEnvironmentHealth,
  draftChecking,
  draftCheckError,
  checkDraftEnvironment,
  settingsDownloadDir,
  settingsConcurrentTasks,
  taskStartInterval,
  taskStartIntervalError,
  updateTaskStartInterval,
  settingsRetryCount,
  settingsSegmentCount,
  settingsFfmpegPath,
  settingsGlobalSpeedLimitMb,
  settingsGlobalSpeedLimitError,
  updateGlobalSpeedLimit,
  settingsAutoRefreshExpiredUrls,
  settingsStartupAutoRecovery,
} = form;
const customSpeedSelected = ref(false);
const customIntervalSelected = ref(false);
const taskIntervalPreset = computed({
  get: () => {
    const value = taskStartInterval.value.trim();
    return !customIntervalSelected.value && taskStartIntervalOptions.some(option => option.value === value)
      ? value
      : 'custom';
  },
  set: (value: string) => {
    customIntervalSelected.value = value === 'custom';
    if (value !== 'custom') updateTaskStartInterval(value);
  },
});
const speedLimitPreset = computed({
  get: () => {
    if (customSpeedSelected.value) return 'custom';
    const value = settingsGlobalSpeedLimitMb.value.trim();
    if (!value) return 'unlimited';
    return globalSpeedLimitOptions.some((option) => option.value === value && option.value !== 'unlimited')
      ? value
      : 'custom';
  },
  set: (value: string) => {
    customSpeedSelected.value = value === 'custom';
    if (value !== 'custom') updateGlobalSpeedLimit(value === 'unlimited' ? '' : value);
  },
});
</script>

<template>
  <section class="settings-block">
    <SettingsCard v-if="group === 'saving'" :title="desktopPaths ? undefined : '保存位置'">
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
    <SettingsOutputDefaults v-if="group === 'saving'" :form="form" />
    <SettingsCard v-if="group === 'tasks'" title="速度与并发">
      <div class="settings-inline-grid settings-download-options">
        <UiSelect v-model="settingsConcurrentTasks" label="同时下载任务数" :options="concurrentTaskOptions" tall-menu>
          <template #value>{{ settingsConcurrentTasks }}</template>
        </UiSelect>
        <UiSelect v-model="settingsSegmentCount" label="单任务分段数" :options="segmentCountOptions" />
      </div>
      <div class="settings-inline-grid settings-download-options">
        <div class="grid min-w-0 content-start gap-2">
          <UiSelect v-model="taskIntervalPreset" label="任务间隔" :options="taskStartIntervalOptions" />
          <UiTextField
            v-if="taskIntervalPreset === 'custom'"
            :model-value="taskStartInterval"
            label="自定义间隔（秒）"
            inputmode="numeric"
            :error="taskStartIntervalError ?? undefined"
            @update:model-value="updateTaskStartInterval"
          />
        </div>
        <div class="grid min-w-0 content-start gap-2">
          <UiSelect v-model="speedLimitPreset" label="下载总限速" :options="globalSpeedLimitOptions" tall-menu />
          <UiTextField
            v-if="speedLimitPreset === 'custom'"
            :model-value="settingsGlobalSpeedLimitMb"
            label="全局下载限速（MB/s）"
            placeholder="留空时不限速"
            :error="settingsGlobalSpeedLimitError ?? undefined"
            @update:model-value="updateGlobalSpeedLimit"
          />
        </div>
      </div>
      <p class="settings-note">下载任务共享带宽上限。合并单独排队，每次处理 1 个。</p>
    </SettingsCard>
    <SettingsCard v-if="group === 'tasks'" title="重试与续传">
      <div class="settings-inline-grid settings-download-options">
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
    <ParseRulesEditor v-if="group === 'links'" v-model="settings.draft.parse_rules" />
    <SettingsCard v-if="desktopPaths && group === 'saving'" id="settings-ffmpeg" tabindex="-1" title="FFmpeg">
      <div class="directory-row">
        <UiTextField v-model="settingsFfmpegPath" label="FFmpeg 路径" placeholder="留空时使用系统 FFmpeg" />
        <UiButton variant="secondary" :disabled="settings.loading || settings.saving" @click="settings.chooseFfmpegPath"
          >选择</UiButton>
      </div>
      <SettingsEnvironmentSummary
        :health="settingsEnvironmentHealth"
        :checking="draftChecking"
        @check="checkDraftEnvironment"
        @choose-ffmpeg="settings.chooseFfmpegPath"
        @use-system-ffmpeg="settings.clearFfmpegPath"
      />
      <p v-if="draftCheckError" class="settings-error">{{ draftCheckError }}</p>
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
    </SettingsCard>
  </section>
</template>

<style scoped>
.mobile-directory-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: var(--space-8);
  overflow-wrap: anywhere;
}

</style>
