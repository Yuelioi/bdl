<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue'
import { usePreparationsStore } from '../stores/preparations'
import { commandDiagnostic } from '../utils/commandDiagnostic'
import type { PreparationJob } from '../api/dto'
import UiButton from './Button.vue'
import UiDialog from './Dialog.vue'
import ErrorDetails from './ErrorDetails.vue'
import UiInlineNotice from './InlineNotice.vue'
import { isMobilePlatform } from '../platform/environment'

const preparations = usePreparationsStore()
const { showActive = false } = defineProps<{ showActive?: boolean }>()
const open = ref(false)
const mobile = isMobilePlatform()
const labels = {
  queued: '等待添加', preparing: '正在添加', awaiting_confirmation: '等待确认重复项',
  completed: '添加完成', failed: '添加失败', cancelled: '已停止添加', interrupted: '添加已中断',
} as const
const attentionJobs = computed(() => preparations.jobs.filter(job => ['awaiting_confirmation', 'failed', 'interrupted'].includes(job.status)))
const activeJobs = computed(() => preparations.pending.filter(job => job.status !== 'awaiting_confirmation'))
const visibleJobs = computed(() => showActive ? [...attentionJobs.value, ...activeJobs.value] : attentionJobs.value)
const current = computed(() => visibleJobs.value[0])
const diagnostic = (job: PreparationJob) => job.error ? commandDiagnostic(job.error, `操作: 准备下载任务\n提交入口: ${job.origin}\n任务: ${job.id}\n来源: ${job.title}\n链接: ${job.input}\n进度: ${job.processed}/${job.total}`, job.created_at) : ''
onMounted(() => preparations.start())
onUnmounted(() => preparations.stop())
</script>

<template>
  <section v-if="current || preparations.error" class="preparation-status" :class="{ 'preparation-mobile': mobile }" aria-label="下载任务准备">
    <div class="min-w-0 flex-1" role="status">
      <span class="block truncate text-xs">{{ current ? `${labels[current.status]} · ${current.title}` : preparations.error }}</span>
    </div>
    <UiButton size="compact" variant="secondary" @click="open = true">查看详情</UiButton>
  </section>
  <UiDialog v-model="open" title="添加下载任务">
    <UiInlineNotice v-if="preparations.error" tone="danger">
      {{ preparations.error }}
      <ErrorDetails v-if="preparations.detail" :detail="preparations.detail" />
      <UiButton size="compact" variant="ghost" @click="preparations.load()">刷新</UiButton>
    </UiInlineNotice>
    <div class="grid min-w-0 gap-3" :class="{ 'preparation-mobile': mobile }">
      <article v-for="job in visibleJobs" :key="job.id" class="preparation-entry" :aria-label="job.title">
        <div class="min-w-0">
          <strong class="block break-words text-sm font-medium" :class="{ truncate: !mobile }" :title="job.title">{{ job.title }}</strong>
          <p class="mt-1 mb-0 text-xs text-(--color-muted)">
            {{ labels[job.status] }}
            <template v-if="job.status === 'preparing' && job.total"> · {{ job.processed }} / {{ job.total }} 项</template>
            <template v-if="job.created"> · 已加入队列 {{ job.created }} 项</template>
            <template v-if="job.skipped_existing"> · 跳过 {{ job.skipped_existing }} 项</template>
          </p>
        </div>
        <p v-if="job.error" class="m-0 break-words text-xs text-(--color-danger)">{{ job.error.message }}</p>
        <ErrorDetails v-if="job.error" :detail="diagnostic(job)" />
        <template v-if="job.status === 'awaiting_confirmation'">
          <p class="m-0 text-xs">{{ job.duplicates.length }} 项已在传输记录中</p>
          <ul class="m-0 max-h-40 overflow-auto pl-5 text-xs" aria-label="重复任务">
            <li v-for="match in job.duplicates" :key="match.proposed_task_id" class="break-words">{{ match.title }}</li>
          </ul>
        </template>
        <div class="flex flex-wrap gap-2">
          <template v-if="job.status === 'awaiting_confirmation'">
            <UiButton size="compact" variant="secondary" :disabled="preparations.busyIds.includes(job.id)" @click="preparations.act(job.id, 'skip')">跳过重复项</UiButton>
            <UiButton size="compact" :disabled="preparations.busyIds.includes(job.id)" @click="preparations.act(job.id, 'create')">再下载一份</UiButton>
          </template>
          <UiButton v-if="['queued', 'preparing', 'awaiting_confirmation'].includes(job.status)" size="compact" variant="ghost" :disabled="preparations.busyIds.includes(job.id)" @click="preparations.act(job.id, 'cancel')">停止添加</UiButton>
          <UiButton v-else size="compact" variant="ghost" :disabled="preparations.busyIds.includes(job.id)" @click="preparations.act(job.id, 'dismiss')">关闭记录</UiButton>
        </div>
      </article>
      <p v-if="!visibleJobs.length && !preparations.error" class="m-0 text-sm text-(--color-muted)">没有需要处理的下载</p>
    </div>
  </UiDialog>
</template>

<style scoped>
.preparation-status {
  display: flex;
  min-width: 0;
  align-items: center;
  gap: var(--space-sm);
  padding: var(--space-sm) var(--space-md);
  border-bottom: 1px solid var(--color-border);
  background: var(--color-surface);
}

.preparation-entry {
  display: grid;
  min-width: 0;
  gap: var(--space-sm);
  padding: var(--space-md);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-8);
}

.preparation-mobile :deep(.ui-button) {
  min-height: 3rem;
  height: auto;
}
</style>
