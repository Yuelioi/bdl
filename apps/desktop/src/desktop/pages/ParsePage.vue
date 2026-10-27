<script setup lang="ts">
import UiInlineNotice from '../../shared/ui/InlineNotice.vue';
import ParseSourceInput from '../../shared/features/parse/ParseSourceInput.vue';
import ParseDownloadPlanner from '../../shared/features/parse/ParseDownloadPlanner.vue';
import ParseBatchWorkspace from '../components/parse/ParseBatchWorkspace.vue';
import ParseResultWorkspace from '../components/parse/ParseResultWorkspace.vue';
import { useParsePage } from '../../shared/features/parse/useParsePage';
const {
  parse,
  inputMode,
  singleInput,
  createLoading,
  activeStage,
  submitInput,
  pasteSingleInput,
  pasteInput,
  downloadSelected,
  runNoticeAction,
} = useParsePage();
</script>

<template>
  <section class="page-grid grid-cols-1">
    <section class="panel parse-panel min-h-0 overflow-hidden bg-(--color-surface)">
      <ParseSourceInput
        v-model:mode="inputMode"
        v-model:input="parse.input"
        v-model:single-input="singleInput"
        :busy="createLoading"
        :has-results="activeStage === 'content'"
        @submit="submitInput"
        @paste="pasteInput"
        @single-paste="pasteSingleInput"
      />
      <UiInlineNotice
        v-if="parse.notice"
        :tone="parse.notice.tone"
        :detail="parse.notice.detail"
        :action-label="parse.notice.actionLabel"
        @action="runNoticeAction"
        >{{ parse.notice.message }}</UiInlineNotice
      >
      <template v-if="activeStage === 'content'">
        <ParseBatchWorkspace v-if="parse.isBatch" embedded inline-download @download="downloadSelected" />
        <ParseResultWorkspace v-else embedded inline-download @download="downloadSelected" />
      </template>
      <div v-else class="parse-entry-hint">
        <UIcon name="i-tabler-link" aria-hidden="true" />
        <h2>从一个链接开始</h2>
        <p>视频、合集、收藏夹、UP 空间、番剧或课程</p>
        <p>多条链接可分行粘贴，解析后选择需要的内容。</p>
      </div>
      <ParseDownloadPlanner ref="download-planner" :inline="activeStage === 'content'" />
    </section>
  </section>
</template>

<style scoped>
.parse-panel {
  gap: 1rem;
  padding: 0;
}

.parse-entry-hint {
  display: grid;
  flex: 1;
  align-content: center;
  justify-items: center;
  gap: 0.5rem;
  min-height: 0;
  overflow-y: auto;
  color: var(--color-muted);
  text-align: center;
}

.parse-entry-hint > :deep(svg) {
  width: 1.75rem;
  height: 1.75rem;
}

.parse-entry-hint h2 {
  margin: 0;
  font-size: var(--font-18);
  font-weight: 600;
  color: var(--color-text);
}

.parse-entry-hint p {
  margin: 0;
  font-size: var(--font-13);
  line-height: 1.6;
}
</style>
