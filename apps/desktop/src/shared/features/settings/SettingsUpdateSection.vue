<script setup lang="ts">
import SettingsCard from './SettingsCard.vue';
import { useUpdateStore } from '../../stores/update';
import UiButton from '../../ui/Button.vue';
import UiInlineNotice from '../../ui/InlineNotice.vue';

const update = useUpdateStore();
</script>

<template>
  <div class="grid gap-5">
    <SettingsCard aria-label="自动检测">
      <div class="settings-update-heading">
        <div>
          <h4>自动检测</h4>
          <p>默认关闭。开启后会在应用启动时静默检测新版本，不会自动下载或安装。</p>
        </div>
        <USwitch :model-value="update.autoCheck" aria-label="自动检测更新" @update:model-value="update.setAutoCheck" />
      </div>
    </SettingsCard>

    <SettingsCard aria-label="当前版本">
      <div class="settings-update-heading">
        <div>
          <h4>当前版本</h4>
          <p class="tabular-nums">v{{ update.currentVersion }}</p>
        </div>
        <UiButton
          variant="secondary"
          :disabled="update.checking || update.installing"
          @click="update.checkForUpdate(false)"
        >
          {{ update.checking ? '检测中' : '检测更新' }}
        </UiButton>
      </div>

      <UiInlineNotice v-if="update.error" tone="danger">{{ update.error }}</UiInlineNotice>
      <div v-if="update.hasUpdate" class="rounded-lg border border-(--color-accent) bg-(--color-accent-soft) p-4">
        <div class="flex flex-wrap items-start justify-between gap-4">
          <div class="grid min-w-0 flex-1 gap-1">
            <strong>发现新版本 v{{ update.availableVersion }}</strong>
            <p v-if="update.notes" class="m-0 whitespace-pre-line break-words text-sm text-(--color-muted)">{{ update.notes }}</p>
          </div>
          <UiButton :disabled="update.installing" @click="update.install">
            {{ update.android ? '下载 APK' : update.installing ? `更新中 ${update.progress}%` : '下载并安装' }}
          </UiButton>
        </div>
      </div>
      <p v-if="update.android && update.hasUpdate" class="m-0 text-xs leading-5 text-(--color-muted)">
        在浏览器下载 APK 后，打开安装包完成更新。
      </p>
      <UiInlineNotice v-if="!update.hasUpdate && update.checked && !update.checking && !update.error" tone="info">
        当前已是最新版本。
      </UiInlineNotice>
    </SettingsCard>

    <p v-if="!update.android" class="m-0 text-xs leading-5 text-(--color-muted)">
      更新包必须通过 BDL 签名验证。更新服务支持静态清单和动态服务，后续可接入国内镜像而无需改动界面。
    </p>
  </div>
</template>

<style scoped>
.settings-update-heading {
  min-width: 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-12);
}

.settings-update-heading h4,
.settings-update-heading p {
  margin: 0;
}

.settings-update-heading h4 {
  font-size: var(--font-13);
}

.settings-update-heading p {
  margin-top: var(--space-4);
  color: var(--color-muted);
  font-size: var(--font-12);
  line-height: 1.5;
}
</style>
