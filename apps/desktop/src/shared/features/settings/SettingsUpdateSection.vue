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
  <div class="grid gap-5">
    <SettingsCard aria-label="更新来源">
      <UFormField label="更新来源">
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
    </SettingsCard>
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
          :disabled="update.loadingSources || update.checking || update.installing"
          @click="update.checkForUpdate(false)"
        >
          {{ update.checking ? '检测中' : '检测更新' }}
        </UiButton>
      </div>

      <UiInlineNotice v-if="update.error" tone="danger">{{ update.error }}</UiInlineNotice>
      <UiInlineNotice v-if="update.warning" tone="info">{{ update.warning }}</UiInlineNotice>
      <div v-if="update.hasUpdate" class="rounded-lg border border-(--color-accent) bg-(--color-accent-soft) p-4">
        <div class="flex flex-wrap items-start justify-between gap-4">
          <div class="grid min-w-0 flex-1 gap-1">
            <strong>发现新版本 v{{ update.availableVersion }}</strong>
            <p v-if="update.notes" class="m-0 whitespace-pre-line break-words text-sm text-(--color-muted)">{{ update.notes }}</p>
          </div>
          <UiButton :disabled="update.installing" @click="update.install">
            {{ update.installing ? `更新中 ${update.progress}%` : update.source === 'quark' ? '打开夸克网盘' : update.manualDownloadUrl ? '浏览器下载' : update.android ? '下载 APK' : '下载并安装' }}
          </UiButton>
        </div>
      </div>
      <p v-if="update.android && update.hasUpdate" class="m-0 text-xs leading-5 text-(--color-muted)">
        在浏览器下载 APK 后，打开安装包完成更新。
      </p>
      <UiInlineNotice v-if="!update.hasUpdate && update.checked && !update.checking && !update.error" tone="info">
        当前来源暂无更新。
      </UiInlineNotice>
    </SettingsCard>

    <p v-if="!update.android" class="m-0 text-xs leading-5 text-(--color-muted)">
      应用内安装会校验 BDL 更新签名。网络代理沿用“网络与维护”中已保存的代理设置。
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
