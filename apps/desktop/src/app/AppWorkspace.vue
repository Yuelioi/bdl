<script setup lang="ts">
import { computed, defineAsyncComponent } from 'vue';
import { useUiStore } from '../shared/stores/ui';
import BackgroundParseStatus from '../shared/ui/BackgroundParseStatus.vue';
import PreparationStatus from '../shared/ui/PreparationStatus.vue';
const { mobile = false } = defineProps<{ mobile?: boolean }>();
const ui = useUiStore();
const desktopPages = {
  parse: defineAsyncComponent(() => import('../desktop/pages/ParsePage.vue')),
  library: defineAsyncComponent(() => import('../desktop/pages/LibraryPage.vue')),
  transfer: defineAsyncComponent(() => import('../desktop/pages/TransferPage.vue')),
  settings: defineAsyncComponent(() => import('../desktop/pages/SettingsPage.vue')),
  about: defineAsyncComponent(() => import('../desktop/pages/AboutPage.vue')),
};
const mobilePages = {
  parse: defineAsyncComponent(() => import('../mobile/pages/ParsePage.vue')),
  library: defineAsyncComponent(() => import('../mobile/pages/LibraryPage.vue')),
  transfer: defineAsyncComponent(() => import('../mobile/pages/TransferPage.vue')),
  settings: defineAsyncComponent(() => import('../mobile/pages/PersonalPage.vue')),
  about: defineAsyncComponent(() => import('../mobile/pages/AboutPage.vue')),
};
const activePageComponent = computed(() => (mobile ? mobilePages : desktopPages)[ui.activeTab]);
</script>
<template>
  <section class="main-region" :data-page="ui.activeTab" :class="{ 'workspace-mobile': mobile }">
    <div class="workspace-status">
      <PreparationStatus />
      <BackgroundParseStatus />
    </div>
    <Suspense>
      <Transition name="workspace" mode="out-in">
        <KeepAlive>
          <component :is="activePageComponent" :key="ui.activeTab" />
        </KeepAlive>
      </Transition>
      <template #fallback>
        <div class="workspace-loading" role="status">
          <span></span>
          正在准备工作区
        </div>
      </template>
    </Suspense>
  </section>
</template>
<style scoped>
.main-region {
  grid-column: 2;
  grid-row: 2;
  min-width: 0;
  min-height: 0;
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
  overflow: hidden;
}

.workspace-status {
  min-width: 0;
}

.workspace-loading {
  display: grid;
  place-content: center;
  justify-items: center;
  gap: var(--space-sm);
  color: var(--color-muted);
  font-size: var(--font-13);
}

.workspace-loading span {
  width: 34px;
  height: 3px;
  overflow: hidden;
  border-radius: 2px;
  background: var(--color-panel-strong);
}

.workspace-loading span::after {
  width: 50%;
  height: 100%;
  display: block;
  background: var(--color-accent);
  content: '';
  animation: loading-track 900ms var(--ease-out) infinite alternate;
}

.workspace-enter-active,
.workspace-leave-active {
  transition:
    opacity var(--duration-base) var(--ease-out),
    transform var(--duration-base) var(--ease-out);
}

.workspace-enter-from {
  opacity: 0;
  transform: translateY(5px);
}

.workspace-leave-to {
  opacity: 0;
  transform: translateY(-3px);
}

@keyframes loading-track {
  from {
    transform: translateX(-100%);
  }

  to {
    transform: translateX(200%);
  }
}

.main-region.workspace-mobile {
  grid-area: workspace;
  padding: 0;
}
</style>
