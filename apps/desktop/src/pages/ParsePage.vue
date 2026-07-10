<script setup lang="ts">
import { computed, useTemplateRef } from 'vue'

import { useParseStore } from '../stores/parse'
import { useUiStore } from '../stores/ui'
import UiButton from '../ui/Button.vue'
import UiInlineNotice from '../ui/InlineNotice.vue'
import UiStatusBadge from '../ui/StatusBadge.vue'
import UiTextarea from '../ui/Textarea.vue'
import ParseDownloadPlanner from './parse/ParseDownloadPlanner.vue'
import ParseBatchWorkspace from './parse/ParseBatchWorkspace.vue'
import ParseResultWorkspace from './parse/ParseResultWorkspace.vue'

const parse = useParseStore()
const ui = useUiStore()
const downloadPlanner = useTemplateRef<{ openDialog: () => Promise<void> }>('download-planner')
const createLoading = computed(() => Boolean(parse.loadingBySource.__create__))
const hasResults = computed(() => Boolean(parse.activeSource))

const submitInput = () => void parse.createSource()
const openDownloadSettings = () => void downloadPlanner.value?.openDialog()

const runNoticeAction = () => {
  if (parse.notice?.actionLabel !== '查看传输') return
  ui.setTab('transfer')
  parse.clearNotice()
}
</script>

<template>
  <section class="page-grid grid-cols-1 grid-rows-[auto_minmax(0,1fr)]">
    <section
      class="panel relative col-span-full overflow-visible bg-(--color-surface)"
      :class="hasResults ? 'gap-3 py-3' : ''"
    >
      <div class="panel-heading">
        <div class="flex min-w-0 items-start gap-3">
          <span class="mt-px font-(--font-display) text-[11px] font-bold text-(--color-accent-strong)">01</span>
          <div class="grid min-w-0 gap-1">
            <h2 class="text-balance">添加来源</h2>
            <p v-if="!hasResults" class="text-pretty m-0 max-w-[62ch] text-xs leading-5 text-(--color-muted)">
              粘贴链接或编号，BDL 会识别类型并整理成可选择的分集。
            </p>
          </div>
        </div>
        <div class="inline-flex min-w-0 items-center justify-end gap-2">
          <UiStatusBadge v-if="hasResults && !createLoading" status="done">已解析</UiStatusBadge>
          <UiStatusBadge v-if="createLoading" status="downloading">解析中</UiStatusBadge>
        </div>
      </div>

      <form class="grid grid-cols-[minmax(0,1fr)_116px] items-end gap-3 max-[840px]:grid-cols-1" @submit.prevent="submitInput">
        <UiTextarea
          v-model="parse.input"
          label="链接或 BV / AV（每行一个）"
          :rows="hasResults ? 2 : 3"
          placeholder="粘贴一个或多个视频链接；批量时每行作为一个视频"
          :disabled="createLoading"
        />
        <UiButton type="submit" :disabled="createLoading">{{ createLoading ? '解析中' : '开始解析' }}</UiButton>
      </form>

      <UiInlineNotice
        v-if="parse.notice"
        :tone="parse.notice.tone"
        :action-label="parse.notice.actionLabel"
        @action="runNoticeAction"
      >{{ parse.notice.message }}</UiInlineNotice>
    </section>

    <ParseBatchWorkspace v-if="parse.isBatch" @download="openDownloadSettings" />
    <ParseResultWorkspace v-else @download="openDownloadSettings" />
    <ParseDownloadPlanner ref="download-planner" />
  </section>
</template>
