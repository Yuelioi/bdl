<script setup lang="ts">
import { computed } from 'vue';
import type { MediaPreferences, VideoCodecPreference, AudioCodecPreference } from '../../api/dto';
import CodecPriorityEditor from './CodecPriorityEditor.vue';
import UiButton from '../../ui/Button.vue';
import UiSelect from '../../ui/Select.vue';
import { audioQualityOptions, audioCodecOptions, codecOptions, videoQualityOptions } from './settingsCatalog';

const model = defineModel<MediaPreferences>({ required: true });
const {
  video = true,
  audio = true,
  videoEncoding = true,
  audioEncoding = true,
  disabled = false,
} = defineProps<{
  video?: boolean;
  audio?: boolean;
  videoEncoding?: boolean;
  audioEncoding?: boolean;
  disabled?: boolean;
}>();
const qualityOptions = videoQualityOptions;
const hasCombinationCodecs = computed(() => model.value.video.some((rule) => rule.codec !== 'auto'));
const videoCodecs = computed({
  get: () => model.value.video_codecs ?? [],
  set: (value: string[]) => {
    model.value = { ...model.value, video_codecs: value as Exclude<VideoCodecPreference, 'auto'>[] };
  },
});
const audioCodecs = computed({
  get: () => model.value.audio_codecs ?? [],
  set: (value: string[]) => {
    model.value = { ...model.value, audio_codecs: value as AudioCodecPreference[] };
  },
});
const encodingOptions = codecOptions.map((option) =>
  option.value === 'auto' ? { ...option, label: '不限编码' } : option,
);
const updateVideoCodec = (index: number, value: string) => {
  const video = model.value.video.map((rule) => ({ ...rule }));
  const rule = video[index];
  if (!rule) return;
  rule.codec = value as VideoCodecPreference;
  model.value = { ...model.value, video };
};
const move = (kind: 'video' | 'audio', index: number, offset: number) => {
  const entries = [...model.value[kind]];
  const destination = index + offset;
  if (destination < 0 || destination >= entries.length) return;
  const entry = entries.splice(index, 1)[0];
  if (entry === undefined) return;
  entries.splice(destination, 0, entry);
  model.value = { ...model.value, [kind]: entries };
};
</script>

<template>
  <div class="preference-editor">
    <div v-if="video || audio" class="preference-quality-grid" :class="{ paired: video && audio }">
      <section v-if="video" class="preference-group">
        <div class="preference-heading">
          <div class="preference-heading-copy">
            <span class="preference-heading-icon" aria-hidden="true">
              <UIcon name="i-tabler-video" />
            </span>
            <div>
              <h4>清晰度优先级</h4>
              <p v-if="hasCombinationCodecs">此配置还限定了每档画质的编码。</p>
            </div>
          </div>
        </div>
        <ol class="preference-list" aria-label="画质优先顺序">
          <li
            v-for="(rule, index) in model.video"
            :key="index"
            class="preference-row"
            :class="{ 'preference-simple-row': !hasCombinationCodecs }"
          >
            <span class="preference-position" aria-hidden="true">{{ index + 1 }}</span>
            <span class="preference-quality">{{
              qualityOptions.find((option) => option.value === rule.quality)?.label ?? rule.quality
            }}</span>
            <UiSelect
              v-if="hasCombinationCodecs"
              hide-label
              :model-value="rule.codec"
              :label="`第 ${index + 1} 优先编码`"
              :options="encodingOptions"
              :disabled="disabled"
              @update:model-value="updateVideoCodec(index, $event)"
            />
            <div class="preference-actions">
              <UiButton
                variant="ghost"
                size="compact"
                :disabled="disabled || index === 0"
                :aria-label="`上移第 ${index + 1} 条视频偏好`"
                @click="move('video', index, -1)"
                ><UIcon name="i-tabler-arrow-up"
              /></UiButton>
              <UiButton
                variant="ghost"
                size="compact"
                :disabled="disabled || index === model.video.length - 1"
                :aria-label="`下移第 ${index + 1} 条视频偏好`"
                @click="move('video', index, 1)"
                ><UIcon name="i-tabler-arrow-down"
              /></UiButton>
            </div>
          </li>
        </ol>
      </section>

      <section v-if="audio" class="preference-group">
        <div class="preference-heading">
          <div class="preference-heading-copy">
            <span class="preference-heading-icon" aria-hidden="true">
              <UIcon name="i-tabler-volume" />
            </span>
            <div>
              <h4>音质优先级</h4>
            </div>
          </div>
        </div>
        <ol class="preference-list" aria-label="音频优先顺序">
          <li v-for="(quality, index) in model.audio" :key="index" class="preference-row preference-audio-row">
            <span class="preference-position" aria-hidden="true">{{ index + 1 }}</span>
            <span class="preference-quality">{{
              audioQualityOptions.find((option) => option.value === quality)?.label ?? quality
            }}</span>
            <div class="preference-actions">
              <UiButton
                variant="ghost"
                size="compact"
                :disabled="disabled || index === 0"
                :aria-label="`上移第 ${index + 1} 条音频偏好`"
                @click="move('audio', index, -1)"
                ><UIcon name="i-tabler-arrow-up"
              /></UiButton>
              <UiButton
                variant="ghost"
                size="compact"
                :disabled="disabled || index === model.audio.length - 1"
                :aria-label="`下移第 ${index + 1} 条音频偏好`"
                @click="move('audio', index, 1)"
                ><UIcon name="i-tabler-arrow-down"
              /></UiButton>
            </div>
          </li>
        </ol>
      </section>
    </div>
    <div
      v-if="(videoEncoding && videoCodecs.length) || (audioEncoding && audioCodecs.length)"
      class="preference-codec-grid"
      :class="{ paired: videoEncoding && videoCodecs.length && audioEncoding && audioCodecs.length }"
    >
      <CodecPriorityEditor
        v-if="videoEncoding && videoCodecs.length"
        v-model="videoCodecs"
        label="视频编码"
        :options="codecOptions.filter((option) => option.value !== 'auto')"
        :disabled="disabled"
      />
      <CodecPriorityEditor
        v-if="audioEncoding && audioCodecs.length"
        v-model="audioCodecs"
        label="音频编码"
        :options="audioCodecOptions"
        :disabled="disabled"
      />
    </div>
  </div>
</template>

<style scoped>
.preference-editor,
.preference-quality-grid,
.preference-codec-grid,
.preference-list {
  display: grid;
  gap: var(--space-12);
}

.paired {
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--space-16);
}

.preference-quality-grid:not(.paired) {
  max-width: 32rem;
}

.preference-group {
  min-width: 0;
  display: grid;
  align-content: start;
  gap: var(--space-12);
  padding-top: var(--space-12);
  border-top: 1px solid var(--color-border);
}

.preference-heading {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-12);
}

.preference-heading-copy {
  min-width: 0;
  display: flex;
  align-items: center;
  gap: var(--space-10);
}

.preference-heading-icon {
  width: 2rem;
  height: 2rem;
  display: grid;
  flex: 0 0 auto;
  place-items: center;
  border: 0.0625rem solid var(--color-border);
  border-radius: var(--radius-6);
  background: var(--color-surface);
  color: var(--color-muted);
}

.preference-heading-icon svg {
  width: 1.0625rem;
  height: 1.0625rem;
}

.preference-heading h4 {
  margin: 0;
  font-size: var(--font-13);
}

.preference-heading p {
  margin: 0;
  color: var(--color-muted);
  font-size: var(--font-12);
  line-height: 1.6;
}

.preference-heading-copy > div {
  min-width: 0;
  display: grid;
  gap: 0.125rem;
}

.preference-list {
  gap: var(--space-4);
  margin: 0;
  padding: 0;
  list-style: none;
}

.preference-row {
  display: grid;
  grid-template-columns: 1.25rem minmax(0, 1fr) minmax(0, 1fr) auto;
  align-items: end;
  gap: var(--space-8);
  padding-block: var(--space-2);
}

.preference-audio-row {
  grid-template-columns: 1.25rem minmax(0, 1fr) auto;
}

.preference-simple-row {
  grid-template-columns: 1.25rem minmax(0, 1fr) auto;
}

.preference-position {
  align-self: end;
  padding-bottom: var(--space-8);
  color: var(--color-muted);
  font-size: var(--font-12);
  font-variant-numeric: tabular-nums;
}

.preference-quality {
  align-self: center;
  min-width: 0;
  padding-block: var(--space-6);
  font-size: var(--font-13);
}

.preference-actions {
  display: flex;
  align-items: center;
}

.preference-actions :deep(button) {
  min-width: 2rem;
  padding-inline: var(--space-8);
}

@media (width <= 840px) {
  .paired {
    grid-template-columns: minmax(0, 1fr);
  }

  .preference-heading {
    align-items: flex-start;
  }

  .preference-row {
    grid-template-columns: 1.25rem repeat(2, minmax(0, 1fr));
  }

  .preference-row > .preference-actions {
    grid-column: 2 / -1;
    justify-self: end;
  }

  .preference-audio-row {
    grid-template-columns: 1.25rem minmax(0, 1fr) auto;
  }

  .preference-simple-row {
    grid-template-columns: 1.25rem minmax(0, 1fr) auto;
  }

  .preference-audio-row > .preference-actions,
  .preference-simple-row > .preference-actions {
    grid-column: auto;
  }
}
</style>
