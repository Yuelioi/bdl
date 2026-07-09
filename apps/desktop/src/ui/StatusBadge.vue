<script setup lang="ts">
const { status = 'ready' } = defineProps<{
  status?: 'ready' | 'downloading' | 'queued' | 'done' | 'warning' | 'error' | 'paused'
}>()
</script>

<template>
  <span class="status-badge" :class="`status-${status}`">
    <slot />
  </span>
</template>

<style scoped>
.status-badge {
  height: 22px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 0 var(--space-8);
  border-radius: 999px;
  font-size: var(--font-12);
  font-weight: 700;
  line-height: 1;
  white-space: nowrap;
}

.status-ready,
.status-queued {
  background: #edf1ef;
  color: var(--color-muted);
}

.status-downloading {
  background: #e8f3ee;
  color: var(--color-accent-strong);
}

.status-done {
  background: #eaf5eb;
  color: var(--color-success);
}

.status-warning {
  background: #fff4e6;
  color: var(--color-warning);
}

.status-paused {
  background: #eef2ff;
  color: #4254b7;
}

.status-error {
  background: #fff0f0;
  color: var(--color-danger);
}
</style>
