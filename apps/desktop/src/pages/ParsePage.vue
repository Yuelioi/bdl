<script setup lang="ts">
import { computed, useTemplateRef } from 'vue'

import { useParseStore } from '../stores/parse'
import { useUiStore } from '../stores/ui'
import UiButton from '../ui/Button.vue'
import UiInlineNotice from '../ui/InlineNotice.vue'
import UiStatusBadge from '../ui/StatusBadge.vue'
import UiTextarea from '../ui/Textarea.vue'
import ParseDownloadPlanner from './parse/ParseDownloadPlanner.vue'
import ParseResultWorkspace from './parse/ParseResultWorkspace.vue'
import SourceSwitcher from './parse/SourceSwitcher.vue'

const parse = useParseStore()
const ui = useUiStore()
const inputFile = useTemplateRef<HTMLInputElement>('input-file')
const downloadPlanner = useTemplateRef<{ openDialog: () => Promise<void> }>('download-planner')
const createLoading = computed(() => Boolean(parse.loadingBySource.__create__))
const hasResults = computed(() => Boolean(parse.activeSource))

const submitInput = () => void parse.createSource()
const importTextFile = () => inputFile.value?.click()
const selectSource = (sourceId: string) => parse.setActiveSource(sourceId)
const openDownloadSettings = () => void downloadPlanner.value?.openDialog()

const handleTextFile = async (event: Event) => {
  const target = event.target as HTMLInputElement
  const file = target.files?.[0]
  if (!file) return
  parse.appendInput(await file.text())
  target.value = ''
}

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
          <SourceSwitcher
            :sources="parse.orderedSources"
            :active-source-id="parse.activeSourceId"
            :selection-by-source="parse.selectionBySource"
            @select="selectSource"
          />
          <UiStatusBadge v-if="createLoading" status="downloading">解析中</UiStatusBadge>
        </div>
      </div>

      <form class="grid grid-cols-[minmax(0,1fr)_116px] items-end gap-3 max-[840px]:grid-cols-1" @submit.prevent="submitInput">
        <UiTextarea
          v-model="parse.input"
          label="链接、BV / AV 或多行列表"
          :rows="hasResults ? 2 : 3"
          placeholder="粘贴视频、合集、收藏夹、番剧或课程链接"
          :disabled="createLoading"
        />
        <div class="grid gap-2 max-[840px]:grid-cols-2">
          <UiButton type="submit" :disabled="createLoading">{{ createLoading ? '解析中' : '开始解析' }}</UiButton>
          <UiButton type="button" variant="secondary" :disabled="createLoading" @click="importTextFile">导入文本</UiButton>
          <input ref="input-file" class="pointer-events-none fixed size-px opacity-0" type="file" accept=".txt,.list,.csv,text/plain" @change="handleTextFile" />
        </div>
      </form>

      <UiInlineNotice
        v-if="parse.notice"
        :tone="parse.notice.tone"
        :action-label="parse.notice.actionLabel"
        @action="runNoticeAction"
      >{{ parse.notice.message }}</UiInlineNotice>
    </section>

    <ParseResultWorkspace @download="openDownloadSettings" />
    <ParseDownloadPlanner ref="download-planner" />
  </section>
</template>
