<script setup lang="ts">
import UiButton from '../../ui/Button.vue';
import UiSelect from '../../ui/Select.vue';
import UiTextField from '../../ui/TextField.vue';
import { codecOptions, missingQualityOptions, segmentCountOptions } from './settingsCatalog';
import type { SettingsForm } from './useSettingsForm';

const { form } = defineProps<{ form: SettingsForm }>();
const { settings, settingsCodec, settingsMissingQualityPolicy, settingsSegmentCount, settingsFfmpegPath } = form;
</script>

<template>
  <section class="settings-block">
    <div class="settings-inline-grid">
      <UiSelect v-model="settingsCodec" label="视频编码偏好" :options="codecOptions" />
      <UiSelect v-model="settingsMissingQualityPolicy" label="目标质量不可用" :options="missingQualityOptions" />
    </div>
    <UiSelect v-model="settingsSegmentCount" label="单任务分段数" :options="segmentCountOptions" />
    <div class="directory-row">
      <UiTextField v-model="settingsFfmpegPath" label="FFmpeg 路径" placeholder="留空时使用系统 PATH 中的 ffmpeg" />
      <UiButton variant="secondary" :disabled="settings.loading || settings.saving" @click="settings.chooseFfmpegPath"
        >选择</UiButton
      >
    </div>
    <div v-if="settings.draft.ffmpeg_path" class="settings-actions">
      <UiButton variant="ghost" :disabled="settings.loading || settings.saving" @click="settings.clearFfmpegPath"
        >使用系统 FFmpeg</UiButton
      >
    </div>
    <p class="settings-note">
      编码是偏好而非硬性过滤；目标清晰度不存在时，默认会选择最接近的可用轨道。选择“提示后再处理”时，当前版本会阻止创建任务并显示原因。
    </p>
  </section>
</template>
