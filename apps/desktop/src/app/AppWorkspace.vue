<script setup lang="ts">
import { computed, defineAsyncComponent } from 'vue';
import { useUiStore } from '../stores/ui';
import BackgroundParseStatus from '../ui/BackgroundParseStatus.vue';
const { mobile = false } = defineProps<{ mobile?: boolean }>();
const ui = useUiStore();
const desktopPages = {
  parse: defineAsyncComponent(() => import('../pages/ParsePage.vue')),
  library: defineAsyncComponent(() => import('../pages/LibraryPage.vue')),
  transfer: defineAsyncComponent(() => import('../pages/TransferPage.vue')),
  settings: defineAsyncComponent(() => import('../pages/SettingsPage.vue')),
  about: defineAsyncComponent(() => import('../pages/AboutPage.vue')),
};
const mobilePages = {
  parse: defineAsyncComponent(() => import('../pages/mobile/ParsePage.vue')),
  library: defineAsyncComponent(() => import('../pages/mobile/LibraryPage.vue')),
  transfer: defineAsyncComponent(() => import('../pages/mobile/TransferPage.vue')),
  settings: defineAsyncComponent(() => import('../pages/mobile/PersonalPage.vue')),
  about: defineAsyncComponent(() => import('../pages/mobile/AboutPage.vue')),
};
const activePageComponent = computed(() => (mobile ? mobilePages : desktopPages)[ui.activeTab]);
</script>
<template>
  <section class="main-region" :data-page="ui.activeTab" :class="{ 'workspace-mobile': mobile }">
    <BackgroundParseStatus />
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
  grid-template-rows: minmax(0, 1fr);
  overflow: hidden;
}

.main-region:has(> [aria-label='后台解析下载']) {
  grid-template-rows: auto minmax(0, 1fr);
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
  grid-column: 1;
  grid-row: 2;
  padding: 0;
}
</style>
