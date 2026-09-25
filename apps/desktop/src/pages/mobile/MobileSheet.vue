<script setup lang="ts">
const open = defineModel<boolean>({ default: false });
defineProps<{ title: string; description?: string }>();
</script>
<template>
  <UModal
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
  max-height: 85dvh;
  border-radius: 20px 20px 0 0;
  padding-bottom: var(--bdl-safe-area-bottom, env(safe-area-inset-bottom, 0));
}

.mobile-sheet-body {
  display: grid;
  gap: 16px;
}

.bdl-mobile-sheet .mobile-sheet-body button {
  min-height: 44px;
}

.bdl-mobile-sheet[data-state='open'] {
  animation: mobile-sheet-in 180ms ease-out;
}

@keyframes mobile-sheet-in {
  from {
    opacity: 0;
    transform: translateY(24px);
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
