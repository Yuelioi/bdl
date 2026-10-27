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
const showSettings = (cache = false) => {
  ui.settingsSection = cache ? 'settings-maintenance' : 'settings-download';
  ui.mobilePersonalPage = 'settings';
};
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
      @back="ui.mobilePersonalPage = 'home'"
    />
    <section v-else-if="ui.mobilePersonalPage === 'about'" class="personal-about">
      <UButton
        variant="ghost"
        color="neutral"
        :ui="{ label: 'contents' }"
        class="personal-back mobile-icon-button"
        type="button"
        aria-label="返回我的"
        @click="ui.mobilePersonalPage = 'home'"
      >
        <UIcon name="i-tabler-arrow-left" /></UButton
      ><AboutPage />
    </section>
    <section v-else class="mobile-page personal-page">
      <section class="profile-card">
        <UButton
          variant="ghost"
          color="neutral"
          :ui="{ label: 'contents' }"
          class="profile-account"
          type="button"
          aria-label="账号管理"
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
            ><span>{{ account.profile.logged_in ? '已登录 · 本地下载管理' : '连接你的收藏与订阅' }}</span></span
          ><UIcon name="i-tabler-chevron-right" class="row-chevron" />
        </UButton>
        <div class="profile-stats">
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
        </div>
      </section>
      <h2>常用功能</h2>
      <section class="personal-group">
        <UButton
          variant="ghost"
          color="neutral"
          :ui="{ label: 'contents' }"
          class="personal-row"
          type="button"
          @click="showSettings()"
        >
          <span class="row-icon"><UIcon name="i-tabler-settings" /></span
          ><span class="row-copy"><strong>下载设置</strong><span>目录、并发与恢复</span></span
          ><UIcon name="i-tabler-chevron-right" class="row-chevron" />
        </UButton>
        <div class="personal-row">
          <span class="row-icon"><UIcon name="i-tabler-palette" /></span
          ><span class="row-copy"><strong>外观</strong><span>主题与深色模式</span></span
          ><AppearanceMenu />
        </div>
        <UButton
          variant="ghost"
          color="neutral"
          :ui="{ label: 'contents' }"
          class="personal-row"
          type="button"
          @click="showSettings(true)"
        >
          <span class="row-icon"><UIcon name="i-tabler-trash" /></span
          ><span class="row-copy"><strong>清理与维护</strong><span>缓存、临时文件与诊断</span></span
          ><UIcon name="i-tabler-chevron-right" class="row-chevron" />
        </UButton>
      </section>
      <h2>账号与应用</h2>
      <section class="personal-group">
        <UButton
          variant="ghost"
          color="neutral"
          :ui="{ label: 'contents' }"
          class="personal-row"
          type="button"
          @click="account.profile.logged_in ? (accountOpen = true) : ui.openLoginDialog()"
        >
          <span class="row-icon"><UIcon name="i-tabler-user" /></span
          ><span class="row-copy"><strong>账号管理</strong><span>登录状态与账号信息</span></span
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
          ><span class="row-copy"><strong>关于 BDL</strong><span>版本与项目链接</span></span
          ><UIcon name="i-tabler-chevron-right" class="row-chevron" />
        </UButton>
      </section>
    </section>
    <MobileSheet v-model="accountOpen" title="账号管理" :description="account.profile.name || undefined"
      ><UiButton variant="secondary" :disabled="account.loading" @click="account.verify()">刷新账号信息</UiButton
      ><UiButton
        variant="danger"
        :disabled="account.loading"
        @click="
          account.logout();
          accountOpen = false;
        "
        >退出登录</UiButton
      ></MobileSheet
    >
  </div>
</template>
<style scoped>
.personal-workspace {
  min-height: 0;
  display: grid;
  grid-template-rows: minmax(0, 1fr);
  overflow: hidden;
}

.personal-page {
  overflow-y: auto;
  gap: 0.625rem;
}

.profile-card {
  background: var(--mobile-card);
  border-radius: var(--radius-6);
  padding: 0.625rem 0.75rem 0.4375rem;
  box-shadow: var(--mobile-card-shadow);
}

.profile-account {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 0.625rem;
  border: 0;
  padding: 0 0 0.5625rem;
  background: transparent;
  text-align: left;
  color: var(--color-text);
}

.profile-avatar {
  width: 2.75rem;
  height: 2.75rem;
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
  width: 1.6875rem;
  height: 1.6875rem;
}

.profile-copy,
.row-copy {
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
.row-copy > span {
  font-size: var(--mobile-font-caption);
  color: var(--color-muted);
}

.profile-stats {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  padding: 0.5rem 0 0;
  border-top: 0.0625rem solid var(--color-border);
}

.profile-stats button {
  display: grid;
  align-items: center;
  justify-content: center;
  gap: 0.1875rem;
  min-height: 2.375rem;
  border: 0;
  border-right: 0.0625rem solid var(--color-border);
  background: transparent;
  color: var(--color-text);
  text-align: center;
}

.profile-stats button:last-child {
  border-right: 0;
}

.profile-stats strong {
  font-family: inherit;
  font-size: var(--mobile-font-page-title);
  font-variant-numeric: tabular-nums;
}

.profile-stats button > span {
  font-size: var(--mobile-font-caption);
  color: var(--color-muted);
}

h2 {
  font-size: var(--mobile-font-section-title);
  margin: 0.4375rem 0.1875rem -0.125rem;
  font-weight: var(--mobile-weight-item);
}

.personal-group {
  padding: 0 0.75rem;
  background: var(--mobile-card);
  border-radius: var(--radius-6);
  box-shadow: var(--mobile-card-shadow);
}

.personal-row {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 0.5625rem;
  min-height: 3.25rem;
  border: 0;
  border-bottom: 0.0625rem solid var(--color-border);
  background: transparent;
  text-align: left;
  color: var(--color-text);
  padding: 0.4375rem 0;
}

.personal-row:last-child {
  border: 0;
}

.row-icon {
  display: grid;
  place-items: center;
  width: 1.5rem;
  height: 1.5rem;
  color: var(--color-accent-strong);
  background: transparent;
  border-radius: 0;
  flex-shrink: 0;
}

.row-icon :deep(svg) {
  width: 1.25rem;
  height: 1.25rem;
}

.row-copy strong {
  font-size: var(--mobile-font-item-title);
  font-weight: var(--mobile-weight-item);
}

.row-chevron {
  width: 1.125rem;
  height: 1.125rem;
  flex-shrink: 0;
  color: var(--color-muted);
}

.personal-about {
  display: flex;
  flex-direction: column;
  min-height: 0;
}

.personal-back {
  margin: 0.5rem 1rem 0;
}

.personal-row :deep(.ui-icon-button) {
  width: 1.5rem;
  height: 1.5rem;
  min-width: 2.125rem;
  min-height: 2.125rem;
  border-color: var(--mobile-card-border);
  border-radius: 0.6875rem;
}
</style>
