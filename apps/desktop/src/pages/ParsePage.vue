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
  const parsed = await parse.createSource()
  if (parsed) activeStage.value = 'content'
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

      <section
        v-if="activeStage === 'source'"
        class="grid min-h-0 flex-1 place-items-center overflow-y-auto py-8 max-[840px]:place-items-start max-[840px]:py-4"
      >
        <div
          class="grid w-full max-w-4xl grid-cols-[220px_minmax(0,1fr)] items-start gap-10 max-[840px]:grid-cols-1 max-[840px]:gap-6"
        >
          <aside class="grid min-w-0 gap-6" aria-label="支持的来源类型">
            <div class="grid gap-2">
              <span class="text-[11px] font-bold text-(--color-accent-strong)">快速解析</span>
              <h2 class="text-balance m-0 text-xl leading-7 text-(--color-text)">从链接整理下载内容</h2>
              <p class="text-pretty m-0 text-xs leading-5 text-(--color-muted)">
                BDL 会识别来源类型，并把标题、分集与时长整理成下一步可选择的清单。
              </p>
            </div>

            <ul class="m-0 grid list-none gap-2 p-0 text-xs text-(--color-muted)">
              <li class="flex items-center gap-2">
                <UIcon
                  name="i-tabler-circle-check"
                  class="size-4 shrink-0 text-(--color-accent-strong)"
                  aria-hidden="true"
                />
                视频、BV 与 AV 编号
              </li>
              <li class="flex items-center gap-2">
                <UIcon
                  name="i-tabler-circle-check"
                  class="size-4 shrink-0 text-(--color-accent-strong)"
                  aria-hidden="true"
                />
                合集、收藏夹与 UP 空间
              </li>
              <li class="flex items-center gap-2">
                <UIcon
                  name="i-tabler-circle-check"
                  class="size-4 shrink-0 text-(--color-accent-strong)"
                  aria-hidden="true"
                />
                番剧、课程与多行批量视频
              </li>
            </ul>
          </aside>

          <form class="grid min-w-0 gap-3" @submit.prevent="submitInput">
            <div class="flex min-w-0 items-end justify-between gap-4">
              <div class="grid min-w-0 gap-1">
                <strong class="text-[13px] text-(--color-text)">粘贴来源</strong>
                <span class="text-xs text-(--color-muted)">每行一个链接；容器来源会展开为完整内容。</span>
              </div>
              <UiStatusBadge v-if="createLoading" status="downloading">解析中</UiStatusBadge>
            </div>

            <UiTextarea
              v-model="parse.input"
              label="链接或 BV / AV"
              :rows="6"
              placeholder="https://www.bilibili.com/video/BV...&#10;https://space.bilibili.com/..."
              :disabled="createLoading"
            />

            <div class="flex min-w-0 items-center justify-between gap-4 max-[620px]:items-stretch max-[620px]:flex-col">
              <span class="flex min-w-0 items-center gap-2 text-xs leading-5 text-(--color-muted)">
                <UIcon name="i-tabler-bolt" class="size-4 shrink-0 text-(--color-accent-strong)" aria-hidden="true" />
                媒体地址会在创建下载任务时获取
              </span>
              <UiButton class="min-w-28" type="submit" :disabled="createLoading">
                {{ createLoading ? '解析中' : '开始解析' }}
              </UiButton>
            </div>
          </form>
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
