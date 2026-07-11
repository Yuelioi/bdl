<script setup lang="ts">
import UiCheckbox from '../../ui/Checkbox.vue';
import UiInlineNotice from '../../ui/InlineNotice.vue';
import UiSelect from '../../ui/Select.vue';
import { archiveModeOptions } from './settingsCatalog';
import type { SettingsForm } from './useSettingsForm';

const { form } = defineProps<{ form: SettingsForm }>();
const {
  settings,
  settingsArchiveMode,
  settingsArchiveCover,
  settingsArchiveSubtitles,
  settingsArchiveDanmaku,
  settingsArchiveNfo,
  settingsRetainRawStreams,
  settingsEmbedCover,
  settingsEmbedSubtitles,
  settingsEmbeddingFormatError,
  settingsArchiveDescription,
} = form;
</script>

<template>
  <section class="settings-block">
    <UiSelect v-model="settingsArchiveMode" label="下载范围" :options="archiveModeOptions" />
    <div v-if="settings.draft.archive_mode === 'custom'" class="archive-option-grid">
      <UiCheckbox v-model="settingsArchiveCover" label="保存封面" :disabled="settings.loading || settings.saving" />
      <UiCheckbox v-model="settingsArchiveSubtitles" label="保存字幕" :disabled="settings.loading || settings.saving" />
      <UiCheckbox v-model="settingsArchiveDanmaku" label="保存弹幕" :disabled="settings.loading || settings.saving" />
      <UiCheckbox v-model="settingsArchiveNfo" label="生成 NFO" :disabled="settings.loading || settings.saving" />
    </div>
    <div class="archive-option-grid archive-single-grid">
      <UiCheckbox
        v-model="settingsRetainRawStreams"
        label="保留原始视频/音频轨道"
        :disabled="settings.loading || settings.saving"
      />
    </div>
    <div class="archive-option-grid embed-option-grid">
      <UiCheckbox
        v-model="settingsEmbedCover"
        label="嵌入封面（仅 MKV）"
        :disabled="settings.loading || settings.saving"
      />
      <UiCheckbox
        v-model="settingsEmbedSubtitles"
        label="嵌入字幕（仅 MKV）"
        :disabled="settings.loading || settings.saving"
      />
    </div>
    <UiInlineNotice v-if="settingsEmbeddingFormatError" tone="danger">{{
      settingsEmbeddingFormatError
    }}</UiInlineNotice>
    <p class="settings-note">{{ settingsArchiveDescription }}</p>
  </section>
</template>
