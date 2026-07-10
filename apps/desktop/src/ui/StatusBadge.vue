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
  background: var(--color-panel-strong);
  color: var(--color-muted);
}

.status-downloading {
  background: var(--color-accent-soft);
  color: var(--color-accent-strong);
}

.status-done {
  background: var(--color-success-soft);
  color: var(--color-success);
}

.status-warning {
  background: var(--color-warning-soft);
  color: var(--color-warning);
}

.status-paused {
  background: var(--color-info-soft);
  color: var(--color-info);
}

.status-error {
  background: var(--color-danger-soft);
  color: var(--color-danger);
}
</style>
