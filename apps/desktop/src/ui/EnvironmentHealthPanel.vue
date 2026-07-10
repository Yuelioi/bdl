<script setup lang="ts">
import { computed } from 'vue'

import type { EnvironmentHealthSnapshot } from '../api/dto'
import UiButton from './Button.vue'
import UiStatusBadge from './StatusBadge.vue'

const { health, checking = false, compact = false } = defineProps<{
  health: EnvironmentHealthSnapshot | null
  checking?: boolean
  compact?: boolean
}>()

const emit = defineEmits<{
  check: []
  createDirectory: []
  chooseDirectory: []
  chooseFfmpeg: []
  useSystemFfmpeg: []
}>()

const overallLabel = computed(() => {
  if (checking) return '检查中'
  if (!health) return '待检查'
  return health.ready ? '环境就绪' : '需要处理'
})
const overallBadge = computed(() => {
  if (checking) return 'downloading'
  if (!health) return 'queued'
  return health.ready ? 'done' : 'warning'
})
const directoryReady = computed(() => health?.download_directory.status === 'ready')
const ffmpegReady = computed(() => health?.ffmpeg.status === 'ready')
</script>

<template>
  <section class="environment-health" :class="{ compact }" aria-label="运行环境检查" :aria-busy="checking">
    <header>
      <div>
        <strong>运行环境</strong>
        <span v-if="!compact">下载前检查保存目录和 FFmpeg</span>
      </div>
      <div class="environment-health-heading-actions">
        <span role="status" aria-live="polite" aria-atomic="true">
          <UiStatusBadge :status="overallBadge">{{ overallLabel }}</UiStatusBadge>
        </span>
        <UiButton variant="ghost" :disabled="checking" @click="emit('check')">重新检查</UiButton>
      </div>
    </header>

    <div v-if="health" class="environment-components">
      <article>
        <span class="component-icon" :class="{ ready: directoryReady }" aria-hidden="true">
          <UIcon name="i-tabler-folder-check" />
        </span>
        <div class="component-copy">
          <div>
            <strong>保存目录</strong>
            <UiStatusBadge :status="directoryReady ? 'done' : 'warning'">
              {{ directoryReady ? '可写' : '异常' }}
            </UiStatusBadge>
          </div>
          <p>{{ health.download_directory.message }}</p>
          <code :title="health.download_directory.path">{{ health.download_directory.path }}</code>
        </div>
        <div v-if="!directoryReady" class="component-actions">
          <UiButton
            v-if="health.download_directory.status === 'missing'"
            variant="secondary"
            :disabled="checking"
            @click="emit('createDirectory')"
          >
            创建目录
          </UiButton>
          <UiButton variant="ghost" :disabled="checking" @click="emit('chooseDirectory')">重新选择</UiButton>
        </div>
      </article>

      <article>
        <span class="component-icon" :class="{ ready: ffmpegReady }" aria-hidden="true">
          <UIcon name="i-tabler-terminal-2" />
        </span>
        <div class="component-copy">
          <div>
            <strong>FFmpeg</strong>
            <UiStatusBadge :status="ffmpegReady ? 'done' : 'warning'">
              {{ ffmpegReady ? '可用' : '异常' }}
            </UiStatusBadge>
          </div>
          <p>{{ health.ffmpeg.version ?? health.ffmpeg.message }}</p>
          <code v-if="health.ffmpeg.path" :title="health.ffmpeg.path">{{ health.ffmpeg.path }}</code>
        </div>
        <div v-if="!ffmpegReady" class="component-actions">
          <UiButton variant="secondary" :disabled="checking" @click="emit('chooseFfmpeg')">选择 FFmpeg</UiButton>
          <UiButton
            v-if="health.ffmpeg.source === 'configured'"
            variant="ghost"
            :disabled="checking"
            @click="emit('useSystemFfmpeg')"
          >
            使用系统版本
          </UiButton>
        </div>
      </article>
    </div>

    <p v-else class="environment-pending">检查后会显示目录写入权限和 FFmpeg 版本。</p>
  </section>
</template>

<style scoped>
.environment-health {
  display: grid;
  gap: var(--space-12);
  padding: var(--space-16);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-12);
  background: var(--color-panel);
}

.environment-health > header,
.environment-health-heading-actions,
.component-copy > div,
.component-actions {
  display: flex;
  align-items: center;
  gap: var(--space-8);
}

.environment-health > header {
  justify-content: space-between;
}

.environment-health > header > div:first-child {
  min-width: 0;
  display: grid;
  gap: 2px;
}

.environment-health > header span,
.component-copy p,
.environment-pending {
  color: var(--color-text-muted);
}

.environment-components {
  display: grid;
  gap: var(--space-8);
}

.environment-components article {
  min-width: 0;
  display: grid;
  grid-template-columns: auto minmax(0, 1fr) auto;
  align-items: center;
  gap: var(--space-12);
  padding: var(--space-12);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-8);
  background: var(--color-surface);
}

.component-icon {
  width: 34px;
  height: 34px;
  display: grid;
  place-items: center;
  border-radius: var(--radius-8);
  background: var(--color-warning-soft);
  color: var(--color-warning);
}

.component-icon.ready {
  background: var(--color-accent-soft);
  color: var(--color-accent-strong);
}

.component-copy {
  min-width: 0;
  display: grid;
  gap: 3px;
}

.component-copy p,
.environment-pending {
  margin: 0;
  font-size: var(--font-12);
}

.component-copy code {
  min-width: 0;
  overflow: hidden;
  color: var(--color-text-muted);
  font-family: ui-monospace, "Cascadia Mono", monospace;
  font-size: var(--font-11);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.component-actions {
  justify-content: flex-end;
  flex-wrap: wrap;
}

.environment-health.compact {
  padding: var(--space-12);
}

.compact .environment-components article {
  grid-template-columns: auto minmax(0, 1fr);
}

.compact .component-actions {
  grid-column: 2;
  justify-content: flex-start;
}

@media (width <= 760px) {
  .environment-components article {
    grid-template-columns: auto minmax(0, 1fr);
  }

  .component-actions {
    grid-column: 2;
    justify-content: flex-start;
  }
}
</style>
