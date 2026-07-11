<script setup lang="ts">
import { useUpdateStore } from '../../stores/update'
import UiButton from '../../ui/Button.vue'

const update = useUpdateStore()
</script>

<template>
  <div class="grid gap-5">
    <section class="settings-group">
      <div class="settings-group-heading">
        <div>
          <h4>自动检测</h4>
          <p>默认关闭。开启后会在应用启动时静默检测新版本，不会自动下载或安装。</p>
        </div>
        <USwitch :model-value="update.autoCheck" aria-label="自动检测更新" @update:model-value="update.setAutoCheck" />
      </div>
    </section>

    <section class="settings-group grid gap-3">
      <div class="settings-group-heading">
        <div>
          <h4>当前版本</h4>
          <p class="tabular-nums">v{{ update.currentVersion }}</p>
        </div>
        <UiButton variant="secondary" :disabled="update.checking || update.installing" @click="update.checkForUpdate(false)">
          {{ update.checking ? '检测中' : '检测更新' }}
        </UiButton>
      </div>

      <UiInlineNotice v-if="update.error" tone="danger">{{ update.error }}</UiInlineNotice>
      <div v-if="update.hasUpdate" class="rounded-lg border border-(--color-accent) bg-(--color-accent-soft) p-4">
        <div class="flex items-start justify-between gap-4">
          <div class="grid gap-1">
            <strong>发现新版本 v{{ update.availableVersion }}</strong>
            <p v-if="update.notes" class="m-0 whitespace-pre-line text-sm text-(--color-muted)">{{ update.notes }}</p>
          </div>
          <UiButton :disabled="update.installing" @click="update.install">
            {{ update.installing ? `更新中 ${update.progress}%` : '下载并安装' }}
          </UiButton>
        </div>
      </div>
      <UiInlineNotice v-else-if="update.checked && !update.checking && !update.error" tone="info">
        当前已是最新版本。
      </UiInlineNotice>
    </section>

    <p class="m-0 text-xs leading-5 text-(--color-muted)">
      更新包必须通过 BDL 签名验证。更新服务支持静态清单和动态服务，后续可接入国内镜像而无需改动界面。
    </p>
  </div>
</template>
