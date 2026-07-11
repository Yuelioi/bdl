<script setup lang="ts">
import { getVersion } from '@tauri-apps/api/app'
import { onMounted, ref } from 'vue'

import { openExternalUrl } from '../api/tauri'
import { useUiStore } from '../stores/ui'
import UiButton from '../ui/Button.vue'

const ui = useUiStore()
const version = ref('0.1.0')
const links = [
  {
    label: 'GitHub 仓库',
    value: 'github.com/Yuelioi/bdl',
    url: 'https://github.com/Yuelioi/bdl',
    icon: 'i-tabler-brand-github',
  },
  {
    label: 'Bilibili 主页',
    value: 'space.bilibili.com/4279370',
    url: 'https://space.bilibili.com/4279370',
    icon: 'i-tabler-brand-bilibili',
  },
  {
    label: '个人网站',
    value: 'www.yuelili.com',
    url: 'https://www.yuelili.com',
    icon: 'i-tabler-world-www',
  },
]

const openLink = async (url: string) => {
  try {
    await openExternalUrl(url)
  } catch (error) {
    ui.pushToast(error instanceof Error ? error.message : String(error), 'danger')
  }
}

onMounted(async () => {
  try {
    version.value = await getVersion()
  } catch {
    // The package version remains available in browser-only previews.
  }
})
</script>

<template>
  <section class="page-grid grid-cols-1">
    <section class="panel mx-auto w-full max-w-3xl gap-8 p-8">
      <header class="flex min-w-0 items-start gap-4 border-b border-(--color-border) pb-6">
        <span class="grid size-12 shrink-0 place-items-center rounded-xl bg-(--color-accent) text-(--color-on-accent)" aria-hidden="true">
          <UIcon class="size-6" name="i-tabler-chart-bar" />
        </span>
        <div class="grid min-w-0 gap-1">
          <span class="text-xs font-bold text-(--color-accent-strong)">BILIBILI DOWNLOAD LAB</span>
          <h1 class="text-balance m-0 text-2xl text-(--color-text-strong)">BDL</h1>
          <p class="text-pretty m-0 max-w-2xl text-sm leading-6 text-(--color-muted)">
            一个专注解析、选择和稳定下载的 Bilibili 桌面工具。
          </p>
        </div>
      </header>

      <dl class="m-0 grid grid-cols-[120px_minmax(0,1fr)] gap-x-6 gap-y-3 text-sm">
        <dt class="text-(--color-muted)">当前版本</dt>
        <dd class="tabular-nums m-0 font-bold text-(--color-text)">v{{ version }}</dd>
        <dt class="text-(--color-muted)">应用名称</dt>
        <dd class="m-0 font-bold text-(--color-text)">Bilibili Download Lab</dd>
      </dl>

      <section class="grid gap-3" aria-labelledby="creator-links-heading">
        <h2 id="creator-links-heading" class="m-0 text-base text-(--color-text)">作者链接</h2>
        <div class="grid gap-2">
          <div
            v-for="link in links"
            :key="link.url"
            class="flex min-w-0 items-center gap-3 rounded-lg border border-(--color-border) bg-(--color-surface) p-3"
          >
            <UIcon :name="link.icon" class="size-5 shrink-0 text-(--color-accent-strong)" aria-hidden="true" />
            <span class="grid min-w-0 flex-1 gap-0.5">
              <strong class="text-[13px] text-(--color-text)">{{ link.label }}</strong>
              <span class="truncate text-xs text-(--color-muted)">{{ link.value }}</span>
            </span>
            <UiButton variant="secondary" size="compact" @click="openLink(link.url)">打开</UiButton>
          </div>
        </div>
      </section>
    </section>
  </section>
</template>
