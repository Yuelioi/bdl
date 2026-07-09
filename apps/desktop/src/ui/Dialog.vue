<script setup lang="ts">
import { computed } from 'vue'

const model = defineModel<boolean>({ default: false })
const { title, size = 'default' } = defineProps<{
  title: string
  size?: 'default' | 'wide'
}>()

const modalUi = computed(() => ({
  content: size === 'wide' ? 'sm:max-w-[920px]' : 'sm:max-w-[520px]',
  header: 'min-w-0',
  title: 'min-w-0 truncate',
  body: 'min-h-0',
  footer: 'justify-end',
}))
</script>

<template>
  <UModal
    v-model:open="model"
    :title
    :ui="modalUi"
    close-icon="i-tabler-x"
    :close="{ color: 'neutral', variant: 'ghost', size: 'sm' }"
  >
    <template #body>
      <div class="dialog-body">
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
</style>
