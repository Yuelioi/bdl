<script setup lang="ts">
import { computed, ref } from 'vue'

import type { TaskActionDescriptor, TaskActionKind, TransferTaskView } from '../stores/transferView'
import UiIconButton from './IconButton.vue'

const { view, disabled = false } = defineProps<{
  view: TransferTaskView
  disabled?: boolean
}>()

const emit = defineEmits<{
  action: [action: Exclude<TaskActionKind, 'none'>]
}>()

const open = ref(false)
const hasPrimaryAction = computed(() => view.primaryAction !== 'none')
const hasSecondaryActions = computed(() => view.secondaryActions.length > 0)

const runPrimary = () => {
  if (view.primaryAction === 'none') {
    return
  }

  emit('action', view.primaryAction)
}

const runSecondary = (action: TaskActionDescriptor) => {
  open.value = false
  emit('action', action.kind)
}
</script>

<template>
  <div class="task-action-menu">
    <UiIconButton
      v-if="hasPrimaryAction"
      class="primary-action"
      :icon="view.primaryActionIcon"
      :label="view.primaryActionLabel"
      variant="secondary"
      :disabled
      @click.stop="runPrimary"
    />
    <UiIconButton
      v-if="hasSecondaryActions"
      icon="more"
      label="更多操作"
      variant="ghost"
      :disabled
      @click.stop="open = !open"
    />
    <div v-if="open" class="action-popover" role="menu" @click.stop>
      <button
        v-for="action in view.secondaryActions"
        :key="action.kind"
        type="button"
        role="menuitem"
        :class="{ danger: action.tone === 'danger' }"
        @click="runSecondary(action)"
      >
        <span>{{ action.label }}</span>
      </button>
    </div>
  </div>
</template>

<style scoped>
.task-action-menu {
  position: relative;
  min-width: 0;
  display: inline-flex;
  align-items: center;
  justify-content: flex-end;
  gap: 2px;
}

.task-action-menu :deep(.ui-icon-button) {
  width: 28px;
  height: 28px;
}

.action-popover {
  position: absolute;
  top: calc(100% + var(--space-4));
  right: 0;
  z-index: 30;
  min-width: 112px;
  display: grid;
  gap: 2px;
  padding: var(--space-4);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-8);
  background: var(--color-surface);
  box-shadow: 0 12px 28px rgb(23 33 29 / 12%);
}

.action-popover button {
  height: 30px;
  border: 0;
  border-radius: var(--radius-6);
  background: transparent;
  color: var(--color-text);
  padding: 0 var(--space-8);
  text-align: left;
  font-size: var(--font-13);
  font-weight: 650;
}

.action-popover button:hover {
  background: var(--color-panel);
}

.action-popover button.danger {
  color: var(--color-danger);
}
</style>
