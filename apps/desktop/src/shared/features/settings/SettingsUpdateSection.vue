<script setup lang="ts">
import { onMounted } from 'vue';
import SettingsCard from './SettingsCard.vue';
import { useUpdateStore } from '../../stores/update';
import UiButton from '../../ui/Button.vue';
import UiInlineNotice from '../../ui/InlineNotice.vue';
import UiTextField from '../../ui/TextField.vue';

const update = useUpdateStore();
onMounted(() => { void update.loadSources(); });
</script>

<template>
  <div class="settings-block">
    <SettingsCard title="版本状态">
      <div class="settings-update-heading">
        <div>
          <span>当前版本</span>
          <strong class="tabular-nums">v{{ update.currentVersion }}</strong>
        </div>
        <UiButton
          variant="secondary"
          :disabled="update.loadingSources || update.checking || update.installing"
          @click="update.checkForUpdate(false)"
        >
          {{ update.checking ? '检测中' : '检测更新' }}
        </UiButton>
      </div>

      <UiInlineNotice v-if="update.error" tone="danger">{{ update.error }}</UiInlineNotice>
      <UiInlineNotice v-if="update.warning" tone="info">{{ update.warning }}</UiInlineNotice>
      <div v-if="update.hasUpdate" class="min-w-0 rounded-lg border border-(--color-accent) bg-(--color-accent-soft) p-4">
        <div class="flex flex-wrap items-center justify-between gap-2">
          <strong class="min-w-0 wrap-anywhere">发现新版本 v{{ update.availableVersion }}</strong>
          <UiButton :disabled="update.installing" @click="update.install">
            {{ update.installing ? `更新中 ${update.progress}%` : update.source === 'quark' ? '打开夸克网盘' : update.manualDownloadUrl ? '浏览器下载' : update.android ? '下载 APK' : '下载并安装' }}
          </UiButton>
        </div>
        <p v-if="update.notes" class="mt-2 mb-0 whitespace-pre-line wrap-anywhere text-sm text-(--color-muted)">{{ update.notes }}</p>
      </div>
      <p v-if="update.android && update.hasUpdate" class="m-0 text-xs leading-5 text-(--color-muted)">
        在浏览器下载 APK 后，打开安装包完成更新。
      </p>
      <UiInlineNotice v-if="!update.hasUpdate && update.checked && !update.checking && !update.error" tone="info">
        当前已是最新版本。
      </UiInlineNotice>
    </SettingsCard>
    <SettingsCard title="更新偏好">
      <UFormField label="下载来源">
        <USelect :model-value="update.source" :items="update.sourceOptions" class="w-full" :loading="update.loadingSources" :disabled="update.loadingSources || update.checking || update.installing" @update:model-value="(value: string) => update.setSource(value as typeof update.source)" />
      </UFormField>
      <UiInlineNotice v-if="update.sourcesError" tone="danger">
        <div class="flex flex-wrap items-center justify-between gap-2">
          <span>无法读取软件站下载来源：{{ update.sourcesError }}</span>
          <UiButton variant="secondary" :disabled="update.loadingSources || update.checking" @click="update.loadSources">重试</UiButton>
        </div>
      </UiInlineNotice>
      <UiTextField
        v-if="update.source === 'mirror'" label="自定义加速地址" :model-value="update.customMirror"
        placeholder="https://你的加速站/" helper="支持地址前缀、{url}（完整链接）或 {path}（不含 https:// 的链接）模板。"
        :disabled="update.checking || update.installing" @update:model-value="value => update.setSource('mirror', value)" />
      <div v-if="update.source === 'quark'" class="grid justify-items-start gap-3">
        <p class="m-0 text-xs leading-5 text-(--color-muted)">打开夸克目录后，按版本与系统选择安装包，下载后手动安装。</p>
        <UiButton variant="secondary" :disabled="update.openingQuark" @click="update.openQuark">
          {{ update.openingQuark ? '正在打开…' : '打开夸克网盘' }}
        </UiButton>
        <UiInlineNotice v-if="update.quarkError" tone="danger">{{ update.quarkError }}</UiInlineNotice>
      </div>
      <div class="settings-update-heading update-preference-row">
        <label for="settings-check-updates">启动时检测更新</label>
        <USwitch id="settings-check-updates" :model-value="update.autoCheck" @update:model-value="update.setAutoCheck" />
      </div>
    </SettingsCard>

    <p v-if="!update.android" class="m-0 text-xs leading-5 text-(--color-muted)">
      应用内安装会校验 BDL 更新签名，并使用「下载代理」中已保存的设置。
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

.update-preference-row {
  padding-block-start: var(--space-12);
  border-block-start: 1px solid var(--color-border);
}

.settings-update-heading strong {
  margin-inline-start: var(--space-8);
  font-weight: 600;
}

.update-preference-row label {
  font-size: var(--font-13);
}
</style>
