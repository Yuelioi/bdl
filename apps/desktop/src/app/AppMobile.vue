<script setup lang="ts">
import { useUiStore } from '../stores/ui';
import { useNavigationStatus } from './navigation';

const navItems = [
  { value: 'parse', label: '解析', icon: 'i-tabler-link' },
  { value: 'library', label: '内容库', icon: 'i-tabler-books' },
  { value: 'transfer', label: '传输', icon: 'i-tabler-bolt' },
  { value: 'settings', label: '我的', icon: 'i-tabler-user' },
] as const;
const ui = useUiStore();
const { transferBadgeCount } = useNavigationStatus();
</script>

<template>
  <main class="app-mobile platform-mobile">
    <header class="mobile-header" aria-label="BDL">
      <span class="mobile-mark" aria-hidden="true"><i></i><i></i></span>
      <strong class="mobile-brand-name">BDL</strong>
    </header>
    <slot />
    <nav class="mobile-navigation" aria-label="主导航">
      <button
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
      </button>
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
  grid-template-columns: minmax(0, 1fr);
  grid-template-rows: auto minmax(0, 1fr) auto;
  overflow: hidden;
  background: var(--mobile-page-bg);
}

.mobile-header {
  display: flex;
  align-items: center;
  gap: 9px;
  min-height: calc(48px + var(--bdl-safe-area-top, env(safe-area-inset-top, 0)));
  padding: var(--bdl-safe-area-top, env(safe-area-inset-top, 0)) 12px 0;
  border-bottom: 1px solid var(--mobile-card-border);
  background: var(--color-nav);
}

.mobile-mark {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: flex-end;
  justify-content: center;
  gap: 3px;
  padding: 7px;
  border-radius: 10px;
  background: var(--color-accent);
}

.mobile-mark i {
  width: 5px;
  height: 10px;
  border-radius: 1px;
  background: var(--color-on-accent);
}

.mobile-mark i:last-child {
  height: 16px;
}

.mobile-brand-name {
  color: var(--color-text);
  font-size: 21px;
  font-weight: 800;
  letter-spacing: -0.02em;
  line-height: 1;
}

.mobile-navigation {
  grid-row: 3;
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: 0;
  padding: 2px calc(6px + var(--bdl-safe-area-right, 0px))
    calc(2px + var(--bdl-safe-area-bottom, env(safe-area-inset-bottom, 0px)))
    calc(6px + var(--bdl-safe-area-left, 0px));
  border-top: 1px solid var(--color-border);
  background: var(--color-nav);
}

.mobile-nav-item {
  position: relative;
  min-width: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  border: 0;
  background: transparent;
  color: var(--color-muted);
  font-weight: 650;
  min-height: 48px;
  margin-inline: 2px;
  border-radius: 12px;
  font-size: 11px;
  gap: 2px;
  transition:
    color var(--duration-fast) var(--ease-out),
    background var(--duration-fast) var(--ease-out);
}

.mobile-nav-item.active {
  background: transparent;
  color: var(--color-accent-strong);
}

.mobile-nav-icon {
  width: 22px;
  height: 22px;
}

.mobile-nav-badge {
  position: absolute;
  top: 0;
  left: calc(50% + 5px);
  min-width: 16px;
  padding: 0 4px;
  border-radius: 999px;
  background: var(--color-accent);
  color: var(--color-on-accent);
  font-size: 10px;
}

.mobile-nav-item:focus-visible {
  outline: 2px solid var(--color-accent);
  outline-offset: -2px;
}
</style>

<style src="../pages/mobile/mobile.css"></style>
