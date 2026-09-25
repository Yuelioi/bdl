<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { useAccountStore } from '../stores/account';
import { useUiStore } from '../stores/ui';
import { openExternalUrl } from '../api/tauri';
import { bilibiliUserUrl } from '../utils/bilibiliLinks';
const { mobile: isMobile = false } = defineProps<{ mobile?: boolean }>();
const account = useAccountStore();
const ui = useUiStore();
const avatarLoadFailed = ref(false);
const accountMenuItems = computed(() => {
  const accountAction = {
    label: account.profile.logged_in ? '切换账号' : '登录',
    icon: 'i-tabler-user',
    onSelect: openLoginDialog,
  };
  if (!account.profile.logged_in) return [[accountAction]];
  return [
    [
      {
        label: 'Bilibili 主页',
        icon: 'i-tabler-external-link',
        onSelect: openAccountProfile,
      },
      accountAction,
    ],
    [{ label: '退出登录', icon: 'i-tabler-logout', onSelect: () => account.logout() }],
  ];
});
const openAccountProfile = async () => {
  const url = bilibiliUserUrl(account.profile.mid);
  if (!url) return;
  try {
    await openExternalUrl(url);
  } catch (error) {
    ui.pushToast(error instanceof Error ? error.message : String(error), 'danger');
  }
};
const openLoginDialog = () => {
  ui.openLoginDialog();
};

watch(
  () => account.profile.avatar_url,
  () => {
    avatarLoadFailed.value = false;
  },
);
</script>
<template>
  <div class="account-menu" :class="{ 'mobile-account': isMobile }">
    <button
      v-if="isMobile && !account.profile.logged_in"
      class="account-button account-button-login"
      type="button"
      aria-label="登录 Bilibili 账号"
      @click="openLoginDialog"
    >
      <UIcon name="i-tabler-user" aria-hidden="true" />
      <span class="account-name">登录</span>
    </button>
    <UDropdownMenu
      v-else
      :items="accountMenuItems"
      :content="{ align: 'end', sideOffset: 6, collisionPadding: 12 }"
      :ui="{ content: 'min-w-36' }"
    >
      <button class="account-button" type="button">
        <span class="account-avatar" aria-hidden="true">
          <img
referrerpolicy="no-referrer"
            v-if="account.profile.avatar_url && !avatarLoadFailed"
            :src="account.profile.avatar_url"
            alt=""
            @error="avatarLoadFailed = true"
          />
          <span v-else>{{ account.avatarLabel }}</span>
        </span>
        <span class="account-name">{{ account.displayName }}</span>
        <UIcon name="i-tabler-chevron-down" class="account-chevron" aria-hidden="true" />
      </button>
    </UDropdownMenu>
  </div>
</template>
<style scoped>
.account-menu {
  display: flex;
  align-items: center;
}

.account-button {
  height: 30px;
  display: inline-flex;
  align-items: center;
  gap: var(--space-8);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-6);
  background: var(--color-surface);
  color: var(--color-nav-text);
  padding: 0 var(--space-8);
  font-size: var(--font-13);
  font-weight: 650;
}

.account-button:hover {
  background: var(--color-nav-raised);
}

.account-avatar {
  width: 22px;
  height: 22px;
  flex-shrink: 0;
  display: inline-grid;
  place-items: center;
  overflow: hidden;
  border-radius: 999px;
  background: var(--color-accent-soft);
  color: var(--color-accent-strong);
  font-size: var(--font-12);
}

.account-avatar img {
  width: 100%;
  height: 100%;
  display: block;
  object-fit: cover;
}

.account-chevron {
  width: 15px;
  height: 15px;
  margin-left: 2px;
  color: var(--color-muted);
}

.mobile-account .account-button {
  width: var(--height-button);
  height: var(--height-button);
  min-height: var(--height-button);
  flex-shrink: 0;
  padding: 0;
  justify-content: center;
}

.mobile-account .account-button-login {
  width: auto;
  min-width: 62px;
  min-height: 36px;
  height: 36px;
  border-color: transparent;
  background: var(--color-accent-soft);
  color: var(--color-accent-strong);
  padding: 0 var(--space-10);
}



.mobile-account .account-chevron {
  display: none;
}

.mobile-account .account-name {
  display: none;
}

.mobile-account .account-button-login .account-name {
  display: inline;
}

@media (width <= 720px) {
  .account-name {
    display: none;
  }
}
</style>
