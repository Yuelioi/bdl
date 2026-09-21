<script setup lang="ts">
import type { MediaPreferences, VideoCodecPreference } from '../../api/dto'
import UiButton from '../../ui/Button.vue'
import UiSelect from '../../ui/Select.vue'
import { audioQualityOptions, codecOptions, videoQualityOptions } from './settingsCatalog'

const model = defineModel<MediaPreferences>({ required: true })
const qualityOptions = [
  { label: '任意 SDR', value: 'sdr' },
  ...videoQualityOptions.filter((option) => option.value !== 'best' && option.value !== 'sdr'),
]
const encodingOptions = codecOptions.map((option) => option.value === 'auto' ? { ...option, label: '不限编码' } : option)
const fallbackOptions = [
  { label: '提示无匹配，不创建任务', value: 'error' },
  { label: '回退到最佳可用', value: 'best' },
]
const updateVideo = (index: number, key: 'quality' | 'codec', value: string) => {
  const video = model.value.video.map((rule) => ({ ...rule }))
  const rule = video[index]
  if (!rule) return
  if (key === 'codec') rule.codec = value as VideoCodecPreference
  else rule.quality = value
  model.value = { ...model.value, video }
}
const updateAudio = (index: number, value: string) => {
  const audio = [...model.value.audio]
  audio[index] = value
  model.value = { ...model.value, audio }
}
const move = (kind: 'video' | 'audio', index: number, offset: number) => {
  const entries = [...model.value[kind]]
  const destination = index + offset
  if (destination < 0 || destination >= entries.length) return
  const entry = entries.splice(index, 1)[0]
  if (entry === undefined) return
  entries.splice(destination, 0, entry)
  model.value = { ...model.value, [kind]: entries }
}
const remove = (kind: 'video' | 'audio', index: number) => {
  model.value = { ...model.value, [kind]: model.value[kind].filter((_, position) => position !== index) }
}
const addVideo = () => {
  model.value = { ...model.value, video: [...model.value.video, { quality: qualityOptions.find((option) => option.value !== 'sdr' && !model.value.video.some((rule) => rule.quality === option.value))?.value ?? '127', codec: 'auto' }] }
}
const addAudio = () => {
  model.value = { ...model.value, audio: [...model.value.audio, 'best'] }
}
</script>

<template>
  <div class="preference-editor">
    <div class="preference-heading">
      <div>
        <h4>画质优先顺序</h4>
      </div>
      <UiButton variant="secondary" size="compact" :disabled="model.video.length >= 32" @click="addVideo">添加画质</UiButton>
    </div>
    <p v-if="!model.video.length" class="preference-empty">尚未自定义，使用上方的视频清晰度和编码设置。</p>
    <ol v-else class="preference-list" aria-label="画质优先顺序">
      <li v-for="(rule, index) in model.video" :key="index" class="preference-row">
        <span class="preference-position" aria-hidden="true">{{ index + 1 }}</span>
        <UiSelect :model-value="rule.quality" :label="`第 ${index + 1} 优先画质`" :options="qualityOptions" @update:model-value="updateVideo(index, 'quality', $event)" />
        <UiSelect :model-value="rule.codec" :label="`第 ${index + 1} 优先编码`" :options="encodingOptions" @update:model-value="updateVideo(index, 'codec', $event)" />
        <div class="preference-actions">
          <UiButton variant="ghost" size="compact" :disabled="index === 0" :aria-label="`上移第 ${index + 1} 条视频偏好`" @click="move('video', index, -1)"><UIcon name="i-tabler-arrow-up" /></UiButton>
          <UiButton variant="ghost" size="compact" :disabled="index === model.video.length - 1" :aria-label="`下移第 ${index + 1} 条视频偏好`" @click="move('video', index, 1)"><UIcon name="i-tabler-arrow-down" /></UiButton>
          <UiButton variant="ghost" size="compact" :aria-label="`删除第 ${index + 1} 条视频偏好`" @click="remove('video', index)"><UIcon name="i-tabler-x" /></UiButton>
        </div>
      </li>
    </ol>

    <div class="preference-heading">
      <div>
        <h4>音频优先顺序</h4>
        <p>独立选择音轨，再与选中的视频合并。</p>
      </div>
      <UiButton variant="secondary" size="compact" :disabled="model.audio.length >= 32" @click="addAudio">添加音质</UiButton>
    </div>
    <p v-if="!model.audio.length" class="preference-empty">尚未自定义，使用上方的音频质量设置。</p>
    <ol v-else class="preference-list" aria-label="音频优先顺序">
      <li v-for="(quality, index) in model.audio" :key="index" class="preference-row preference-audio-row">
        <span class="preference-position" aria-hidden="true">{{ index + 1 }}</span>
        <UiSelect :model-value="quality" :label="`第 ${index + 1} 优先音质`" :options="audioQualityOptions" @update:model-value="updateAudio(index, $event)" />
        <div class="preference-actions">
          <UiButton variant="ghost" size="compact" :disabled="index === 0" :aria-label="`上移第 ${index + 1} 条音频偏好`" @click="move('audio', index, -1)"><UIcon name="i-tabler-arrow-up" /></UiButton>
          <UiButton variant="ghost" size="compact" :disabled="index === model.audio.length - 1" :aria-label="`下移第 ${index + 1} 条音频偏好`" @click="move('audio', index, 1)"><UIcon name="i-tabler-arrow-down" /></UiButton>
          <UiButton variant="ghost" size="compact" :aria-label="`删除第 ${index + 1} 条音频偏好`" @click="remove('audio', index)"><UIcon name="i-tabler-x" /></UiButton>
        </div>
      </li>
    </ol>
    <UiSelect v-if="model.video.length || model.audio.length" :model-value="model.fallback" label="所有偏好都不可用时" :options="fallbackOptions" @update:model-value="model = { ...model, fallback: $event as MediaPreferences['fallback'] }" />
  </div>
</template>

<style scoped>
.preference-editor,
.preference-list {
  display: grid;
  gap: var(--space-12);
}

.preference-heading {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-12);
  margin-top: var(--space-16);
}

.preference-heading h4 {
  margin: 0;
  font-size: var(--font-13);
}

.preference-heading p,
.preference-empty {
  margin: 0;
  color: var(--color-muted);
  font-size: var(--font-12);
  line-height: 1.6;
}

.preference-list {
  margin: 0;
  padding: 0;
  list-style: none;
}

.preference-row {
  display: grid;
  grid-template-columns: 20px minmax(0, 1fr) minmax(0, 1fr) auto;
  align-items: end;
  gap: var(--space-8);
}

.preference-audio-row {
  grid-template-columns: 20px minmax(0, 1fr) auto;
}

.preference-position {
  align-self: end;
  padding-bottom: var(--space-8);
  color: var(--color-muted);
  font-size: var(--font-12);
  font-variant-numeric: tabular-nums;
}

.preference-actions {
  display: flex;
  align-items: center;
}

.preference-actions :deep(button) {
  min-width: 32px;
  padding-inline: var(--space-8);
}

@media (width <= 840px) {
  .preference-row {
    grid-template-columns: 20px minmax(0, 1fr);
  }

  .preference-row > :nth-child(n + 3) {
    grid-column: 2;
  }
}
</style>
