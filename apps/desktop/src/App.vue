<script setup lang="ts">
import { computed, ref } from 'vue'

import ParsePage from './pages/ParsePage.vue'
import TransferPage from './pages/TransferPage.vue'
import UiButton from './ui/Button.vue'
import UiDialog from './ui/Dialog.vue'
import UiDrawer from './ui/Drawer.vue'
import UiIconButton from './ui/IconButton.vue'
import UiSelect from './ui/Select.vue'
import UiTabs from './ui/Tabs.vue'
import UiTextarea from './ui/Textarea.vue'
import UiTextField from './ui/TextField.vue'
import UiToastHost from './ui/ToastHost.vue'
import { useUiStore, type AppTab } from './stores/ui'

const ui = useUiStore()
const outputDir = ref('')
const outputFormat = ref('mp4')
const accountMenuOpen = ref(false)
const loginDialogOpen = ref(false)
const helpDrawerOpen = ref(false)
const loginMode = ref('qr')
const cookieText = ref('')

const navItems: Array<{ value: AppTab; label: string }> = [
  { value: 'parse', label: '解析' },
  { value: 'transfer', label: '传输' },
  { value: 'history', label: '历史' },
  { value: 'settings', label: '设置' },
]

const activeTitle = computed(() => navItems.find((item) => item.value === ui.activeTab)?.label ?? '解析')

const openLoginDialog = () => {
  accountMenuOpen.value = false
  loginDialogOpen.value = true
}

const signOut = () => {
  accountMenuOpen.value = false
  ui.pushToast('已退出登录', 'info')
}

</script>

<template>
  <main class="app-shell">
    <aside class="side-nav" aria-label="主导航">
      <div class="brand-block">
        <span class="brand-mark">BDL</span>
        <span class="brand-name">Bilibili Downloader</span>
      </div>

      <nav class="nav-list">
        <button
          v-for="item in navItems"
          :key="item.value"
          class="nav-item"
          :class="{ active: ui.activeTab === item.value }"
          type="button"
          @click="ui.setTab(item.value)"
        >
          {{ item.label }}
        </button>
      </nav>
    </aside>

    <section class="main-region">
      <header class="top-bar">
        <div>
          <h1>{{ activeTitle }}</h1>
          <p>本地任务 · 未登录</p>
        </div>
        <div class="top-actions">
          <UiIconButton icon="?" label="帮助" @click="helpDrawerOpen = true" />
          <div class="account-split">
            <button class="account-button" type="button" @click="openLoginDialog">
              <span class="account-avatar" aria-hidden="true">未</span>
              <span>登录</span>
            </button>
            <button
              class="account-menu-button"
              type="button"
              aria-label="账户菜单"
              @click="accountMenuOpen = !accountMenuOpen"
            >
              v
            </button>
            <div v-if="accountMenuOpen" class="account-popover" role="menu">
              <button type="button" role="menuitem" @click="openLoginDialog">登录</button>
              <button type="button" role="menuitem" @click="signOut">退出</button>
            </div>
          </div>
        </div>
      </header>

      <ParsePage v-if="ui.activeTab === 'parse'" />

      <TransferPage v-else-if="ui.activeTab === 'transfer'" />

      <section v-else-if="ui.activeTab === 'history'" class="page-grid single-grid">
        <section class="panel empty-panel">
          <h2>历史</h2>
          <p>完成记录将在这里按时间排列。</p>
        </section>
      </section>

      <section v-else class="page-grid settings-grid">
        <section class="panel">
          <div class="panel-heading">
            <h2>下载</h2>
          </div>
          <UiTextField v-model="outputDir" label="保存目录" placeholder="D:/Downloads" />
          <UiSelect
            v-model="outputFormat"
            label="封装格式"
            :options="[
              { label: 'MP4', value: 'mp4' },
              { label: 'MKV', value: 'mkv' },
            ]"
          />
        </section>
      </section>
    </section>

    <UiDialog v-model="loginDialogOpen" title="登录">
      <UiTabs
        v-model="loginMode"
        :tabs="[
          { label: '扫码', value: 'qr' },
          { label: 'Cookie', value: 'cookie' },
        ]"
      />
      <div v-if="loginMode === 'qr'" class="qr-panel">
        <div class="qr-box">QR</div>
        <p>等待扫码</p>
      </div>
      <UiTextarea v-else v-model="cookieText" label="Cookie" placeholder="SESSDATA=..." />
      <template #footer>
        <UiButton variant="secondary" @click="loginDialogOpen = false">取消</UiButton>
        <UiButton @click="loginDialogOpen = false">保存</UiButton>
      </template>
    </UiDialog>

    <UiDrawer v-model="helpDrawerOpen" title="流程">
      <div class="flow-list">
        <div>
          <strong>解析</strong>
          <span>识别输入并拉取可选分集。</span>
        </div>
        <div>
          <strong>归一化</strong>
          <span>统一成下载任务和媒体轨道。</span>
        </div>
        <div>
          <strong>下载</strong>
          <span>按任务队列获取、合并并写入历史。</span>
        </div>
      </div>
    </UiDrawer>

    <UiToastHost />
  </main>
</template>
