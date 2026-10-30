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
const open = ref(false)
const mobile = isMobilePlatform()
const labels = {
  queued: '等待准备', preparing: '正在准备', awaiting_confirmation: '等待处理重复项',
  completed: '准备完成', failed: '准备失败', cancelled: '已停止', interrupted: '准备已中断',
} as const
const current = computed(() => preparations.pending[0] ?? preparations.jobs.find(job => ['failed', 'interrupted'].includes(job.status)) ?? preparations.jobs[0])
const confirmationCount = computed(() => preparations.jobs.filter(job => job.status === 'awaiting_confirmation').length)
const diagnostic = (job: PreparationJob) => job.error ? commandDiagnostic(job.error, `操作: 准备下载任务\n提交入口: ${job.origin}\n任务: ${job.id}\n来源: ${job.title}\n链接: ${job.input}\n进度: ${job.processed}/${job.total}`, job.created_at) : ''
onMounted(() => preparations.start())
onUnmounted(() => preparations.stop())
</script>

<template>
  <section v-if="current || preparations.error" class="preparation-status" :class="{ 'preparation-mobile': mobile }" aria-label="下载任务准备">
    <div class="min-w-0 flex-1" role="status">
      <span class="block truncate text-xs">{{ current ? `${labels[current.status]} · ${current.title}` : preparations.error }}</span>
      <span v-if="current" class="text-xs text-(--color-muted)">
        {{ ['queued', 'preparing'].includes(current.status) ? `已处理 ${current.processed} / ${current.total} 项` : `已加入 ${current.created} 个任务 · 跳过 ${current.skipped_existing} 项` }}
        <template v-if="confirmationCount"> · {{ confirmationCount }} 批待确认</template>
      </span>
    </div>
    <UiButton size="compact" variant="secondary" @click="open = true">查看准备任务</UiButton>
  </section>
  <UiDialog v-model="open" title="任务准备" size="wide">
    <UiInlineNotice v-if="preparations.error" tone="danger">
      {{ preparations.error }}
      <ErrorDetails v-if="preparations.detail" :detail="preparations.detail" />
      <UiButton size="compact" variant="ghost" @click="preparations.load()">重新读取</UiButton>
    </UiInlineNotice>
    <div class="grid min-w-0 gap-3" :class="{ 'preparation-mobile': mobile }">
      <article v-for="job in preparations.jobs" :key="job.id" class="preparation-entry" :aria-label="job.title">
        <div class="min-w-0">
          <strong class="block break-words text-sm font-medium" :class="{ truncate: !mobile }" :title="job.title">{{ job.title }}</strong>
          <p class="mt-1 mb-0 text-xs text-(--color-muted)">{{ labels[job.status] }} · 已处理 {{ job.processed }} / {{ job.total }} 项 · 已加入 {{ job.created }} 个任务 · 跳过 {{ job.skipped_existing }} 项</p>
        </div>
        <p v-if="job.error" class="m-0 break-words text-xs text-(--color-danger)">{{ job.error.message }}</p>
        <ErrorDetails v-if="job.error" :detail="diagnostic(job)" />
        <template v-if="job.status === 'awaiting_confirmation'">
          <p class="m-0 text-xs">{{ job.duplicates.length }} 个分集已在传输记录中</p>
          <ul class="m-0 max-h-40 overflow-auto pl-5 text-xs" aria-label="重复任务">
            <li v-for="match in job.duplicates" :key="match.proposed_task_id" class="break-words">{{ match.title }}</li>
          </ul>
        </template>
        <div class="flex flex-wrap gap-2">
          <template v-if="job.status === 'awaiting_confirmation'">
            <UiButton size="compact" variant="secondary" :disabled="preparations.busyIds.includes(job.id)" @click="preparations.act(job.id, 'skip')">跳过重复项</UiButton>
            <UiButton size="compact" :disabled="preparations.busyIds.includes(job.id)" @click="preparations.act(job.id, 'create')">创建副本任务</UiButton>
          </template>
          <UiButton v-if="['queued', 'preparing', 'awaiting_confirmation'].includes(job.status)" size="compact" variant="ghost" :disabled="preparations.busyIds.includes(job.id)" @click="preparations.act(job.id, 'cancel')">停止准备</UiButton>
          <UiButton v-else size="compact" variant="ghost" :disabled="preparations.busyIds.includes(job.id)" @click="preparations.act(job.id, 'dismiss')">关闭记录</UiButton>
        </div>
      </article>
      <p v-if="!preparations.jobs.length" class="m-0 text-sm text-(--color-muted)">暂无准备任务</p>
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
