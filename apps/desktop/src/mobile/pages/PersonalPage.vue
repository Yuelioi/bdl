<script setup lang="ts">
import { ref, watch } from 'vue';
import { useUiStore } from '../../shared/stores/ui';
import { useQueueStore, type QueueFilter } from '../../shared/stores/queue';
import { useAccountStore } from '../../shared/stores/account';
import { coverUrl } from '../../shared/utils/coverUrl';
import AppearanceMenu from '../../shared/ui/AppearanceMenu.vue';
import UiButton from '../../shared/ui/Button.vue';
import MobileSheet from '../components/MobileSheet.vue';
import SettingsPage from './SettingsPage.vue';
import AboutPage from './AboutPage.vue';
import { useMobileBack } from '../app/useMobileBack';
const ui = useUiStore();
const queue = useQueueStore();
const account = useAccountStore();
const accountOpen = ref(false);
const avatarFailed = ref(false);
watch(
  () => account.profile.avatar_url,
  () => {
    avatarFailed.value = false;
  },
);
const closePersonalPage = () => {
  const returnTab = ui.mobileSettingsReturnTab;
  ui.mobileSettingsReturnTab = null;
  ui.mobilePersonalPage = 'home';
  if (returnTab) ui.setTab(returnTab);
};
useMobileBack('settings', () => {
  if (ui.mobilePersonalPage !== 'about') return false;
  closePersonalPage();
  return true;
});
const showTasks = (filter: QueueFilter) => {
  queue.setFilter(filter);
  ui.setTab('transfer');
};
const stats = [
  { filter: 'active', label: '活动任务' },
  { filter: 'completed', label: '已完成' },
  { filter: 'failed', label: '失败' },
] as const;
</script>
<template>
  <div class="personal-workspace">
    <SettingsPage
      v-if="ui.mobilePersonalPage === 'settings'"
      @back="closePersonalPage"
    />
    <AboutPage v-else-if="ui.mobilePersonalPage === 'about'" @back="closePersonalPage" />
    <section v-else class="mobile-page personal-page">
      <section class="profile-card">
        <UButton
          variant="ghost"
          color="neutral"
          :ui="{ label: 'contents' }"
          class="profile-account"
          type="button"
          :aria-label="account.profile.logged_in ? '账号管理' : '登录 Bilibili'"
          @click="account.profile.logged_in ? (accountOpen = true) : ui.openLoginDialog()"
        >
          <span class="profile-avatar"
            ><img
              v-if="account.profile.avatar_url && !avatarFailed"
              referrerpolicy="no-referrer"
              :src="coverUrl(account.profile.avatar_url) ?? undefined"
              alt=""
              @error="avatarFailed = true" /><UIcon v-else name="i-tabler-user-filled"
          /></span>
          <span class="profile-copy"
            ><strong>{{
              account.profile.name || (account.profile.logged_in ? 'Bilibili 用户' : '登录 Bilibili')
            }}</strong
            ><span v-if="!account.profile.logged_in">查看收藏与订阅</span></span
          ><UIcon name="i-tabler-chevron-right" class="row-chevron" />
        </UButton>
        <nav class="profile-stats" aria-label="传输快捷入口">
          <UButton
            variant="ghost"
            color="neutral"
            :ui="{ label: 'contents' }"
            v-for="stat in stats"
            :key="stat.filter"
            type="button"
            @click="showTasks(stat.filter)"
          >
            <strong>{{ queue.countByFilter(stat.filter) }}</strong>
            <span>{{ stat.label }}</span>
          </UButton>
        </nav>
      </section>
      <section class="personal-group" aria-label="应用选项">
        <UButton
          variant="ghost"
          color="neutral"
          :ui="{ label: 'contents' }"
          class="personal-row"
          type="button"
          @click="ui.openMobileSettings('overview')"
        >
          <span class="row-icon"><UIcon name="i-tabler-settings" /></span
          ><span class="row-label">设置</span
          ><UIcon name="i-tabler-chevron-right" class="row-chevron" />
        </UButton>
        <AppearanceMenu v-slot="{ preferenceLabel }">
          <UButton
            variant="ghost"
            color="neutral"
            :ui="{ label: 'contents' }"
            class="personal-row"
            type="button"
            :aria-label="`外观：${preferenceLabel}`"
          >
            <span class="row-icon"><UIcon name="i-tabler-palette" /></span>
            <span class="row-label">外观</span>
            <span class="row-value">{{ preferenceLabel }}</span>
            <UIcon name="i-tabler-chevron-down" class="row-chevron" aria-hidden="true" />
          </UButton>
        </AppearanceMenu>
        <UButton
          variant="ghost"
          color="neutral"
          :ui="{ label: 'contents' }"
          class="personal-row"
          type="button"
          @click="ui.openMobileSettings('storage')"
        >
          <span class="row-icon"><UIcon name="i-tabler-trash" /></span
          ><span class="row-label">存储与诊断</span
          ><UIcon name="i-tabler-chevron-right" class="row-chevron" />
        </UButton>
        <UButton
          variant="ghost"
          color="neutral"
          :ui="{ label: 'contents' }"
          class="personal-row"
          type="button"
          @click="ui.mobilePersonalPage = 'about'"
        >
          <span class="row-icon"><UIcon name="i-tabler-info-circle" /></span
          ><span class="row-label">关于 BDL</span
          ><UIcon name="i-tabler-chevron-right" class="row-chevron" />
        </UButton>
      </section>
    </section>
    <MobileSheet v-model="accountOpen" title="账号管理" :description="account.profile.name || undefined">
      <UiButton variant="secondary" :disabled="account.loading" @click="account.verify()">
        <UIcon name="i-tabler-refresh" class="size-4" aria-hidden="true" />
        刷新账号信息
      </UiButton>
      <UButton
        class="account-logout-action"
        color="error"
        variant="outline"
        :disabled="account.loading"
        @click="
          account.logout();
          accountOpen = false;
        "
      >
        <UIcon name="i-tabler-logout" class="size-4" aria-hidden="true" />
        退出登录
      </UButton>
    </MobileSheet>
  </div>
</template>
<style scoped>
.personal-workspace {
  min-height: 0;
  display: grid;
  grid-template-rows: minmax(0, 1fr);
  overflow: hidden;
}

.account-logout-action {
  justify-content: center;
  min-height: 2.25rem;
}

.personal-page {
  overflow-y: auto;
  gap: var(--space-16);
}

.profile-card {
  flex-shrink: 0;
}

.profile-account {
  width: 100%;
  display: flex;
  align-items: center;
  gap: var(--space-12);
  min-height: 4rem;
  border: 0;
  padding: var(--space-8);
  text-align: left;
  color: var(--color-text);
}

.profile-avatar {
  width: 2.25rem;
  height: 2.25rem;
  display: grid;
  place-items: center;
  border-radius: 50%;
  background: var(--color-accent-faint);
  color: var(--color-accent);
  overflow: hidden;
  flex-shrink: 0;
}

.profile-avatar img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.profile-avatar :deep(svg) {
  width: 1.25rem;
  height: 1.25rem;
}

.profile-copy {
  display: grid;
  gap: 0.125rem;
  flex: 1;
  min-width: 0;
}

.profile-copy strong {
  font-size: var(--mobile-font-section-title);
  font-weight: var(--mobile-weight-title);
}

.profile-copy > span,
.row-value {
  font-size: var(--font-12);
  color: var(--color-muted);
}

.profile-stats {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  padding-block: var(--space-4);
  border-block: 0.0625rem solid var(--color-border);
}

.profile-stats button {
  display: flex;
  align-items: center;
  justify-content: center;
  flex-wrap: wrap;
  gap: 0 var(--space-6);
  min-height: 3rem;
  border: 0;
  color: var(--color-text);
  text-align: center;
}

.profile-stats strong {
  font-size: var(--font-14);
  font-variant-numeric: tabular-nums;
}

.profile-stats button > span {
  font-size: var(--font-12);
  color: var(--color-muted);
}

.personal-group {
  flex-shrink: 0;
  display: grid;
}

.personal-row {
  width: 100%;
  display: flex;
  align-items: center;
  gap: var(--space-8);
  min-height: 3rem;
  border: 0;
  border-bottom: 0.0625rem solid var(--color-border);
  border-radius: 0;
  text-align: left;
  color: var(--color-text);
  padding: var(--space-8);
}

.personal-row:last-child {
  border: 0;
}

.row-icon {
  display: grid;
  place-items: center;
  width: 1.5rem;
  height: 1.5rem;
  color: var(--color-muted);
  flex-shrink: 0;
}

.row-icon :deep(svg) {
  width: 1.25rem;
  height: 1.25rem;
}

.row-label {
  flex: 1;
  font-size: var(--font-14);
  font-weight: var(--mobile-weight-item);
}

.row-chevron {
  width: 1.125rem;
  height: 1.125rem;
  flex-shrink: 0;
  color: var(--color-muted);
}

</style>
