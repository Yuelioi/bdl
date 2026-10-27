<script setup lang="ts">
import { computed, onDeactivated, ref } from 'vue';
import UiSelect, { type SelectOption } from '../../ui/Select.vue';
import UiButton from '../../ui/Button.vue';
import MediaPreferenceEditor from './MediaPreferenceEditor.vue';
import {
  audioQualityOptions,
  audioCodecOptions as audioCodecCatalog,
  codecOptions,
  videoQualityOptions,
  videoQualityLevels,
} from './settingsCatalog';
import type { SettingsForm } from './useSettingsForm';
import type { AudioCodecPreference, VideoCodecPreference } from '../../api/dto';

const { form } = defineProps<{ form: SettingsForm }>();
const { settings, settingsCodec, settingsMissingQualityPolicy } = form;
const disabled = computed(() => settings.loading || settings.saving);
const editors = ref({ video: false, audio: false, videoCodec: false, audioCodec: false });
onDeactivated(() => {
  editors.value = { video: false, audio: false, videoCodec: false, audioCodec: false };
});
const videoOrdered = computed(() => settings.draft.media_preferences.video.length > 0);
const audioOrdered = computed(() => settings.draft.media_preferences.audio.length > 0);
const videoCodecOrdered = computed(() => Boolean(settings.draft.media_preferences.video_codecs?.length));
const audioCodecOrdered = computed(() => Boolean(settings.draft.media_preferences.audio_codecs?.length));
const videoEditorVisible = computed(() => editors.value.video && videoOrdered.value);
const audioEditorVisible = computed(() => editors.value.audio && audioOrdered.value);
const videoCodecVisible = computed(() => editors.value.videoCodec && videoCodecOrdered.value);
const audioCodecVisible = computed(() => editors.value.audioCodec && audioCodecOrdered.value);
const videoCodecLevels = codecOptions.filter((option) => option.value !== 'auto');
const audioLevels = audioQualityOptions.filter((option) => option.value !== 'best');
const rankedOptions = (options: readonly SelectOption[], order: readonly string[]) =>
  [...options].sort((a, b) => {
    const aRank = order.indexOf(a.value);
    const bRank = order.indexOf(b.value);
    return (aRank < 0 ? order.length : aRank) - (bRank < 0 ? order.length : bRank);
  });
const completeOrder = (options: readonly SelectOption[], order: readonly string[]) => [
  ...order,
  ...options.filter((option) => !order.includes(option.value)).map((option) => option.value),
];
const toggleVideoEditor = () => {
  editors.value.video = !videoEditorVisible.value;
  if (!editors.value.video) return;
  const rules = settings.draft.media_preferences.video;
  if (!rules.length) {
    const levels = videoQualityLevels.map((option) => ({ quality: option.value, codec: 'auto' as const }));
    const first = levels.find((rule) => rule.quality === settings.draft.quality);
    settings.draft.media_preferences.video = first ? [first, ...levels.filter((rule) => rule !== first)] : levels;
  } else {
    const missing = videoQualityLevels.filter((option) => !rules.some((rule) => rule.quality === option.value));
    if (missing.length)
      settings.draft.media_preferences.video = [
        ...rules,
        ...missing.map((option) => ({ quality: option.value, codec: 'auto' as const })),
      ];
  }
};
const toggleAudioEditor = () => {
  editors.value.audio = !audioEditorVisible.value;
  if (editors.value.audio) {
    const current = settings.draft.media_preferences.audio;
    const first = settings.draft.audio_quality;
    settings.draft.media_preferences.audio = completeOrder(
      audioLevels,
      current.length ? current : first === 'best' ? [] : [first],
    );
  }
};
const toggleVideoCodec = () => {
  editors.value.videoCodec = !videoCodecVisible.value;
  if (editors.value.videoCodec) {
    const current = settings.draft.media_preferences.video_codecs ?? [];
    settings.draft.media_preferences.video_codecs = completeOrder(
      videoCodecLevels,
      current.length ? current : settings.draft.codec === 'auto' ? [] : [settings.draft.codec],
    ) as Exclude<VideoCodecPreference, 'auto'>[];
  }
};
const toggleAudioCodec = () => {
  editors.value.audioCodec = !audioCodecVisible.value;
  if (editors.value.audioCodec)
    settings.draft.media_preferences.audio_codecs = completeOrder(
      audioCodecCatalog,
      settings.draft.media_preferences.audio_codecs ?? [],
    ) as AudioCodecPreference[];
};
const videoCodecOptions = computed(() => [
  codecOptions[0]!,
  ...rankedOptions(videoCodecLevels, settings.draft.media_preferences.video_codecs ?? []),
]);
const audioCodecOptions = computed(() => [
  { label: '自动', value: 'auto' },
  ...rankedOptions(audioCodecCatalog, settings.draft.media_preferences.audio_codecs ?? []),
]);
const videoCodec = computed({
  get: () => settings.draft.media_preferences.video_codecs?.[0] ?? settingsCodec.value,
  set: (value: string) => {
    settingsCodec.value = 'auto';
    settings.draft.media_preferences.video_codecs =
      value === 'auto'
        ? []
        : (completeOrder(videoCodecLevels, [
            value,
            ...(settings.draft.media_preferences.video_codecs ?? []).filter((codec) => codec !== value),
          ]) as Exclude<VideoCodecPreference, 'auto'>[]);
  },
});
const audioCodec = computed({
  get: () => settings.draft.media_preferences.audio_codecs?.[0] ?? 'auto',
  set: (value: string) => {
    settings.draft.media_preferences.audio_codecs =
      value === 'auto'
        ? []
        : (completeOrder(audioCodecCatalog, [
            value,
            ...(settings.draft.media_preferences.audio_codecs ?? []).filter((codec) => codec !== value),
          ]) as AudioCodecPreference[]);
  },
});
const videoOptions = computed(() => [
  ...videoQualityOptions.filter((option) => ['best', 'sdr'].includes(option.value)),
  ...rankedOptions(
    videoQualityLevels,
    settings.draft.media_preferences.video.map((rule) => rule.quality),
  ),
]);
const audioOptions = computed(() => [
  audioQualityOptions[0]!,
  ...rankedOptions(audioLevels, settings.draft.media_preferences.audio),
]);
const video = computed({
  get: () => settings.draft.quality,
  set: (value: string) => {
    const wasAutomatic = ['best', 'sdr'].includes(settings.draft.quality);
    settings.setVideoQuality(value);
    if (wasAutomatic && !['best', 'sdr'].includes(value)) settings.setMissingQualityPolicy('skip');
  },
});
const audio = computed({
  get: () => settings.draft.audio_quality,
  set: (value: string) => {
    const wasAutomatic = settings.draft.audio_quality === 'best';
    settings.setAudioQuality(value);
    if (wasAutomatic && value !== 'best') settings.setMissingQualityPolicy('skip');
  },
});
const needsMissingPolicy = computed(
  () => !['best', 'sdr'].includes(settings.draft.quality) || settings.draft.audio_quality !== 'best',
);
const missingPolicy = computed({
  get: () => (settingsMissingQualityPolicy.value === 'lower' ? 'lower' : 'skip'),
  set: (value: string) => {
    settingsMissingQualityPolicy.value = value;
  },
});
const policyOptions = [
  { label: '不换档（推荐）', value: 'skip' },
  { label: '允许自动换档', value: 'lower' },
];
const videoLabel = (quality: string) =>
  videoQualityOptions.find((option) => option.value === quality)?.label ?? quality;
const audioLabel = (quality: string) =>
  audioQualityOptions.find((option) => option.value === quality)?.label ?? quality;
const videoSummary = computed(() => {
  if (settings.draft.quality === 'best') return videoOrdered.value ? '最佳可用按编排顺序选择。' : '';
  if (settings.draft.quality === 'sdr')
    return videoOrdered.value
      ? '按编排顺序选择 SDR，不选择 HDR 或杜比视界。'
      : '选择最高可用 SDR，不选择 HDR 或杜比视界。';
  return missingPolicy.value === 'lower'
    ? '优先 ' + videoLabel(settings.draft.quality) + '；没有时自动使用其他可用档位，可能降到 480P 或更低。'
    : '仅下载 ' + videoLabel(settings.draft.quality) + '；没有时不创建任务。';
});
const audioSummary = computed(() => {
  if (settings.draft.audio_quality === 'best') return audioOrdered.value ? '音质按编排顺序选择。' : '';
  return missingPolicy.value === 'lower'
    ? '优先 ' + audioLabel(settings.draft.audio_quality) + '；没有时允许使用其他可用音质。'
    : '仅下载 ' + audioLabel(settings.draft.audio_quality) + '；没有时不创建任务。';
});
</script>

<template>
  <div class="quality-section">
    <p class="settings-note">画质和音质设置用于所有下载预设。</p>
    <div class="settings-inline-grid settings-quality-options">
      <div class="quality-choice">
        <UiSelect v-model="video" label="视频清晰度" :options="videoOptions" :disabled="disabled" />
        <UiButton
          variant="ghost"
          size="compact"
          aria-label="编排清晰度优先级"
          :aria-pressed="videoEditorVisible"
          :disabled="disabled"
          @click="toggleVideoEditor"
          >编排</UiButton
        >
      </div>
      <div class="quality-choice">
        <UiSelect v-model="videoCodec" label="视频编码" :options="videoCodecOptions" :disabled="disabled" />
        <UiButton
          variant="ghost"
          size="compact"
          aria-label="编排视频编码优先级"
          :aria-pressed="videoCodecVisible"
          :disabled="disabled"
          @click="toggleVideoCodec"
          >编排</UiButton
        >
      </div>
      <div class="quality-choice">
        <UiSelect v-model="audio" label="音频质量" :options="audioOptions" :disabled="disabled" />
        <UiButton
          variant="ghost"
          size="compact"
          aria-label="编排音质优先级"
          :aria-pressed="audioEditorVisible"
          :disabled="disabled"
          @click="toggleAudioEditor"
          >编排</UiButton
        >
      </div>
      <div class="quality-choice">
        <UiSelect v-model="audioCodec" label="音频编码" :options="audioCodecOptions" :disabled="disabled" />
        <UiButton
          variant="ghost"
          size="compact"
          aria-label="编排音频编码优先级"
          :aria-pressed="audioCodecVisible"
          :disabled="disabled"
          @click="toggleAudioCodec"
          >编排</UiButton
        >
      </div>
      <UiSelect
        v-if="needsMissingPolicy"
        v-model="missingPolicy"
        label="指定档位不可用时"
        :options="policyOptions"
        :disabled="disabled"
      />
    </div>
    <div
      v-if="videoSummary || audioSummary || videoCodecOrdered || audioCodecOrdered"
      class="quality-summary"
      role="status"
      aria-live="polite"
    >
      <p v-if="videoSummary">{{ videoSummary }}</p>
      <p v-if="audioSummary">{{ audioSummary }}</p>
      <p v-if="videoCodecOrdered || audioCodecOrdered">先选画质或音质，同档位按编码顺序选择。</p>
    </div>
    <MediaPreferenceEditor
      v-if="videoEditorVisible || audioEditorVisible || videoCodecVisible || audioCodecVisible"
      v-model="settings.draft.media_preferences"
      :video="videoEditorVisible"
      :audio="audioEditorVisible"
      :video-encoding="videoCodecVisible"
      :audio-encoding="audioCodecVisible"
      :disabled="disabled"
    />
  </div>
</template>

<style scoped>
.quality-section {
  min-width: 0;
  display: grid;
  gap: var(--space-12);
}

.quality-summary {
  padding: var(--space-10) var(--space-12);
  border-radius: var(--radius-6);
  background: var(--color-inset);
  color: var(--color-muted);
  font-size: var(--font-12);
  line-height: 1.6;
}

.quality-summary p {
  margin: 0;
}

.quality-summary p + p {
  margin-top: var(--space-4);
}

.quality-choice {
  min-width: 0;
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  align-items: end;
  gap: var(--space-4);
}

.quality-choice > :deep(button) {
  margin-bottom: var(--space-4);
}
</style>
