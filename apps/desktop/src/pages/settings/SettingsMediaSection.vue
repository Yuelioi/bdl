<script setup lang="ts">
import { computed } from 'vue';
import MediaPreferenceEditor from './MediaPreferenceEditor.vue';
import UiInlineNotice from '../../ui/InlineNotice.vue';
import UiSelect from '../../ui/Select.vue';
import { audioQualityOptions, outputFormatOptions, videoQualityOptions } from './settingsCatalog';
import type { SettingsForm } from './useSettingsForm';

const { form } = defineProps<{ form: SettingsForm }>();
const { settingsVideoQuality, settingsAudioQuality, settingsOutputFormat, settingsEmbeddingFormatError } = form;
const preferences = computed({
  get: () => form.settings.draft.media_preferences,
  set: (value) => form.settings.setMediaPreferences(value),
});
</script>

<template>
  <section class="settings-block">
    <div class="settings-inline-grid">
      <UiSelect v-model="settingsVideoQuality" label="视频清晰度" :options="videoQualityOptions" :disabled="preferences.video.length > 0" />
      <UiSelect v-model="settingsAudioQuality" label="音频质量" :options="audioQualityOptions" :disabled="preferences.audio.length > 0" />
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
    <MediaPreferenceEditor v-model="preferences" />
  </section>
</template>
