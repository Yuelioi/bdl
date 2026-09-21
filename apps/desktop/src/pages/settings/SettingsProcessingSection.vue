<script setup lang="ts">
import UiButton from '../../ui/Button.vue';
import UiSelect from '../../ui/Select.vue';
import UiTextField from '../../ui/TextField.vue';
import { codecOptions, missingQualityOptions, segmentCountOptions } from './settingsCatalog';
import type { SettingsForm } from './useSettingsForm';

const { form, desktopPaths = true } = defineProps<{ form: SettingsForm; desktopPaths?: boolean }>();
const { settings, settingsCodec, settingsMissingQualityPolicy, settingsSegmentCount, settingsFfmpegPath } = form;
</script>

<template>
  <section class="settings-block">
    <div class="settings-inline-grid">
      <UiSelect v-model="settingsCodec" label="视频编码偏好" :options="codecOptions" :disabled="settings.draft.media_preferences.video.length > 0" :helper="settings.draft.media_preferences.video.length ? '已使用媒体设置中的组合编码' : undefined" />
      <UiSelect v-model="settingsMissingQualityPolicy" label="目标质量不可用" :options="missingQualityOptions" />
    </div>
    <UiSelect v-model="settingsSegmentCount" label="单任务分段数" :options="segmentCountOptions" />
    <div v-if="desktopPaths" class="directory-row">
      <UiTextField
        v-model="settingsFfmpegPath"
        label="FFmpeg 路径"
        placeholder="留空时自动发现系统或常见包管理器中的 ffmpeg"
      />
      <UiButton variant="secondary" :disabled="settings.loading || settings.saving" @click="settings.chooseFfmpegPath"
        >选择</UiButton
      >
    </div>
    <div v-if="desktopPaths && settings.draft.ffmpeg_path" class="settings-actions">
      <UiButton variant="ghost" :disabled="settings.loading || settings.saving" @click="settings.clearFfmpegPath"
        >使用系统 FFmpeg</UiButton
      >
    </div>

  </section>
</template>
