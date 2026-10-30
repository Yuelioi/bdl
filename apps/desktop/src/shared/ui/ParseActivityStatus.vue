<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import { parseProgress } from '../api/tauri'
import { useParseStore } from '../stores/parse'

const props = defineProps<{ sourceId: string; stopping?: boolean }>()
const parse = useParseStore()
const now = ref(Date.now())
const waitingUntil = ref(0)
const seconds = computed(() => Math.max(0, Math.ceil((waitingUntil.value - now.value) / 1000)))
const updateClock = () => { now.value = Date.now() }
let clock: ReturnType<typeof setInterval> | undefined
onMounted(() => {
  clock = setInterval(updateClock, 250)
  document.addEventListener('visibilitychange', updateClock)
  window.addEventListener('focus', updateClock)
})
const queued = ref(false)
let generation = 0
let timer: ReturnType<typeof setTimeout> | undefined

watch(() => props.sourceId, (sourceId) => {
  const current = ++generation
  clearTimeout(timer)
  waitingUntil.value = 0
  queued.value = false
  const poll = async () => {
    try {
      const requestedAt = Date.now()
      const operationId = parse.operationsBySource[sourceId]?.[0]
      const progress = operationId ? await parseProgress(operationId) : { active: false, waiting_seconds: 0, queued: false }
      if (current !== generation) return
      waitingUntil.value = progress.active ? requestedAt + progress.waiting_seconds * 1000 : 0
      updateClock()
      queued.value = progress.active && progress.queued
    } catch {
      // Keep the activity label when a status update is unavailable.
    }
    if (current === generation) timer = setTimeout(() => { void poll() }, 500)
  }
  void poll()
}, { immediate: true })

onUnmounted(() => {
  generation++
  clearTimeout(timer)
  clearInterval(clock)
  document.removeEventListener('visibilitychange', updateClock)
  window.removeEventListener('focus', updateClock)
})
</script>

<template>
  <span>{{ stopping ? '正在停止…' : seconds > 0 ? `休息 ${seconds} 秒后继续` : queued ? '等待其他解析完成…' : '正在解析…' }}</span>
</template>
