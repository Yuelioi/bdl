<script setup lang="ts">
import UiButton from '../ui/Button.vue';
import UiInlineNotice from '../ui/InlineNotice.vue';
import UiStatusBadge from '../ui/StatusBadge.vue';
import UiTextarea from '../ui/Textarea.vue';
import UiTextField from '../ui/TextField.vue';
import UiWorkflowSteps from '../ui/WorkflowSteps.vue';
import ParseDownloadPlanner from './parse/ParseDownloadPlanner.vue';
import ParseBatchWorkspace from './parse/ParseBatchWorkspace.vue';
import ParseResultWorkspace from './parse/ParseResultWorkspace.vue';
import { useParsePage } from './useParsePage';
const {
  parse,
  inputMode,
  singleInput,
  createLoading,
  activeStage,
  workflowSteps,
  submitInput,
  switchInputMode,
  pasteSingleInput,
  pasteInput,
  openDownloadSettings,
  runNoticeAction,
} = useParsePage();
</script>
<template>
  <section class="page-grid grid-cols-1">
    <section class="panel parse-panel min-h-0 gap-4 overflow-hidden bg-(--color-surface) p-4">
      <UiWorkflowSteps v-model="activeStage" :steps="workflowSteps" />

      <UiInlineNotice
        v-if="parse.notice"
        :tone="parse.notice.tone"
        :action-label="parse.notice.actionLabel"
        @action="runNoticeAction"
        >{{ parse.notice.message }}</UiInlineNotice
      >

      <section v-if="activeStage === 'source'" class="parse-entry">
        <div class="parse-entry-shell">
          <header class="parse-entry-header">
            <div class="parse-entry-heading">
              <h2>解析链接</h2>
              <p>
                {{
                  inputMode === 'single'
                    ? '只解析当前视频及其分 P，不展开所属合集。'
                    : '粘贴一个或多个 Bilibili 来源，解析后再选择要下载的视频与分集。'
                }}
              </p>
            </div>
            <UiStatusBadge v-if="createLoading" status="downloading">解析中</UiStatusBadge>
          </header>

          <form class="parse-entry-form" @submit.prevent="submitInput" @keydown.ctrl.enter.prevent="submitInput">
            <div class="parse-mode-switch" role="group" aria-label="解析模式">
              <UiButton
                v-for="mode in [
                  { value: 'batch', label: '批量解析' },
                  { value: 'single', label: '单个视频' },
                ] as const"
                :key="mode.value"
                size="compact"
                variant="secondary"
                :aria-pressed="inputMode === mode.value"
                :disabled="createLoading"
                @click="switchInputMode(mode.value)"
                >{{ mode.label }}</UiButton
              >
            </div>
            <UiTextField
              v-if="inputMode === 'single'"
              v-model="singleInput"
              label="视频链接或 BV / AV"
              placeholder="https://www.bilibili.com/video/BV..."
              :disabled="createLoading"
              @paste.prevent="pasteSingleInput"
            />
            <UiTextarea
              v-else
              v-model="parse.input"
              label="Bilibili 链接或 BV / AV"
              hide-label
              helper="每行一个来源；合集、收藏夹与 UP 空间会按页加载。"
              :rows="7"
              placeholder="https://www.bilibili.com/video/BV...&#10;https://space.bilibili.com/..."
              :disabled="createLoading"
            />

            <div class="parse-entry-actions">
              <UiButton type="button" variant="secondary" :disabled="createLoading" @click="pasteInput">
                <UIcon name="i-tabler-clipboard" aria-hidden="true" />
                粘贴链接
              </UiButton>
              <div class="parse-entry-submit">
                <span class="parse-entry-shortcut" aria-hidden="true">{{
                  inputMode === 'single' ? 'Enter' : 'Ctrl + Enter'
                }}</span>
                <UiButton class="min-w-28" type="submit" :disabled="createLoading">
                  {{ createLoading ? '解析中' : '开始解析' }}
                </UiButton>
              </div>
            </div>
          </form>

          <aside v-if="inputMode === 'batch'" class="parse-entry-sources" aria-label="支持的来源类型">
            <strong>支持来源</strong>
            <ul>
              <li><UIcon name="i-tabler-video" aria-hidden="true" />视频与分 P</li>
              <li><UIcon name="i-tabler-bookmarks" aria-hidden="true" />收藏夹与合集</li>
              <li><UIcon name="i-tabler-device-tv" aria-hidden="true" />番剧与课程</li>
              <li><UIcon name="i-tabler-user-square-rounded" aria-hidden="true" />UP 空间</li>
              <li><UIcon name="i-tabler-list-details" aria-hidden="true" />多行批量</li>
            </ul>
          </aside>
        </div>
      </section>

      <template v-else>
        <ParseBatchWorkspace v-if="parse.isBatch" embedded @download="openDownloadSettings" />
        <ParseResultWorkspace v-else embedded @download="openDownloadSettings" />
      </template>
    </section>
    <ParseDownloadPlanner ref="download-planner" />
  </section>
</template>

<style scoped>
.parse-entry {
  min-height: 0;
  flex: 1;
  overflow-y: auto;
  padding: clamp(40px, 8vh, 68px) var(--space-xl) var(--space-xl);
}

.parse-entry-shell {
  width: 100%;
  max-width: 760px;
  display: grid;
  gap: var(--space-lg);
  margin-inline: auto;
}

.parse-entry-header {
  min-width: 0;
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--space-md);
}

.parse-entry-heading {
  min-width: 0;
  display: grid;
  gap: var(--space-xs);
}

.parse-entry-heading h2 {
  margin: 0;
  color: var(--color-text-strong);
  font-family: var(--font-display);
  font-size: clamp(24px, 2.5vw, 28px);
  font-weight: 760;
  line-height: 1.25;
  letter-spacing: -0.02em;
}

.parse-entry-heading p {
  max-width: 60ch;
  margin: 0;
  color: var(--color-muted);
  font-size: var(--font-13);
  line-height: 1.65;
}

.parse-entry-form {
  min-width: 0;
  display: grid;
  gap: var(--space-md);
}

.parse-mode-switch {
  display: flex;
  align-items: center;
  gap: var(--space-8);
}

.parse-mode-switch :deep(button[aria-pressed='true']) {
  border-color: var(--color-accent);
  background: var(--color-selected-surface);
  color: var(--color-accent-strong);
}

.parse-entry-form :deep(.textarea-control) {
  min-height: 156px;
  border-radius: var(--radius-8);
  background: var(--color-surface-raised);
  font-size: var(--font-14);
  line-height: 1.6;
}

.parse-entry-actions {
  min-width: 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-md);
}

.parse-entry-submit {
  flex: 0 0 auto;
  display: flex;
  align-items: center;
  gap: var(--space-sm);
}

.parse-entry-shortcut {
  color: var(--color-dimmed);
  font-size: var(--font-11);
  font-variant-numeric: tabular-nums;
}

.parse-entry-sources {
  min-width: 0;
  display: grid;
  grid-template-columns: auto minmax(0, 1fr);
  align-items: center;
  gap: var(--space-md);
  padding-top: var(--space-md);
  border-top: 1px solid var(--color-border);
}

.parse-entry-sources > strong {
  color: var(--color-text);
  font-size: var(--font-12);
  white-space: nowrap;
}

.parse-entry-sources ul {
  min-width: 0;
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: var(--space-xs) var(--space-md);
  margin: 0;
  padding: 0;
  list-style: none;
}

.parse-entry-sources li {
  display: inline-flex;
  align-items: center;
  gap: var(--space-2xs);
  color: var(--color-muted);
  font-size: var(--font-12);
  white-space: nowrap;
}

.parse-entry-sources li svg {
  width: 15px;
  height: 15px;
  color: var(--color-accent-strong);
}

@media (width <= 620px) {
  .parse-entry {
    padding: var(--space-lg) var(--space-md);
  }

  .parse-entry-actions {
    align-items: stretch;
    flex-direction: column;
  }

  .parse-entry-actions > :deep(button) {
    min-height: 44px;
  }

  .parse-entry-submit {
    width: 100%;
  }

  .parse-entry-submit :deep(button) {
    width: 100%;
  }

  .parse-entry-shortcut {
    display: none;
  }

  .parse-entry-sources {
    grid-template-columns: 1fr;
    gap: var(--space-sm);
  }
}
</style>
