<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'

import ParsePage from './pages/ParsePage.vue'
import TransferPage from './pages/TransferPage.vue'
import { useAccountStore } from './stores/account'
import {
  defaultNamingTemplate,
  namingTemplatePresets,
  namingVariables,
  useSettingsStore,
} from './stores/settings'
import { useUiStore, type AppTab } from './stores/ui'
import UiButton from './ui/Button.vue'
import UiCheckbox from './ui/Checkbox.vue'
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
const settings = useSettingsStore()
const accountMenuOpen = ref(false)
const loginDialogOpen = ref(false)
const helpDrawerOpen = ref(false)
const loginMode = ref<'qr' | 'cookie'>('qr')
const cookieText = ref('')

const navItems: Array<{ value: AppTab; label: string }> = [
  { value: 'parse', label: '解析' },
  { value: 'transfer', label: '传输' },
  { value: 'settings', label: '设置' },
]

const activeTitle = computed(() => navItems.find((item) => item.value === ui.activeTab)?.label ?? '解析')
const accountSubtitle = computed(() => `本地任务 · ${account.statusLabel}`)
const cookieSaveDisabled = computed(
  () => account.saving || account.qrLoading || (loginMode.value === 'cookie' && !cookieText.value.trim()),
)
const loginActionLabel = computed(() => {
  if (loginMode.value === 'cookie') {
    return '保存'
  }

  return account.qrSession ? '刷新二维码' : '开始扫码'
})
const settingsDownloadDir = computed({
  get: () => settings.draft.download_dir ?? '',
  set: (value: string) => settings.setDownloadDir(value),
})
const settingsArchiveMode = computed({
  get: () => settings.draft.archive_mode,
  set: (value: string) => settings.setArchiveMode(value),
})
const settingsOutputFormat = computed({
  get: () => settings.draft.output_extension,
  set: (value: string) => settings.setOutputExtension(value),
})
const settingsNamingTemplate = computed({
  get: () => settings.draft.naming_template,
  set: (value: string) => settings.setNamingTemplate(value),
})
const settingsConcurrentTasks = computed({
  get: () => String(settings.draft.concurrent_tasks),
  set: (value: string) => settings.setConcurrentTasks(value),
})
const settingsRetryCount = computed({
  get: () => String(settings.draft.retry_count),
  set: (value: string) => settings.setRetryCount(value),
})
const settingsAutoRefreshExpiredUrls = computed({
  get: () => settings.draft.auto_refresh_expired_urls,
  set: (value: boolean) => settings.setAutoRefreshExpiredUrls(value),
})
const settingsArchiveDescription = computed(() =>
  settings.draft.archive_mode === 'complete_archive'
    ? '保存最终视频，并额外生成 NFO；有封面地址时会下载封面。字幕和弹幕抓取还在任务清单中，暂不承诺完整。'
    : '只下载视频轨道和音频轨道，合并为最终可播放文件；不抓取封面、字幕、弹幕或 NFO。',
)
const resetNamingTemplate = () => {
  settings.setNamingTemplate(defaultNamingTemplate)
}

const formatNamingVariable = (name: string): string => `{${name}}`

const openLoginDialog = () => {
  accountMenuOpen.value = false
  loginDialogOpen.value = true
}

const saveLogin = async () => {
  if (loginMode.value === 'qr') {
    await account.startQrLogin()
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
  void settings.load()
  void account.startEventListeners()
})

watch(loginDialogOpen, (open) => {
  if (!open) {
    account.resetQrLogin()
  }
})

watch(loginMode, (mode) => {
  if (mode !== 'qr') {
    account.resetQrLogin()
  }
})

watch(
  () => account.profile.logged_in,
  (loggedIn) => {
    if (loggedIn && loginDialogOpen.value) {
      loginDialogOpen.value = false
    }
  },
)
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
          <UiIconButton icon="help" label="帮助" @click="helpDrawerOpen = true" />
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
              <svg aria-hidden="true" class="chevron-icon" viewBox="0 0 24 24">
                <path d="M7 10l5 5 5-5" />
              </svg>
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

      <section v-else class="page-grid settings-grid">
        <section class="panel settings-panel">
          <div class="panel-heading">
            <h2>设置</h2>
            <div class="settings-actions">
              <UiButton variant="ghost" :disabled="settings.loading || settings.saving || !settings.changed" @click="settings.resetDraft">
                撤销
              </UiButton>
              <UiButton :disabled="settings.loading || settings.saving || !settings.changed" @click="settings.save">
                {{ settings.saving ? '保存中' : '保存' }}
              </UiButton>
            </div>
          </div>

          <section class="settings-block">
            <div class="settings-block-heading">
              <h3>下载</h3>
              <span>默认保存位置和传输行为</span>
            </div>
            <div class="directory-row">
              <UiTextField v-model="settingsDownloadDir" label="保存目录" placeholder="未设置时使用 downloads" />
              <UiButton variant="secondary" :disabled="settings.loading || settings.saving" @click="settings.chooseDownloadDir">
                选择
              </UiButton>
            </div>
            <div class="settings-inline-grid">
              <UiSelect
                v-model="settingsConcurrentTasks"
                label="同时下载任务数"
                :options="[
                  { label: '1', value: '1' },
                  { label: '2', value: '2' },
                  { label: '3', value: '3' },
                  { label: '5', value: '5' },
                ]"
              />
              <UiSelect
                v-model="settingsRetryCount"
                label="失败自动重试次数"
                :options="[
                  { label: '0', value: '0' },
                  { label: '1', value: '1' },
                  { label: '3', value: '3' },
                  { label: '5', value: '5' },
                ]"
              />
            </div>
            <UiCheckbox
              v-model="settingsAutoRefreshExpiredUrls"
              label="链接过期时自动刷新"
              :disabled="settings.loading || settings.saving"
            />
          </section>

          <section class="settings-block">
            <div class="settings-block-heading">
              <h3>媒体</h3>
              <span>当前先支持封装格式，清晰度、音频和编码策略在任务清单中继续补齐</span>
            </div>
            <UiSelect
              v-model="settingsOutputFormat"
              label="封装格式"
              :options="[
                { label: 'MP4', value: 'mp4' },
                { label: 'MKV', value: 'mkv' },
              ]"
            />
          </section>

          <section class="settings-block">
            <div class="settings-block-heading">
              <h3>命名</h3>
              <span>模板会先渲染预览，保存后用于新建任务</span>
            </div>
            <UiTextField v-model="settingsNamingTemplate" label="命名模板" :placeholder="defaultNamingTemplate" />
            <div class="template-presets" aria-label="命名模板预设">
              <button
                v-for="preset in namingTemplatePresets"
                :key="preset.label"
                type="button"
                @click="settings.setNamingTemplate(preset.value)"
              >
                {{ preset.label }}
              </button>
              <button type="button" @click="resetNamingTemplate">恢复默认</button>
            </div>
            <div class="settings-preview">
              <span>预览</span>
              <code>{{ settings.namingPreview }}</code>
            </div>
            <details class="template-help">
              <summary>可用变量</summary>
              <div>
                <span v-for="variable in namingVariables" :key="variable.name" :title="variable.desc">
                  <code>{{ formatNamingVariable(variable.name) }}</code>
                  <small>{{ variable.desc }}</small>
                </span>
              </div>
            </details>
          </section>

          <section class="settings-block">
            <div class="settings-block-heading">
              <h3>归档</h3>
              <span>决定任务会保存哪些文件</span>
            </div>
            <UiSelect
              v-model="settingsArchiveMode"
              label="保存内容"
              :options="[
                { label: '快速下载：仅最终视频', value: 'fast' },
                { label: '完整归档：视频 + 可用素材', value: 'complete_archive' },
              ]"
            />
            <p class="settings-note">{{ settingsArchiveDescription }}</p>
          </section>
          <p v-if="settings.error" class="settings-error">{{ settings.error }}</p>
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
        <div class="qr-box" :class="{ active: account.qrImageSrc }">
          <img v-if="account.qrImageSrc" :src="account.qrImageSrc" alt="" />
          <span v-else>QR</span>
        </div>
        <p>{{ account.qrMessage || '等待扫码' }}</p>
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
          <span>按任务队列获取、合并并生成输出文件。</span>
        </div>
      </div>
    </UiDrawer>

    <UiToastHost />
  </main>
</template>
