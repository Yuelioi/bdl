<script setup lang="ts">
import SettingsCard from './SettingsCard.vue';
import NfoOptionsEditor from './NfoOptionsEditor.vue';
import { createNfoOptions } from '../../utils/nfo';
import { computed, ref } from 'vue';
import type { DownloadWorkflow, WorkflowAssetOutput } from '../../api/dto';
import UiCheckbox from '../../ui/Checkbox.vue';
import UiSelect from '../../ui/Select.vue';
import UiInlineNotice from '../../ui/InlineNotice.vue';
import UiTabs from '../../ui/Tabs.vue';
import { workflowError, workflowOutputs } from '../../utils/downloadWorkflow';
import { isAndroidPlatform } from '../../platform/environment';

const flow = defineModel<DownloadWorkflow>({ required: true });
const { disabled = false } = defineProps<{ disabled?: boolean }>();
const nfoOptions = computed({
  get: () => flow.value.nfo_options ?? createNfoOptions(),
  set: (value) => {
    flow.value.nfo_options = value;
  },
});
const android = isAndroidPlatform();
const output = computed(() => workflowOutputs(flow.value));
const tab = ref('conversion');
const tabs = [
  { label: '转换', value: 'conversion' },
  { label: '封装', value: 'packaging' },
];
const error = computed(() => workflowError(flow.value));
const merge = computed({
  get: () => flow.value.merge_media && flow.value.video.enabled,
  set: (value: boolean) => {
    flow.value.merge_media = value;
    if (!value) {
      for (const rule of [flow.value.video, flow.value.audio]) if (rule.enabled) rule.save = true;
      for (const rule of [flow.value.cover, flow.value.subtitles]) {
        if (rule.embed && !rule.save) rule.save = true;
        rule.embed = false;
      }
    }
  },
});
const mediaEnabled = (kind: 'video' | 'audio') =>
  computed({
    get: () => flow.value[kind].enabled,
    set: (value: boolean) => {
      flow.value[kind].enabled = value;
      if (!flow.value.video.enabled) merge.value = false;
      if (value && !flow.value.merge_media) flow.value[kind].save = true;
    },
  });
const video = mediaEnabled('video');
const audio = mediaEnabled('audio');
const format = computed({
  get: () => flow.value.container,
  set: (value: string) => {
    flow.value.container = value === 'mkv' ? 'mkv' : 'mp4';
    if (value !== 'mkv')
      for (const rule of [flow.value.cover, flow.value.subtitles]) {
        if (rule.embed && !rule.save) rule.save = true;
        rule.embed = false;
      }
  },
});
const assetEnabled = (key: 'cover' | 'subtitles' | 'danmaku') =>
  computed({
    get: () => flow.value[key].enabled,
    set: (value: boolean) => {
      flow.value[key].enabled = value;
      if (!value) flow.value[key].embed = false;
    },
  });
const cover = assetEnabled('cover');
const subtitles = assetEnabled('subtitles');
const danmaku = assetEnabled('danmaku');
const embed = (key: 'cover' | 'subtitles') =>
  computed({
    get: () => flow.value[key].embed,
    set: (value: boolean) => {
      flow.value[key].embed = value;
      if (value) {
        flow.value.merge_media = true;
        flow.value.container = 'mkv';
        if (key === 'subtitles' && flow.value.subtitles.format === 'original') flow.value.subtitles.format = 'srt';
      }
    },
  });
const embedCover = embed('cover');
const embedSubtitles = embed('subtitles');
const videoSave = computed({
  get: () => flow.value.video.save || flow.value.video.retain_original,
  set: (value: boolean) => {
    flow.value.video.save = value;
    flow.value.video.retain_original = false;
  },
});
const subtitleFormat = computed({
  get: () => flow.value.subtitles.format,
  set: (value: WorkflowAssetOutput['format']) => {
    flow.value.subtitles.format = value;
    if (value === 'original') flow.value.subtitles.embed = false;
  },
});
</script>

<template>
  <div class="workflow-editor">
    <section class="workflow-content-selection">
      <h3 class="workflow-section-title">下载内容</h3>
      <div class="grid grid-cols-3 gap-3">
        <UiCheckbox v-model="video" label="下载视频" :disabled="disabled" />
        <UiCheckbox v-model="audio" label="下载音频" :disabled="disabled" />
        <UiCheckbox v-model="subtitles" label="下载字幕" :disabled="disabled" />
        <UiCheckbox v-model="cover" label="下载封面" :disabled="disabled" />
        <UiCheckbox v-model="danmaku" label="下载弹幕" :disabled="disabled" />
        <UiCheckbox v-model="flow.nfo" label="生成 NFO" :disabled="disabled" />
      </div>
    </section>

    <UiSelect
      v-if="video && merge"
      v-model="format"
      label="封装格式"
      :options="[
        { label: 'MP4', value: 'mp4' },
        { label: 'MKV', value: 'mkv' },
      ]"
      :disabled="disabled"
    />
    <section class="workflow-output-options" aria-label="转换与封装">
      <UiTabs v-model="tab" :tabs="tabs" />
      <div class="workflow-tab-content">
        <template v-if="tab === 'conversion'">
          <SettingsCard v-if="video" title="视频">
            <UiCheckbox v-model="videoSave" label="保存独立原始视频（m4s）" :disabled="disabled" />
          </SettingsCard>
          <SettingsCard v-if="audio" title="音频">
            <UiCheckbox v-model="flow.audio.save" label="保存独立音频" :disabled="disabled" />
            <UiSelect
              v-if="flow.audio.save"
              v-model="flow.audio.format"
              label="独立音频格式"
              :options="[
                { label: '不转换（m4s）', value: 'm4s' },
                { label: '转为 MP3', value: 'mp3' },
              ]"
              :disabled="disabled"
            />
            <UiCheckbox
              v-if="!flow.audio.save || flow.audio.format === 'mp3' || flow.audio.retain_original"
              v-model="flow.audio.retain_original"
              label="额外保留原始音频（m4s）"
              :disabled="disabled"
            />
          </SettingsCard>
          <SettingsCard v-if="subtitles" title="字幕">
            <UiSelect
              v-model="subtitleFormat"
              label="字幕格式"
              :options="[
                { label: '不转换（原始 JSON）', value: 'original' },
                { label: 'SRT', value: 'srt' },
                { label: 'ASS', value: 'ass' },
              ]"
              :disabled="disabled"
            />
            <UiCheckbox v-model="flow.subtitles.save" label="保存独立字幕" :disabled="disabled" />
            <UiCheckbox
              v-if="flow.subtitles.format !== 'original'"
              v-model="flow.subtitles.retain_original"
              label="额外保留原始字幕（JSON）"
              :disabled="disabled"
            />
          </SettingsCard>
          <SettingsCard v-if="cover" title="封面"
            ><UiCheckbox v-model="flow.cover.save" label="保存独立封面" :disabled="disabled"
          /></SettingsCard>
          <SettingsCard v-if="danmaku" title="弹幕">
            <UiSelect
              v-model="flow.danmaku.format"
              label="弹幕格式"
              :options="[
                { label: '不转换（XML）', value: 'xml' },
                { label: 'HTML（离线播放）', value: 'html' },
                { label: 'SRT（字幕显示）', value: 'srt' },
                { label: 'ASS（滚动弹幕）', value: 'ass' },
              ]"
              :disabled="disabled"
            />
            <p class="settings-note">
              SRT 按普通字幕显示；ASS 保留滚动和固定位置，需要播放器支持 ASS 特效。高级与脚本弹幕不转换。
            </p>
            <UiCheckbox v-model="flow.danmaku.save" label="保存独立弹幕" :disabled="disabled" />
            <UiCheckbox
              v-if="flow.danmaku.format !== 'xml'"
              v-model="flow.danmaku.retain_original"
              label="额外保留原始弹幕（XML）"
              :disabled="disabled"
            />
          </SettingsCard>
          <NfoOptionsEditor v-if="flow.nfo" v-model="nfoOptions" :disabled="disabled" />
          <p v-if="!video && !audio && !subtitles && !cover && !danmaku && !flow.nfo" class="settings-note">
            当前没有需要转换的内容。
          </p>
        </template>
        <template v-else>
          <template v-if="video">
            <UiCheckbox v-model="merge" label="生成媒体成品" :disabled="disabled" />

            <UiCheckbox
              v-if="subtitles"
              v-model="embedSubtitles"
              label="嵌入字幕"
              :disabled="disabled || (android && !flow.subtitles.embed)"
            />
            <UiCheckbox
              v-if="cover"
              v-model="embedCover"
              label="嵌入封面"
              :disabled="disabled || (android && !flow.cover.embed)"
            />
            <p class="settings-note">嵌入字幕或封面时自动使用 MKV。</p>
            <UiInlineNotice v-if="android && (cover || subtitles)" tone="warning"
              >Android 暂不支持嵌入，请保存独立文件。</UiInlineNotice
            >
          </template>
          <p v-else class="settings-note">没有视频时，按转换设置保存独立文件。</p>
          <div class="rounded-lg border border-(--color-border) p-3" aria-label="预设输出预览">
            <strong class="text-sm">最终输出</strong>
            <p class="m-0 mt-2 text-sm text-(--color-muted)">{{ output.join('、') || '尚未选择输出内容' }}</p>
          </div>
        </template>
      </div>
    </section>
    <UiInlineNotice v-if="error" tone="danger">{{ error }}</UiInlineNotice>
  </div>
</template>

<style scoped>
.workflow-content-selection {
  display: grid;
  gap: 0.75rem;
}

.workflow-section-title {
  margin: 0;
  font-size: var(--font-14);
  font-weight: 500;
}

.workflow-editor,
.workflow-output-options,
.workflow-tab-content {
  min-width: 0;
  display: grid;
  align-content: start;
  gap: var(--space-16);
}
</style>
