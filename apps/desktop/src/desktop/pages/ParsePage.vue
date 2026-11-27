<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import UiButton from '../../shared/ui/Button.vue';
import UiInlineNotice from '../../shared/ui/InlineNotice.vue';
import ParseSourceInput from '../../shared/features/parse/ParseSourceInput.vue';
import ParseDownloadPlanner from '../../shared/features/parse/ParseDownloadPlanner.vue';
import ParseBatchWorkspace from '../components/parse/ParseBatchWorkspace.vue';
import ParseResultWorkspace from '../components/parse/ParseResultWorkspace.vue';
import { useParsePage } from '../../shared/features/parse/useParsePage';
import { sourcePartCount } from '../../shared/features/parse/parseResultTree';
import { useUiStore } from '../../shared/stores/ui';
const {
  parse,
  inputMode,
  singleInput,
  createLoading,
  activeStage,
  submitInput,
  returnToInput,
  pasteSingleInput,
  pasteInput,
  downloadSelected,
  openAllDownloadSettings,
  runNoticeAction,
} = useParsePage();
const ui = useUiStore();
const inputHeading = ref<HTMLElement>();
const resultNavigation = ref<HTMLElement>();
const result = computed(() => parse.isBatch ? undefined : parse.activeSource);
const sourceBusy = computed(() => {
  const id = result.value?.source.id;
  return Boolean(id && (parse.loadingBySource[id] || parse.pacedParsingBySource[id]));
});
const downloadAllSourceId = computed(() => {
  const source = result.value;
  return source && (source.source.has_more || sourcePartCount(source) > 0) ? source.source.id : undefined;
});

watch(activeStage, (stage) => {
  if (ui.activeTab !== 'parse') return;
  const destination = stage === 'content' ? resultNavigation.value : inputHeading.value;
  destination?.focus({ preventScroll: true });
}, { flush: 'post' });
</script>

<template>
  <section class="page-grid grid-cols-1">
    <section
      class="panel parse-panel min-h-0 overflow-hidden bg-(--color-surface)"
      :class="{ 'parse-panel-entry': activeStage !== 'content' }"
    >
      <header v-if="activeStage === 'content'" ref="resultNavigation" class="parse-result-navigation" tabindex="-1" aria-label="解析结果">
        <UiButton variant="ghost" size="compact" @click="returnToInput">
          <UIcon name="i-tabler-arrow-left" class="size-4" aria-hidden="true" />
          返回解析
        </UiButton>
        <UiButton v-if="result" variant="ghost" size="compact" :disabled="sourceBusy" @click="parse.clearWorkspace()">
          <UIcon name="i-tabler-x" class="size-4" aria-hidden="true" />
          清除结果
        </UiButton>
      </header>
      <template v-else>
        <header class="parse-entry-heading">
          <div class="parse-entry-heading-row">
            <h2 ref="inputHeading" tabindex="-1">解析链接</h2>
            <UiButton
              v-if="parse.activeSource"
              variant="secondary"
              size="compact"
              :disabled="createLoading"
              @click="activeStage = 'content'"
            >
              查看上次结果
              <UIcon name="i-tabler-arrow-right" class="size-4" aria-hidden="true" />
            </UiButton>
          </div>
          <p>支持视频、合集、收藏夹、UP 主空间、番剧和课程。</p>
        </header>
        <ParseSourceInput
          v-model:mode="inputMode"
          v-model:input="parse.input"
          v-model:single-input="singleInput"
          :busy="createLoading"
          :paused="Boolean(parse.createPause)"
          :pausing="Boolean(parse.createPause && parse.operationsBySource.__create__?.length)"
          @pause="parse.pauseCreation()"
          @resume="parse.resumeCreation()"
          @submit="submitInput"
          @paste="pasteInput"
          @single-paste="pasteSingleInput"
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
        <ParseResultWorkspace v-else />
      </template>
      <ParseDownloadPlanner ref="download-planner" :inline="activeStage === 'content'">
        <template v-if="downloadAllSourceId" #download-scope="{ busy }">
          <UiButton
            variant="secondary"
            :disabled="busy || sourceBusy"
            @click="openAllDownloadSettings(downloadAllSourceId)"
          >下载全部</UiButton>
        </template>
      </ParseDownloadPlanner>
    </section>
  </section>
</template>

<style scoped>
.parse-panel {
  gap: 1rem;
  padding: 0;
}

.parse-panel-entry {
  width: min(100%, 44rem);
  margin-inline: auto;
  padding-top: clamp(1rem, 10vh, 5rem);
  overflow-y: auto;
}

.parse-entry-heading {
  display: grid;
  gap: var(--space-4);
}

.parse-entry-heading-row {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-8);
}

.parse-result-navigation {
  display: flex;
  justify-content: space-between;
  flex-shrink: 0;
}

.parse-entry-heading h2,
.parse-entry-heading p {
  margin: 0;
}

.parse-result-navigation:focus,
.parse-entry-heading h2:focus {
  outline: none;
}

.parse-entry-heading p {
  color: var(--color-muted);
  font-size: var(--font-13);
}
</style>
