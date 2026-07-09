<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'

import ParsePage from './pages/ParsePage.vue'
import TransferPage from './pages/TransferPage.vue'
import { useAccountStore } from './stores/account'
import { useUiStore, type AppTab } from './stores/ui'
import UiButton from './ui/Button.vue'
import UiDialog from './ui/Dialog.vue'
import UiDrawer from './ui/Drawer.vue'
import UiIconButton from './ui/IconButton.vue'
import UiSelect from './ui/Select.vue'
import UiTabs from './ui/Tabs.vue'
import UiTextarea from './ui/Textarea.vue'
import UiTextField from './ui/TextField.vue'
import UiToastHost from './ui/ToastHost.vue'

const ui = useUiStore()
const account = useAccountStore()
const outputDir = ref('')
const outputFormat = ref('mp4')
const accountMenuOpen = ref(false)
const loginDialogOpen = ref(false)
const helpDrawerOpen = ref(false)
const loginMode = ref<'qr' | 'cookie'>('qr')
const cookieText = ref('')

const navItems: Array<{ value: AppTab; label: string }> = [
  { value: 'parse', label: '解析' },
  { value: 'transfer', label: '传输' },
  { value: 'history', label: '历史' },
  { value: 'settings', label: '设置' },
]

const activeTitle = computed(() => navItems.find((item) => item.value === ui.activeTab)?.label ?? '解析')
const accountSubtitle = computed(() => `本地任务 · ${account.statusLabel}`)
const cookieSaveDisabled = computed(() => account.saving || (loginMode.value === 'cookie' && !cookieText.value.trim()))
const loginActionLabel = computed(() => (loginMode.value === 'cookie' ? '保存' : '开始扫码'))

const openLoginDialog = () => {
  accountMenuOpen.value = false
  loginDialogOpen.value = true
}

const saveLogin = async () => {
  if (loginMode.value === 'qr') {
    ui.pushToast('扫码登录暂未接入', 'info')
    return
  }

  const saved = await account.importCookie(cookieText.value)
  if (saved) {
    cookieText.value = ''
    loginDialogOpen.value = false
  }
}

const signOut = async () => {
  accountMenuOpen.value = false
  await account.logout()
}

onMounted(() => {
  void account.load()
  void account.startEventListeners()
})
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
          <p>{{ accountSubtitle }}</p>
        </div>
        <div class="top-actions">
          <UiIconButton icon="?" label="帮助" @click="helpDrawerOpen = true" />
          <div class="account-split">
            <button class="account-button" type="button" @click="openLoginDialog">
              <span class="account-avatar" aria-hidden="true">
                <img v-if="account.profile.avatar_url" :src="account.profile.avatar_url" alt="" />
                <span v-else>{{ account.avatarLabel }}</span>
              </span>
              <span>{{ account.displayName }}</span>
            </button>
            <button
              class="account-menu-button"
              type="button"
              aria-label="账户菜单"
              :aria-expanded="accountMenuOpen"
              @click="accountMenuOpen = !accountMenuOpen"
            >
              v
            </button>
            <div v-if="accountMenuOpen" class="account-popover" role="menu">
              <button type="button" role="menuitem" @click="openLoginDialog">
                {{ account.profile.logged_in ? '切换账号' : '登录' }}
              </button>
              <button v-if="account.profile.logged_in" type="button" role="menuitem" @click="signOut">退出</button>
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
        <UiButton :disabled="cookieSaveDisabled" @click="saveLogin">{{ loginActionLabel }}</UiButton>
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
