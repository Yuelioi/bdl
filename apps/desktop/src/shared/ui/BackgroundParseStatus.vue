<script setup lang="ts">
import { useParseStore } from '../stores/parse'
import ErrorDetails from './ErrorDetails.vue'
import UiButton from './Button.vue'
import ParseActivityStatus from './ParseActivityStatus.vue'
const parse = useParseStore()
</script>

<template>
  <section v-if="parse.backgroundJob" class="flex shrink-0 flex-wrap items-center gap-3 border-b border-(--color-border) bg-(--color-surface) px-4 py-2" aria-label="后台添加下载">
    <div class="grid min-w-0 flex-1 basis-full gap-1 sm:basis-0" role="status">
      <strong class="truncate text-xs">{{ parse.backgroundJob.title }}</strong>
      <span class="text-xs text-(--color-muted)">
        {{ parse.backgroundJob.status === 'running' ? '正在添加下载任务' : parse.backgroundJob.status === 'completed' ? '添加完成' : parse.backgroundJob.status === 'stopped' ? '已停止添加' : '添加失败' }}
        · 已添加 {{ parse.backgroundJob.created }} 个任务<template v-if="parse.backgroundJob.skipped"> · 已跳过 {{ parse.backgroundJob.skipped }} 项</template>
      </span>
      <ParseActivityStatus v-if="parse.backgroundJob.status === 'running'" class="text-xs text-(--color-muted)" :source-id="parse.backgroundJob.sourceId" :stopping="parse.pacedParsingStopRequestedBySource[parse.backgroundJob.sourceId]" />
      <span v-if="parse.backgroundJob.error" class="break-all text-xs text-(--color-danger)">{{ parse.backgroundJob.error }}</span>
      <ErrorDetails v-if="parse.backgroundJob.detail" :detail="parse.backgroundJob.detail" />
      <details v-if="parse.backgroundJob.failures.length" class="text-xs text-(--color-danger)">
        <summary class="cursor-pointer">{{ parse.backgroundJob.failures.length }} 项失败，展开查看</summary>
        <ul class="m-0 grid max-h-40 gap-2 overflow-y-auto py-2 pl-4">
          <li v-for="failure in parse.backgroundJob.failures" :key="failure.partId" class="break-all">
            {{ failure.title }}：{{ failure.message }}
            <ErrorDetails v-if="failure.detail" :detail="failure.detail" />
          </li>
        </ul>
      </details>
    </div>
    <UiButton v-if="parse.backgroundJob.status === 'running'" size="compact" variant="secondary" :disabled="parse.pacedParsingStopRequestedBySource[parse.backgroundJob.sourceId]" @click="parse.stopPacedParsing(parse.backgroundJob.sourceId)">
      {{ parse.pacedParsingStopRequestedBySource[parse.backgroundJob.sourceId] ? '正在停止' : '停止添加' }}
    </UiButton>
    <div v-else class="flex flex-wrap gap-2">
      <UiButton v-if="parse.backgroundJob.failures.length" size="compact" variant="secondary" @click="parse.startBackgroundDownload(parse.backgroundJob.sourceId, true)">重试失败项</UiButton>
      <UiButton v-if="parse.backgroundJob.status === 'stopped' || parse.backgroundJob.status === 'failed'" size="compact" variant="secondary" @click="parse.startBackgroundDownload(parse.backgroundJob.sourceId)">继续添加</UiButton>
      <UiButton size="compact" variant="ghost" @click="parse.backgroundJob = null">关闭</UiButton>
    </div>
  </section>
</template>
