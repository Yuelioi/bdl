<script setup lang="ts">
import { computed } from 'vue'

const model = defineModel<boolean>({ default: false })
const { title, description, fixedHeight = false, size = 'default', dismissible = true, showClose = true } = defineProps<{
  title: string
  description?: string
  fixedHeight?: boolean
  size?: 'default' | 'wide'
  dismissible?: boolean
  showClose?: boolean
}>()

const modalUi = computed(() => ({
  content: [size === 'wide' ? 'sm:max-w-[920px]' : 'sm:max-w-[520px]', fixedHeight ? 'h-[min(640px,calc(100dvh-32px))]' : ''].join(' '),
  header: 'min-w-0',
  title: 'min-w-0 truncate',
  description: 'truncate',
  body: fixedHeight ? 'min-h-0 flex-1 overflow-hidden pt-0 sm:pt-0' : 'min-h-0',
  footer: 'justify-end flex-wrap sm:flex-nowrap',
}))
</script>

<template>
  <UModal
    v-model:open="model"
    :title
    :description
    :dismissible
    :ui="modalUi"
    close-icon="i-tabler-x"
    :close="showClose ? { color: 'neutral', variant: 'ghost', size: 'sm' } : false"
  >
    <template #body>
      <div class="dialog-body" :class="{ 'dialog-body-fixed': fixedHeight }">
        <slot />
      </div>
    </template>
    <template v-if="$slots.footer" #footer>
      <slot name="footer" />
    </template>
  </UModal>
</template>

<style scoped>
.dialog-body {
  min-width: 0;
  min-height: 0;
  display: grid;
  gap: var(--space-16);
}

.dialog-body-fixed {
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.dialog-body-fixed > :deep(.ui-tabs) {
  flex-shrink: 0;
}
</style>
