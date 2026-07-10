<script setup lang="ts">
import { useUiStore } from '../stores/ui'

const ui = useUiStore()
</script>

<template>
  <Teleport to="body">
    <div class="toast-host" aria-live="polite" aria-atomic="false">
      <div v-for="toast in ui.toasts" :key="toast.id" class="toast feedback-tone" :class="`tone-${toast.tone}`">
        <span>{{ toast.message }}</span>
        <button
          v-if="toast.action"
          class="toast-action"
          type="button"
          @click="ui.setTab(toast.action.tab); ui.removeToast(toast.id)"
        >
          {{ toast.action.label }}
        </button>
        <button class="toast-close" type="button" aria-label="关闭通知" @click="ui.removeToast(toast.id)">x</button>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.toast-host {
  position: fixed;
  right: var(--space-16);
  top: calc(var(--height-topbar) + var(--space-12));
  z-index: 50;
  display: grid;
  gap: var(--space-8);
  width: min(360px, calc(100vw - 32px));
}

.toast {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto auto;
  align-items: center;
  gap: var(--space-8);
  min-height: 44px;
  padding: var(--space-8) var(--space-12);
  border: 1px solid var(--feedback-border);
  border-radius: var(--radius-8);
  background: var(--feedback-background);
  box-shadow: var(--shadow-overlay);
  color: var(--color-text);
  font-size: var(--font-13);
}

.toast span {
  min-width: 0;
  overflow-wrap: anywhere;
}

.toast button {
  width: 28px;
  height: 28px;
  border: 0;
  border-radius: var(--radius-6);
  background: transparent;
  color: var(--color-muted);
}

.toast-action {
  width: auto !important;
  padding: 0 var(--space-8);
  color: var(--color-accent-strong) !important;
  font-weight: 700;
  white-space: nowrap;
}

.toast-close {
  justify-self: end;
}
</style>
