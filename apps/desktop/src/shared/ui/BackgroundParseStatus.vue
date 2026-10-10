<script setup lang="ts">
import { useParseStore } from '../stores/parse'
import UiButton from './Button.vue'
import ParseActivityStatus from './ParseActivityStatus.vue'
const parse = useParseStore()
</script>

<template>
  <section v-if="parse.backgroundJob" class="flex shrink-0 flex-wrap items-center gap-3 border-b border-(--color-border) bg-(--color-surface) px-4 py-2" aria-label="后台解析下载">
    <div class="grid min-w-0 flex-1 basis-full gap-1 sm:basis-0" role="status">
      <strong class="truncate text-xs">{{ parse.backgroundJob.title }}</strong>
      <span class="text-xs text-(--color-muted)">
        {{ parse.backgroundJob.status === 'running' ? '后台解析下载中' : parse.backgroundJob.status === 'completed' ? '后台解析下载已完成' : parse.backgroundJob.status === 'stopped' ? '后台解析下载已停止' : '后台解析下载失败' }}
        · 已处理 {{ parse.backgroundJob.processed }} 项 · 已加入 {{ parse.backgroundJob.created }} 个任务 · 跳过 {{ parse.backgroundJob.skipped }} 项
      </span>
      <ParseActivityStatus v-if="parse.backgroundJob.status === 'running'" class="text-xs text-(--color-muted)" :source-id="parse.backgroundJob.sourceId" :stopping="parse.pacedParsingStopRequestedBySource[parse.backgroundJob.sourceId]" />
      <span v-if="parse.backgroundJob.error" class="break-all text-xs text-(--color-danger)">{{ parse.backgroundJob.error }}</span>
      <details v-if="parse.backgroundJob.failures.length" class="text-xs text-(--color-danger)">
        <summary class="cursor-pointer">{{ parse.backgroundJob.failures.length }} 项失败，展开查看</summary>
        <ul class="m-0 grid max-h-40 gap-2 overflow-y-auto py-2 pl-4">
          <li v-for="failure in parse.backgroundJob.failures" :key="failure.partId" class="break-all">
            {{ failure.title }}：{{ failure.message }}
          </li>
        </ul>
      </details>
    </div>
    <UiButton v-if="parse.backgroundJob.status === 'running'" size="compact" variant="secondary" :disabled="parse.pacedParsingStopRequestedBySource[parse.backgroundJob.sourceId]" @click="parse.stopPacedParsing(parse.backgroundJob.sourceId)">
      {{ parse.pacedParsingStopRequestedBySource[parse.backgroundJob.sourceId] ? '正在停止' : '停止后台解析' }}
    </UiButton>
    <div v-else class="flex flex-wrap gap-2">
      <UiButton v-if="parse.backgroundJob.failures.length" size="compact" variant="secondary" @click="parse.startBackgroundDownload(parse.backgroundJob.sourceId, true)">重试失败项</UiButton>
      <UiButton v-if="parse.backgroundJob.status === 'stopped' || parse.backgroundJob.status === 'failed'" size="compact" variant="secondary" @click="parse.startBackgroundDownload(parse.backgroundJob.sourceId)">继续未处理项</UiButton>
      <UiButton size="compact" variant="ghost" @click="parse.backgroundJob = null">关闭</UiButton>
    </div>
  </section>
</template>
