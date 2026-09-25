<script setup lang="ts">
import UiButton from '../../ui/Button.vue';
import UiInlineNotice from '../../ui/InlineNotice.vue';

import ParseDownloadPlanner from '../parse/ParseDownloadPlanner.vue';
import ParseBatchWorkspace from './ParseBatchWorkspace.vue';
import ParseResultWorkspace from './ParseResultWorkspace.vue';
import { useParsePage } from '../useParsePage';
const {
  parse,
  inputMode,
  singleInput,
  createLoading,
  activeStage,
  submitInput,
  switchInputMode,
  pasteSingleInput,
  pasteInput,
  openDownloadSettings,
  runNoticeAction,
} = useParsePage();
</script>
<template>
  <section class="mobile-page mobile-parse-page">
    <UiInlineNotice
      v-if="parse.notice"
      :tone="parse.notice.tone"
      :action-label="parse.notice.actionLabel"
      @action="runNoticeAction"
      >{{ parse.notice.message }}</UiInlineNotice
    >
    <template v-if="activeStage === 'source'">
      <div class="parse-intro">
        <h3>把喜欢的视频留下来</h3>
        <p>复制 B 站分享链接，粘贴到这里。</p>
      </div>
      <form class="mobile-parse-form" @submit.prevent="submitInput">
        <div class="parse-input-top">
          <label for="mobile-source-input">视频链接</label
          ><UiButton variant="ghost" :disabled="createLoading" @click="pasteInput"
            ><UIcon name="i-tabler-clipboard" />粘贴</UiButton
          >
        </div>
        <textarea
          v-if="inputMode === 'batch'"
          id="mobile-source-input"
          v-model="parse.input"
          aria-label="Bilibili 链接或 BV / AV"
          placeholder="粘贴分享文字、b23 短链或 BV 号…"
          :disabled="createLoading"
          rows="5"
        ></textarea>
        <textarea
          v-else
          id="mobile-source-input"
          v-model="singleInput"
          aria-label="视频链接或 BV / AV"
          placeholder="粘贴一个视频链接…"
          :disabled="createLoading"
          rows="5"
          @paste.prevent="pasteSingleInput"
        ></textarea>
        <label class="parse-behavior"
          ><span>解析范围</span
          ><select
            :value="inputMode"
            :disabled="createLoading"
            @change="switchInputMode(($event.target as HTMLSelectElement).value as 'single' | 'batch')"
          >
            <option value="batch">视频及所属合集</option>
            <option value="single">仅当前视频</option>
          </select></label
        >
        <UiButton class="parse-primary" type="submit" :disabled="createLoading"
          >{{ createLoading ? '正在解析…' : '开始解析' }}<UIcon name="i-tabler-arrow-right"
        /></UiButton>
      </form>
      <p class="parse-help">支持视频、合集、收藏夹和 UP 空间。多个链接可分行粘贴。</p>
    </template>
    <template v-else
      ><ParseBatchWorkspace v-if="parse.isBatch" embedded @download="openDownloadSettings" /><ParseResultWorkspace
        v-else
        embedded
        @download="openDownloadSettings"
    /></template>
    <ParseDownloadPlanner ref="download-planner" />
  </section>
</template>
<style scoped>
.mobile-parse-page {
  overflow-y: auto;
}

.mobile-parse-page:has(.mobile-result-workspace) {
  overflow: hidden;
}

.parse-intro {
  padding: 4px 0;
}

.parse-intro h3 {
  margin: 0;
  font-size: 18px;
  font-weight: 650;
  line-height: 1.5;
}

.parse-intro p {
  margin: 4px 0 0;
  color: var(--color-muted);
  font-size: 14px;
}

.mobile-parse-form {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.parse-input-top {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 14px;
}

.parse-input-top :deep(button) {
  min-height: 44px;
}

.parse-input-top :deep(svg),
.parse-primary :deep(svg) {
  width: 20px;
  height: 20px;
  flex-shrink: 0;
}

textarea {
  width: 100%;
  min-height: 120px;
  resize: vertical;
  border: 1px solid var(--color-border);
  border-radius: 12px;
  background: var(--color-panel);
  color: var(--color-text);
  padding: 12px;
  font-size: 14px;
  line-height: 1.65;
}

textarea:focus {
  outline: 2px solid var(--color-accent);
  outline-offset: 2px;
}

.parse-behavior {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  color: var(--color-muted);
  font-size: 14px;
}

select {
  border: 0;
  min-height: 44px;
  max-width: 65%;
  background: var(--color-surface);
  color: var(--color-text);
}

.parse-primary {
  width: 100%;
  min-height: 50px;
  border-radius: 14px;
  justify-content: space-between;
  padding-inline: 18px;
  font-size: 16px;
}

.parse-help {
  margin: 0;
  color: var(--color-muted);
  font-size: 12px;
  line-height: 1.7;
}
</style>
