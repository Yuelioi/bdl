<script setup lang="ts">
import { useUiStore } from '../stores/ui'
import UiButton from './Button.vue'
import UiIconButton from './IconButton.vue'

const ui = useUiStore()
</script>

<template>
  <Teleport to="body">
    <div class="toast-host" aria-live="polite" aria-atomic="false">
      <div
        v-for="toast in ui.toasts"
        :key="toast.id"
        class="toast feedback-tone"
        :class="`tone-${toast.tone}`"
        :role="toast.tone === 'danger' ? 'alert' : 'status'"
        :aria-live="toast.tone === 'danger' ? 'assertive' : 'polite'"
      >
        <span>{{ toast.message }}</span>
        <UiButton
          v-if="toast.action"
          variant="ghost"
          size="compact"
          @click="ui.setTab(toast.action.tab); ui.removeToast(toast.id)"
        >
          {{ toast.action.label }}
        </UiButton>
        <UiIconButton icon="x" label="关闭通知" variant="ghost" size="compact" @click="ui.removeToast(toast.id)" />
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.toast-host {
  position: fixed;
  right: var(--space-16);
  top: calc(var(--height-topbar) + var(--space-12));
  z-index: var(--z-toast);
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

.toast :deep(.ui-button) {
  color: var(--color-accent-strong);
}
</style>
