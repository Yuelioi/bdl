<script setup lang="ts">
import { computed, ref, useTemplateRef, watch } from 'vue'

import { useParseStore } from '../stores/parse'
import { useUiStore } from '../stores/ui'
import UiButton from '../ui/Button.vue'
import UiInlineNotice from '../ui/InlineNotice.vue'
import UiStatusBadge from '../ui/StatusBadge.vue'
import UiTextarea from '../ui/Textarea.vue'
import UiWorkflowSteps, { type WorkflowStep } from '../ui/WorkflowSteps.vue'
import ParseDownloadPlanner from './parse/ParseDownloadPlanner.vue'
import ParseBatchWorkspace from './parse/ParseBatchWorkspace.vue'
import ParseResultWorkspace from './parse/ParseResultWorkspace.vue'

const parse = useParseStore()
const ui = useUiStore()
const downloadPlanner = useTemplateRef<{ openDialog: () => Promise<void> }>('download-planner')
const createLoading = computed(() => Boolean(parse.loadingBySource.__create__))
const hasResults = computed(() => Boolean(parse.activeSource))
const activeStage = ref<'source' | 'content'>(hasResults.value ? 'content' : 'source')
const workflowSteps = computed<WorkflowStep[]>(() => [
  { value: 'source', label: '解析来源', description: '输入链接', complete: hasResults.value },
  {
    value: 'content',
    label: '选择内容',
    description: hasResults.value ? '筛选并下载' : '等待解析',
    disabled: !hasResults.value,
  },
])

const submitInput = async () => {
  await parse.createSource()
  if (hasResults.value) activeStage.value = 'content'
}
const openDownloadSettings = () => void downloadPlanner.value?.openDialog()

watch(hasResults, (available) => {
  activeStage.value = available ? 'content' : 'source'
})

const runNoticeAction = () => {
  if (parse.notice?.actionLabel !== '查看传输') return
  ui.setTab('transfer')
  parse.clearNotice()
}
</script>

<template>
  <section class="page-grid grid-cols-1">
    <section class="panel min-h-0 gap-4 overflow-hidden bg-(--color-surface) p-4">
      <UiWorkflowSteps v-model="activeStage" :steps="workflowSteps" />

      <UiInlineNotice
        v-if="parse.notice"
        :tone="parse.notice.tone"
        :action-label="parse.notice.actionLabel"
        @action="runNoticeAction"
        >{{ parse.notice.message }}</UiInlineNotice
      >

      <section v-if="activeStage === 'source'" class="grid min-h-0 content-start gap-5 py-1">
        <header class="flex min-w-0 items-start justify-between gap-4">
          <div class="grid min-w-0 gap-1">
            <h2 class="m-0 text-lg text-(--color-text)">添加来源</h2>
            <p class="text-pretty m-0 max-w-[62ch] text-xs leading-5 text-(--color-muted)">
              每行输入一个视频链接；合集、收藏夹等容器链接会展开为可选内容。
            </p>
          </div>
          <UiStatusBadge v-if="createLoading" status="downloading">解析中</UiStatusBadge>
        </header>

        <form
          class="grid grid-cols-[minmax(0,1fr)_116px] items-end gap-3 max-[840px]:grid-cols-1"
          @submit.prevent="submitInput"
        >
          <UiTextarea
            v-model="parse.input"
            label="链接或 BV / AV（每行一个）"
            :rows="4"
            placeholder="粘贴一个或多个视频链接；批量时每行作为一个视频"
            :disabled="createLoading"
          />
          <UiButton type="submit" :disabled="createLoading">{{ createLoading ? '解析中' : '开始解析' }}</UiButton>
        </form>

        <div class="flex items-center gap-2 text-xs text-(--color-muted)">
          <UIcon name="i-tabler-bolt" class="size-4 text-(--color-accent-strong)" aria-hidden="true" />
          这里只读取标题、分集和时长；媒体地址会在创建下载任务时获取。
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
