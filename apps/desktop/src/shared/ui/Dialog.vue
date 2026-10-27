<script setup lang="ts">
import { computed } from 'vue';
import { isMobilePlatform } from '../platform/environment';

const model = defineModel<boolean>({ default: false });
const {
  title,
  description,
  fixedHeight = false,
  size = 'default',
  dismissible = true,
  showClose = true,
} = defineProps<{
  title: string;
  description?: string;
  fixedHeight?: boolean;
  size?: 'compact' | 'default' | 'wide';
  dismissible?: boolean;
  showClose?: boolean;
}>();

const modalUi = computed(() => ({
  overlay: isMobilePlatform()
    ? 'bdl-dialog-overlay bdl-dialog-mobile-overlay bg-black/35 dark:bg-black/60'
    : 'bdl-dialog-overlay bg-black/35 dark:bg-black/60',
  content: [
    'bdl-dialog-content',
    size === 'wide' ? 'max-w-[57.5rem]' : size === 'compact' ? 'max-w-[24rem]' : 'max-w-[35rem]',
    fixedHeight ? 'bdl-dialog-fixed' : '',
  ].join(' '),
  header: 'min-w-0 shrink-0 px-5 py-4 sm:px-5 sm:py-4',
  wrapper: 'min-w-0 flex-1 pr-8',
  title: 'min-w-0 truncate text-(length:--font-16) font-semibold',
  description: 'truncate text-(length:--font-12)',
  body: fixedHeight
    ? 'min-h-0 flex-1 overflow-hidden px-5 pb-4 pt-0 sm:px-5 sm:pb-4 sm:pt-0'
    : 'min-h-0 overflow-y-auto px-5 py-4 sm:px-5 sm:py-4',
  footer: 'shrink-0 justify-end flex-wrap sm:flex-nowrap px-5 py-3 sm:px-5 sm:py-3',
}));
</script>

<template>
  <UModal
    v-model:open="model"
    :title
    :description
    :dismissible
    scrollable
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

<style>
/* The scrollable modal nests content in its overlay, so grid centering needs no CSS translate. */
.bdl-dialog-overlay {
  --bdl-dialog-viewport-height: 100vh;
  --bdl-dialog-gap-top: max(1rem, var(--bdl-layout-safe-area-top));
  --bdl-dialog-gap-bottom: max(1rem, var(--bdl-layout-safe-area-bottom));
  --bdl-dialog-available-height: calc(
    var(--bdl-dialog-viewport-height) - var(--bdl-dialog-gap-top) - var(--bdl-dialog-gap-bottom)
  );

  padding: var(--bdl-dialog-gap-top) max(1rem, var(--bdl-layout-safe-area-right)) var(--bdl-dialog-gap-bottom)
    max(1rem, var(--bdl-layout-safe-area-left));
}

.bdl-dialog-content {
  width: 100%;
  max-height: var(--bdl-dialog-available-height);
  overflow: hidden;
}

.bdl-dialog-fixed {
  height: 40rem;
}

.bdl-dialog-mobile-overlay {
  align-content: end;
  padding: max(2.75rem, var(--bdl-layout-safe-area-top)) 0 var(--bdl-layout-safe-area-bottom);
}

.bdl-dialog-mobile-overlay .bdl-dialog-fixed {
  height: min(44rem, var(--bdl-dialog-available-height));
}

.bdl-dialog-mobile-overlay .bdl-dialog-content {
  max-width: 100%;
  border-radius: var(--radius-12) var(--radius-12) 0 0;
}

@supports (height: 100dvh) {
  .bdl-dialog-overlay {
    --bdl-dialog-viewport-height: 100dvh;
  }
}
</style>

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
