<script setup lang="ts">
import { computed } from 'vue'

import type { TaskActionDescriptor, TaskActionKind, TransferTaskView } from '../stores/transferView'
import UiIconButton from './IconButton.vue'

const { view, disabled = false, showPrimary = true, showInspect = false, excludeActions = [] } = defineProps<{
  view: TransferTaskView
  disabled?: boolean
  showPrimary?: boolean
  showInspect?: boolean
  excludeActions?: TaskActionKind[]
}>()

const emit = defineEmits<{
  action: [action: Exclude<TaskActionKind, 'none'>]
  inspect: []
}>()

const hasPrimaryAction = computed(() => view.primaryAction !== 'none')
const visibleSecondaryActions = computed(() =>
  view.secondaryActions.filter((action) => !excludeActions.includes(action.kind)),
)
const dropdownItems = computed(() => [
  ...(showInspect
    ? [{ label: '详情和诊断', icon: 'i-tabler-info-circle', color: 'neutral', onSelect: () => emit('inspect') }]
    : []),
  ...visibleSecondaryActions.value.map((action) => ({
    label: action.label,
    icon: tablerIcon(action.icon),
    color: action.tone === 'danger' ? 'error' : 'neutral',
    onSelect: () => runSecondary(action),
  })),
])

const runPrimary = () => {
  if (view.primaryAction === 'none') {
    return
  }

  emit('action', view.primaryAction)
}

const runSecondary = (action: TaskActionDescriptor) => {
  emit('action', action.kind)
}

const tablerIcon = (icon: string): string => {
  const aliases: Record<string, string> = {
    pause: 'player-pause',
    play: 'player-play',
    refresh: 'refresh',
    x: 'x',
    trash: 'trash',
    file: 'file',
    folder: 'folder',
    copy: 'copy',
    more: 'dots',
  }

  return `i-tabler-${aliases[icon] ?? icon}`
}
</script>

<template>
  <div class="task-action-menu">
    <UiIconButton
      v-if="showPrimary && hasPrimaryAction"
      class="primary-action"
      :icon="view.primaryActionIcon"
      :label="view.primaryActionLabel"
      variant="ghost"
      size="compact"
      :disabled
      @click.stop="runPrimary"
    />
    <UDropdownMenu
      v-if="dropdownItems.length"
      :items="dropdownItems"
      :disabled
      :content="{ align: 'end', sideOffset: 4, collisionPadding: 12 }"
      :ui="{ content: 'min-w-32' }"
    >
      <UiIconButton icon="more" label="更多操作" variant="ghost" size="compact" :disabled />
    </UDropdownMenu>
  </div>
</template>

<style scoped>
.task-action-menu {
  min-width: 0;
  display: inline-flex;
  align-items: center;
  justify-content: flex-end;
  gap: 2px;
}

</style>
