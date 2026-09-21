<script setup lang="ts">
import { onUnmounted, ref, watch } from 'vue'
import { parseProgress } from '../api/tauri'

const props = defineProps<{ sourceId: string; stopping?: boolean }>()
const seconds = ref(0)
const queued = ref(false)
let generation = 0
let timer: ReturnType<typeof setTimeout> | undefined

watch(() => props.sourceId, (sourceId) => {
  const current = ++generation
  clearTimeout(timer)
  seconds.value = 0
  queued.value = false
  const poll = async () => {
    try {
      const progress = await parseProgress(sourceId)
      if (current !== generation) return
      seconds.value = progress.waiting_seconds
      queued.value = progress.queued
    } catch {
      // Keep the activity label when a status update is unavailable.
    }
    if (current === generation) timer = setTimeout(() => { void poll() }, 500)
  }
  void poll()
}, { immediate: true })

onUnmounted(() => { generation++; clearTimeout(timer) })
</script>

<template>
  <span>{{ stopping ? '正在停止…' : seconds > 0 ? `休息 ${seconds} 秒后继续` : queued ? '等待其他解析完成…' : '正在解析…' }}</span>
</template>
