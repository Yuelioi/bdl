<script setup lang="ts">
import { onMounted, onUnmounted, provide, readonly, ref } from 'vue';
import { mobileRailLayoutKey } from './layout';
import { useUiStore } from '../../shared/stores/ui';
import { useNavigationStatus } from '../../shared/features/navigation';
import { isAndroidPlatform } from '../../shared/platform/environment';
import appIcon from '../../../src-tauri/icons/128x128.png';

// Keep legacy query syntax in JS: CSS minification rewrites it to range syntax.
const railQuery = window.matchMedia(
  '(min-width: 768px), (min-width: 480px) and (max-height: 500px) and (orientation: landscape)',
);
const railLayout = ref(railQuery.matches);
provide(mobileRailLayoutKey, readonly(railLayout));
const compactQuery = window.matchMedia('(min-width: 480px) and (max-height: 500px) and (orientation: landscape)');
const compactLayout = ref(compactQuery.matches);
const updateLayout = () => {
  railLayout.value = railQuery.matches;
  compactLayout.value = compactQuery.matches;
  // High-density Android wide windows need one scale for pages and portaled UI.
  document.documentElement.classList.toggle(
    'bdl-mobile-density-scaled',
    railLayout.value && isAndroidPlatform() && window.devicePixelRatio >= 1.5,
  );
};
updateLayout();
onMounted(() => {
  updateLayout();
  for (const query of [railQuery, compactQuery]) {
    if (query.addEventListener) query.addEventListener('change', updateLayout);
    else query.addListener(updateLayout);
  }
  window.addEventListener('resize', updateLayout);
});
onUnmounted(() => {
  for (const query of [railQuery, compactQuery]) {
    if (query.removeEventListener) query.removeEventListener('change', updateLayout);
    else query.removeListener(updateLayout);
  }
  window.removeEventListener('resize', updateLayout);
  document.documentElement.classList.remove('bdl-mobile-density-scaled');
});

const navItems = [
  { value: 'parse', label: '解析', icon: 'i-tabler-link' },
  { value: 'library', label: '内容库', icon: 'i-tabler-folder' },
  { value: 'transfer', label: '传输', icon: 'i-tabler-transfer' },
  { value: 'settings', label: '我的', icon: 'i-tabler-user' },
] as const;
const ui = useUiStore();
const { transferBadgeCount } = useNavigationStatus();
</script>

<template>
  <main
    class="app-mobile platform-mobile"
    :class="{ 'mobile-layout-rail': railLayout, 'mobile-layout-compact': compactLayout }"
  >
    <header
      v-if="ui.activeTab !== 'settings' || ui.mobilePersonalPage === 'home'"
      class="mobile-header"
      aria-label="BDL"
    >
      <img class="mobile-mark" :src="appIcon" alt="" />
      <strong class="mobile-brand-name">BDL</strong>
    </header>
    <slot />
    <nav class="mobile-navigation" aria-label="主导航">
      <UButton
        variant="ghost"
        color="neutral"
        :ui="{ label: 'contents' }"
        v-for="item in navItems"
        :key="item.value"
        class="mobile-nav-item"
        :class="{ active: ui.activeTab === item.value }"
        type="button"
        :aria-current="ui.activeTab === item.value ? 'page' : undefined"
        @click="
          ui.setTab(item.value);
          if (item.value === 'settings') ui.mobilePersonalPage = 'home';
        "
      >
        <UIcon :name="item.icon" class="mobile-nav-icon" aria-hidden="true" />
        <span>{{ item.label }}</span>
        <span v-if="item.value === 'transfer' && transferBadgeCount > 0" class="mobile-nav-badge">
          {{ transferBadgeCount > 99 ? '99+' : transferBadgeCount }}
        </span>
      </UButton>
    </nav>
  </main>
</template>

<style scoped>
.app-mobile {
  width: 100%;
  height: 100%;
  min-width: 0;
  min-height: 0;
  display: grid;
  grid-template:
    'brand' auto
    'workspace' minmax(0, 1fr)
    'navigation' auto / minmax(0, 1fr);
  overflow: hidden;
  background: var(--mobile-page-bg);
}

.mobile-header {
  grid-area: brand;
  display: flex;
  align-items: center;
  gap: 0.5625rem;
  min-height: calc(2.75rem + var(--bdl-layout-safe-area-top));
  padding: var(--bdl-layout-safe-area-top) 0.75rem 0;
  border-bottom: 0.0625rem solid var(--mobile-card-border);
  background: var(--color-nav);
}

.mobile-mark {
  width: 1.5rem;
  height: 1.5rem;
  display: flex;
  align-items: flex-end;
  justify-content: center;
  gap: 0.1875rem;
  padding: 0;
  border-radius: 0.375rem;
}

.mobile-brand-name {
  color: var(--color-text);
  font-size: 1rem;
  font-weight: 600;
  letter-spacing: -0.02em;
  line-height: 1;
}

.mobile-navigation {
  grid-area: navigation;
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 0;
  padding: 0.125rem calc(0.375rem + var(--bdl-layout-safe-area-right))
    calc(0.125rem + var(--bdl-layout-safe-area-bottom)) calc(0.375rem + var(--bdl-layout-safe-area-left));
  border-top: 0.0625rem solid var(--color-border);
  background: var(--color-nav);
}

.mobile-nav-item {
  position: relative;
  min-width: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  border: 1px solid transparent;
  background: transparent;
  color: var(--color-muted);
  font-weight: 500;
  min-height: 3rem;
  margin-inline: 0.125rem;
  border-radius: 0.75rem;
  font-size: 0.6875rem;
  gap: 0.125rem;
  transition:
    color var(--duration-fast) var(--ease-out),
    background var(--duration-fast) var(--ease-out);
}

.mobile-nav-item.active {
  background: transparent;
  color: var(--color-accent-strong);
}

.mobile-nav-icon {
  width: 1.375rem;
  height: 1.375rem;
}

.mobile-nav-badge {
  position: absolute;
  top: 0;
  left: calc(50% + 0.3125rem);
  min-width: 1rem;
  padding: 0 0.25rem;
  border-radius: 62.4375rem;
  background: var(--color-accent);
  color: var(--color-on-accent);
  font-size: 0.625rem;
}

.mobile-nav-item:focus-visible {
  outline: none;
  border-color: var(--color-accent);
}

.app-mobile.mobile-layout-rail {
  grid-template:
    'brand workspace' auto
    'navigation workspace' minmax(0, 1fr) / var(--mobile-rail-width, 5rem) minmax(0, 1fr);
  padding-top: var(--bdl-layout-safe-area-top);
  padding-right: var(--bdl-layout-safe-area-right);
  padding-left: var(--bdl-layout-safe-area-left);
}

.mobile-layout-rail .mobile-header {
  flex-direction: column;
  justify-content: center;
  min-height: 4rem;
  gap: 0.3125rem;
  padding: 0.5rem 0.25rem;
  border-bottom: 0;
  border-right: 0.0625rem solid var(--color-border);
}

.mobile-layout-rail .mobile-mark {
  width: 1.75rem;
  height: 1.75rem;
  padding: 0;
  border-radius: 0.5rem;
}

.mobile-layout-rail .mobile-brand-name {
  font-size: 0.9375rem;
}

.mobile-layout-rail .mobile-navigation {
  min-height: 0;
  grid-template-columns: minmax(0, 1fr);
  grid-auto-rows: 3.25rem;
  align-content: start;
  gap: 0.25rem;
  padding: 0.5rem 0.375rem max(0.5rem, var(--bdl-layout-safe-area-bottom));
  border-top: 0;
  border-right: 0.0625rem solid var(--color-border);
  overflow-y: auto;
}

.mobile-layout-rail .mobile-nav-item {
  min-height: 3rem;
  margin-inline: 0;
  border-radius: 0.5rem;
  font-size: 0.625rem;
  font-weight: 500;
}

.mobile-layout-rail .mobile-nav-item.active {
  background: var(--color-accent-faint);
}

.mobile-layout-rail .mobile-nav-badge {
  top: 0.1875rem;
}

.mobile-layout-compact .mobile-header {
  min-height: 2.75rem;
  gap: 0.1875rem;
  padding-block: 0.25rem;
}

.mobile-layout-compact .mobile-mark {
  width: 1.375rem;
  height: 1.375rem;
  padding: 0;
  border-radius: 0.375rem;
}

.mobile-layout-compact .mobile-brand-name {
  font-size: 0.6875rem;
}

.mobile-layout-compact .mobile-navigation {
  grid-auto-rows: 2.75rem;
  gap: 0.125rem;
  padding: 0.375rem 0.25rem max(0.375rem, var(--bdl-layout-safe-area-bottom));
}

.mobile-layout-compact .mobile-nav-item {
  min-height: 2.75rem;
  font-size: 0.5625rem;
}

.mobile-layout-compact .mobile-nav-icon {
  width: 1.125rem;
  height: 1.125rem;
}
</style>

<style src="../styles/mobile.css"></style>
