<script setup lang="ts">
import { computed } from 'vue';
import UiInlineNotice from '../../shared/ui/InlineNotice.vue';
import UiButton from '../../shared/ui/Button.vue';
import ParseSourceInput from '../../shared/features/parse/ParseSourceInput.vue';
import ParseDownloadPlanner from '../../shared/features/parse/ParseDownloadPlanner.vue';
import ParseBatchWorkspace from '../components/parse/ParseBatchWorkspace.vue';
import ParseResultWorkspace from '../components/parse/ParseResultWorkspace.vue';
import MobileDownloadScope from '../components/MobileDownloadScope.vue';
import { sourcePartCount } from '../../shared/features/parse/parseResultTree';
import { useParsePage } from '../../shared/features/parse/useParsePage';
import { useMobileBack } from '../app/useMobileBack';
const {
  parse,
  inputMode,
  singleInput,
  createLoading,
  activeStage,
  submittedInput,
  returnToInput,
  pasteInput,
  downloadSelected,
  openAllDownloadSettings,
  runNoticeAction,
} = useParsePage();

const sourceDetails = computed(() => parse.activeSource?.source);
const canDownloadAll = computed(() => {
  const source = parse.activeSource;
  return !parse.isBatch && !!source && (source.source.has_more || sourcePartCount(source) > 1);
});
const downloadAllBusy = computed(() => {
  const id = sourceDetails.value?.id;
  return Boolean(id && (parse.loadingBySource[id] || parse.pacedParsingBySource[id]));
});
const downloadAll = () => {
  const source = sourceDetails.value;
  if (!source || downloadAllBusy.value) return;
  openAllDownloadSettings(source.id);
};

const parseSource = async () => {
  if (createLoading.value) return;
  const input = parse.input;
  if (await parse.createSource(undefined, { expandVideoCollection: false })) {
    const source = parse.activeSource;
    if (!parse.isBatch && source?.source.kind === 'video' && sourcePartCount(source) === 1) {
      parse.selectAllLoaded(source.source.id);
    }
    submittedInput.value = input;
    activeStage.value = 'content';
    if (!parse.isBatch && source?.source.has_more) await parse.loadConfiguredBatch(source.source.id, 'first');
  }
};
const openCollection = async () => {
  const source = sourceDetails.value;
  if (source?.kind !== 'video' || createLoading.value) return;
  const selection = [...parse.activeSelection];
  const draft = parse.input;
  if (!(await parse.createSource(source.input))) return;
  if (!parse.input) parse.input = draft;
  if (parse.activeSource?.source.id === source.id) {
    parse.selectPartIds(source.id, selection, 'replace');
    parse.setNotice('这个视频没有可查看的所属合集。', 'info');
  }
};
useMobileBack('parse', () => {
  if (activeStage.value !== 'content') return false;
  returnToInput();
  return true;
});
</script>

<template>
  <section class="mobile-page mobile-parse-page" :class="{ 'parse-entry': activeStage !== 'content' }">
    <header v-if="activeStage === 'content'" class="mobile-detail-heading">
      <UButton
        variant="ghost"
        color="neutral"
        class="mobile-icon-button parse-back-button"
        aria-label="返回解析首页"
        @click="returnToInput"
      >
        <UIcon name="i-tabler-arrow-left" />
      </UButton>
      <div class="mobile-detail-copy"><h2>{{ parse.isBatch ? '批量解析结果' : '解析结果' }}</h2></div>
    </header>
    <template v-else>
      <header class="mobile-page-heading parse-entry-heading">
        <h2>解析链接</h2>
        <UiButton
          v-if="parse.activeSource"
          class="parse-results-button"
          variant="ghost"
          :disabled="createLoading"
          @click="activeStage = 'content'"
        >查看结果</UiButton>
      </header>
      <p class="parse-entry-hint">支持视频、合集、收藏夹、UP 主空间、番剧和课程。</p>
      <ParseSourceInput
        v-model:mode="inputMode"
        v-model:input="parse.input"
        v-model:single-input="singleInput"
        :busy="createLoading"
        :paused="Boolean(parse.createPause)"
        :pausing="Boolean(parse.createPause && parse.operationsBySource.__create__?.length)"
        @pause="parse.pauseCreation()"
        @resume="parse.resumeCreation()"
        mobile
        @submit="parseSource"
        @paste="pasteInput"
      />
    </template>
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
      <ParseResultWorkspace v-else embedded inline-download @download="downloadSelected" @open-collection="openCollection" />
    </template>
    <ParseDownloadPlanner ref="download-planner" :inline="activeStage === 'content'">
      <template v-if="canDownloadAll" #download-scope="{ busy }">
        <MobileDownloadScope
          :disabled="busy || downloadAllBusy"
          @download="downloadAll"
        />
      </template>
    </ParseDownloadPlanner>
  </section>
</template>

<style scoped>
.mobile-parse-page {
  gap: var(--space-12);
  padding: var(--space-12) var(--space-16) var(--space-8);
  overflow: hidden;
}

.mobile-parse-page.parse-entry {
  overflow-y: auto;
}

.parse-entry-heading {
  min-height: 2.75rem;
}

.parse-results-button {
  min-height: 2.75rem;
}

.parse-back-button {
  flex-basis: 2.75rem;
  width: 2.75rem;
  height: 2.75rem;
}

.parse-entry-hint {
  margin: 0 0 var(--space-4);
  color: var(--color-muted);
  font-size: var(--font-13);
  line-height: 1.6;
}

@media (orientation: landscape) and (height <= 480px) {
  .mobile-parse-page {
    gap: var(--space-6);
    padding: var(--space-8) var(--space-12);
  }

  .parse-back-button {
    flex-basis: 2.25rem;
    width: 2.25rem;
    height: 2.25rem;
  }
}
</style>
