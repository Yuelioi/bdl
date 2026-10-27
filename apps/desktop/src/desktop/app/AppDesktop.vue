<script setup lang="ts">
import { getCurrentWindow } from '@tauri-apps/api/window';
import { useAccountStore } from '../../shared/stores/account';
import { useUiStore } from '../../shared/stores/ui';
import { useUpdateStore } from '../../shared/stores/update';
import { isMacPlatform } from '../../shared/platform/environment';
import AppearanceMenu from '../../shared/ui/AppearanceMenu.vue';
import AppAccountMenu from './AppAccountMenu.vue';
import { navItems, useNavigationStatus } from '../../shared/features/navigation';
import appIcon from '../../../src-tauri/icons/128x128.png';
const ui = useUiStore();
const account = useAccountStore();
const updater = useUpdateStore();
const isMacOs = isMacPlatform();
const appWindow = getCurrentWindow();
const minimizeWindow = () => {
  void appWindow.minimize();
};
const toggleMaximizeWindow = () => {
  void appWindow.toggleMaximize();
};
const closeWindow = () => {
  void appWindow.close();
};
const { transferBadgeCount, attentionCount, queueHealthLabel, aggregateSpeedLabel } = useNavigationStatus();
</script>
<template>
  <main class="app-shell" :class="{ 'platform-macos': isMacOs }">
    <header class="app-titlebar" data-tauri-drag-region @dblclick="toggleMaximizeWindow">
      <div class="titlebar-brand" data-tauri-drag-region>
        <img class="titlebar-logo" :src="appIcon" alt="" />
        <strong data-tauri-drag-region>BDL</strong>
        <span class="titlebar-version tabular-nums" data-tauri-drag-region>v{{ updater.currentVersion }}</span>
      </div>
      <div class="titlebar-actions">
        <UButton
          variant="ghost"
          color="neutral"
          :ui="{ label: 'contents' }"
          v-if="updater.hasUpdate"
          class="titlebar-update"
          type="button"
          @click="ui.setTab('settings')"
        >
          <UIcon name="i-tabler-arrow-up-circle" aria-hidden="true" />
          可更新至 v{{ updater.availableVersion }}
        </UButton>
        <AppearanceMenu />
        <AppAccountMenu />
        <div v-if="!isMacOs" class="window-controls" aria-label="窗口控制">
          <UButton
            variant="ghost"
            color="neutral"
            :ui="{ label: 'contents' }"
            type="button"
            aria-label="最小化"
            @click.stop="minimizeWindow"
          >
            <UIcon name="i-tabler-minus" aria-hidden="true" />
          </UButton>
          <UButton
            variant="ghost"
            color="neutral"
            :ui="{ label: 'contents' }"
            type="button"
            aria-label="最大化或还原"
            @click.stop="toggleMaximizeWindow"
          >
            <UIcon name="i-tabler-square" aria-hidden="true" />
          </UButton>
          <UButton
            variant="ghost"
            color="neutral"
            :ui="{ label: 'contents' }"
            class="close"
            type="button"
            aria-label="关闭"
            @click.stop="closeWindow"
          >
            <UIcon name="i-tabler-x" aria-hidden="true" />
          </UButton>
        </div>
      </div>
    </header>

    <aside class="side-nav" aria-label="主导航">
      <nav class="nav-list">
        <UButton
          variant="ghost"
          color="neutral"
          :ui="{ label: 'contents' }"
          v-for="item in navItems"
          :key="item.value"
          class="nav-item"
          :class="{ active: ui.activeTab === item.value }"
          type="button"
          :aria-current="ui.activeTab === item.value ? 'page' : undefined"
          :aria-label="`${item.label} ${item.description}${item.value === 'transfer' && transferBadgeCount > 0 ? ' ' + transferBadgeCount : ''}`"
          :title="`${item.label} · Ctrl+${item.shortcut}`"
          @click="ui.setTab(item.value)"
        >
          <UIcon :name="item.icon" class="nav-icon" aria-hidden="true" />
          <span class="nav-copy">
            <strong>{{ item.label }}</strong>
            <small class="sr-only">{{ item.description }}</small>
          </span>
          <span v-if="item.value === 'transfer' && transferBadgeCount > 0" class="nav-badge">
            {{ transferBadgeCount > 99 ? '99+' : transferBadgeCount }}
          </span>
        </UButton>
      </nav>

      <div class="nav-status">
        <span class="status-beacon" :class="{ attention: attentionCount > 0 }" aria-hidden="true"></span>
        <span>
          <strong>{{ queueHealthLabel }}</strong>
          <small class="tabular-nums">{{ aggregateSpeedLabel }} · {{ account.statusLabel }}</small>
        </span>
      </div>
    </aside>

    <slot />
  </main>
</template>
<style scoped src="./AppDesktop.css"></style>
