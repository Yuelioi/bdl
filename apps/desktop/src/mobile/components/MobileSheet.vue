<script setup lang="ts">
import { inject, ref } from 'vue';
import { mobileRailLayoutKey } from '../app/layout';
import TabletActionDialog from './TabletActionDialog.vue';

const open = defineModel<boolean>({ default: false });
defineProps<{ title: string; description?: string }>();
const railLayout = inject(mobileRailLayoutKey, ref(false));
</script>
<template>
  <TabletActionDialog v-if="railLayout" v-model="open" :title="title" :description="description">
    <slot />
  </TabletActionDialog>
  <UModal
    v-else
    v-model:open="open"
    :title="title"
    :description="description"
    fullscreen
    :ui="{ content: 'bdl-mobile-sheet', header: 'min-w-0', body: 'min-h-0 overflow-y-auto' }"
  >
    <template #body
      ><div class="mobile-sheet-body"><slot /></div
    ></template>
  </UModal>
</template>
<style>
.bdl-mobile-sheet {
  position: fixed;
  inset: auto 0 0;
  width: 100%;
  max-width: 100%;
  max-height: 85vh;
  border-radius: 0.75rem 0.75rem 0 0;
  padding-bottom: var(--bdl-layout-safe-area-bottom);
}

@supports (height: 100dvh) {
  .bdl-mobile-sheet {
    max-height: 85dvh;
  }
}

.mobile-sheet-body {
  display: grid;
  gap: 1rem;
}

.bdl-mobile-sheet .mobile-sheet-body > .ui-button {
  min-height: 2.25rem;
}

.bdl-mobile-sheet[data-state='open'] {
  animation: mobile-sheet-in 180ms ease-out;
}

@keyframes mobile-sheet-in {
  from {
    opacity: 0;
    transform: translateY(1.5rem);
  }

  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@media (prefers-reduced-motion: reduce) {
  .bdl-mobile-sheet {
    animation: none;
  }
}
</style>
