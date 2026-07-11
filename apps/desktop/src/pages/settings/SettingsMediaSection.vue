<script setup lang="ts">
import UiInlineNotice from '../../ui/InlineNotice.vue';
import UiSelect from '../../ui/Select.vue';
import { audioQualityOptions, outputFormatOptions, videoQualityOptions } from './settingsCatalog';
import type { SettingsForm } from './useSettingsForm';

const { form } = defineProps<{ form: SettingsForm }>();
const { settingsVideoQuality, settingsAudioQuality, settingsOutputFormat, settingsEmbeddingFormatError } = form;
</script>

<template>
  <section class="settings-block">
    <div class="settings-inline-grid">
      <UiSelect v-model="settingsVideoQuality" label="视频清晰度" :options="videoQualityOptions" />
      <UiSelect v-model="settingsAudioQuality" label="音频质量" :options="audioQualityOptions" />
    </div>
    <UiSelect
      v-model="settingsOutputFormat"
      label="封装格式"
      helper="嵌入封面和字幕时需使用 MKV"
      :options="outputFormatOptions"
    />
    <UiInlineNotice v-if="settingsEmbeddingFormatError" tone="danger">
      {{ settingsEmbeddingFormatError }}
    </UiInlineNotice>
  </section>
</template>
