<script setup lang="ts">
import { getCurrentWindow } from '@tauri-apps/api/window'
import { computed, defineAsyncComponent, onBeforeUnmount, onMounted, ref, watch } from 'vue'

import { useAccountStore } from './stores/account'
import { useQueueStore } from './stores/queue'
import { useLibraryStore } from './stores/library'
import { useThemeStore } from './stores/theme'
import { useUpdateStore } from './stores/update'
import { useUiStore, type AppTab } from './stores/ui'
import UiButton from './ui/Button.vue'
import AppearanceMenu from './ui/AppearanceMenu.vue'
import UiDialog from './ui/Dialog.vue'
import UiTabs from './ui/Tabs.vue'
import UiTextarea from './ui/Textarea.vue'
import UiToastHost from './ui/ToastHost.vue'

const ParsePage = defineAsyncComponent(() => import('./pages/ParsePage.vue'))
const LibraryPage = defineAsyncComponent(() => import('./pages/LibraryPage.vue'))
const TransferPage = defineAsyncComponent(() => import('./pages/TransferPage.vue'))
const SettingsPage = defineAsyncComponent(() => import('./pages/SettingsPage.vue'))
const AboutPage = defineAsyncComponent(() => import('./pages/AboutPage.vue'))

const ui = useUiStore()
const account = useAccountStore()
const queue = useQueueStore()
const library = useLibraryStore()
const theme = useThemeStore()
const updater = useUpdateStore()
const loginDialogOpen = computed({
  get: () => ui.loginDialogOpen,
  set: (value: boolean) => {
    ui.loginDialogOpen = value
  },
})
const startupRecoveryDialogOpen = ref(false)
const loginMode = ref<'qr' | 'cookie'>('qr')
const cookieText = ref('')
const avatarLoadFailed = ref(false)

const navItems: Array<{ value: AppTab; label: string; description: string; icon: string; shortcut: string }> = [
  { value: 'parse', label: '解析', description: '添加与选择', icon: 'i-tabler-link', shortcut: '1' },
  { value: 'library', label: '内容库', description: '收藏与订阅', icon: 'i-tabler-books', shortcut: '2' },
  { value: 'transfer', label: '传输', description: '队列与恢复', icon: 'i-tabler-transfer', shortcut: '3' },
  { value: 'settings', label: '设置', description: '偏好与维护', icon: 'i-tabler-adjustments', shortcut: '4' },
  { value: 'about', label: '关于', description: '版本与链接', icon: 'i-tabler-info-circle', shortcut: '5' },
]

const appWindow = getCurrentWindow()
const activePageComponent = computed(() => {
  if (ui.activeTab === 'transfer') return TransferPage
  if (ui.activeTab === 'library') return LibraryPage
  if (ui.activeTab === 'settings') return SettingsPage
  if (ui.activeTab === 'about') return AboutPage
  return ParsePage
})
const transferBadgeCount = computed(
  () => queue.tasks.filter((task) => task.status !== 'completed' && task.status !== 'cancelled').length,
)
const attentionCount = computed(
  () => queue.tasks.filter((task) => task.status === 'failed' || task.status === 'cancelled').length,
)
const scheduledTaskCount = computed(
  () =>
    queue.tasks.filter(
      (task) => task.status === 'waiting' && task.scheduled_at && Date.parse(task.scheduled_at) > Date.now(),
    ).length,
)
const queueHealthLabel = computed(() => {
  if (queue.loading) return '同步队列'
  if (attentionCount.value > 0) return `${attentionCount.value} 项需处理`
  if (scheduledTaskCount.value === transferBadgeCount.value && scheduledTaskCount.value > 0) {
    return `${scheduledTaskCount.value} 项已定时`
  }
  if (transferBadgeCount.value > 0) return `${transferBadgeCount.value} 项进行中`
  return '队列空闲'
})
const aggregateSpeedLabel = computed(() => {
  const bytesPerSecond = queue.totalSpeedBytesPerSecond()
  if (bytesPerSecond <= 0 || !Number.isFinite(bytesPerSecond)) return '0 B/s'
  const units = ['B/s', 'KB/s', 'MB/s', 'GB/s']
  let value = bytesPerSecond
  let unitIndex = 0
  while (value >= 1024 && unitIndex < units.length - 1) {
    value /= 1024
    unitIndex += 1
  }
  return `${value >= 100 || unitIndex === 0 ? value.toFixed(0) : value.toFixed(1)} ${units[unitIndex]}`
})
const cookieSaveDisabled = computed(
  () => account.saving || account.qrLoading || (loginMode.value === 'cookie' && !cookieText.value.trim()),
)
const loginActionLabel = computed(() => {
  if (loginMode.value === 'cookie') {
    return '保存'
  }

  return account.qrSession ? '刷新二维码' : '开始扫码'
})
const startupRecoveryCount = computed(() => queue.startupRecovery?.task_ids.length ?? 0)
const accountMenuItems = computed(() => {
  const accountAction = {
    label: account.profile.logged_in ? '切换账号' : '登录',
    icon: 'i-tabler-user',
    onSelect: openLoginDialog,
  }
  if (!account.profile.logged_in) return [[accountAction]]
  return [[accountAction], [{ label: '退出登录', icon: 'i-tabler-logout', onSelect: signOut }]]
})
const openLoginDialog = () => {
  ui.openLoginDialog()
}

const minimizeWindow = () => {
  void appWindow.minimize()
}
const toggleMaximizeWindow = () => {
  void appWindow.toggleMaximize()
}
const closeWindow = () => {
  void appWindow.close()
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
  await account.logout()
}

const initializeApp = async () => {
  await updater.initialize()
  await account.startEventListeners()
  void account.load()
  await queue.startEventListeners()
  await queue.list()
  const recovery = await queue.loadStartupRecovery()
  if (!recovery || recovery.task_ids.length === 0 || queue.startupRecoveryDismissed) {
    return
  }

  if (recovery.auto_recovery_enabled) {
    await queue.resumeStartupRecovery()
    queue.setNotice(`已自动恢复 ${recovery.task_ids.length} 个任务`, 'success')
    return
  }

  startupRecoveryDialogOpen.value = true
}

const resumeStartupRecovery = async () => {
  await queue.resumeStartupRecovery()
  startupRecoveryDialogOpen.value = false
}

const dismissStartupRecovery = async () => {
  await queue.dismissStartupRecovery()
  startupRecoveryDialogOpen.value = false
}

const handleAppShortcut = (event: KeyboardEvent) => {
  if (!(event.ctrlKey || event.metaKey) || event.altKey || event.shiftKey) return
  const tabByKey: Partial<Record<string, AppTab>> = {
    '1': 'parse',
    '2': 'library',
    '3': 'transfer',
    '4': 'settings',
    '5': 'about',
  }
  const tab = tabByKey[event.key]
  if (!tab) return
  event.preventDefault()
  ui.setTab(tab)
}

onMounted(() => {
  void initializeApp()
  window.addEventListener('keydown', handleAppShortcut)
})

onBeforeUnmount(() => {
  window.removeEventListener('keydown', handleAppShortcut)
  theme.dispose()
})

watch(loginDialogOpen, (open) => {
  if (!open) {
    account.resetQrLogin()
  }
})

watch(startupRecoveryDialogOpen, (open, previous) => {
  if (!open && previous && startupRecoveryCount.value > 0 && !queue.startupRecoveryDismissed) {
    void queue.dismissStartupRecovery(false)
  }
})

watch(loginMode, (mode) => {
  if (mode !== 'qr') {
    account.resetQrLogin()
  }
})

watch(
  () => account.profile.avatar_url,
  () => {
    avatarLoadFailed.value = false
  },
)

watch(
  () => account.profile.mid,
  (mid, previousMid) => {
    if (mid !== previousMid) library.clear()
  },
)

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
  <UApp>
    <main class="app-shell">
      <header class="app-titlebar" data-tauri-drag-region @dblclick="toggleMaximizeWindow">
        <div class="titlebar-brand" data-tauri-drag-region>
          <span class="titlebar-mark" aria-hidden="true"><i></i><i></i></span>
          <strong data-tauri-drag-region>BDL</strong>
          <span class="titlebar-name" data-tauri-drag-region>Bilibili Download Lab</span>
          <span class="titlebar-version tabular-nums" data-tauri-drag-region>v{{ updater.currentVersion }}</span>
        </div>
        <div class="titlebar-actions">
          <button v-if="updater.hasUpdate" class="titlebar-update" type="button" @click="ui.setTab('settings')">
            <UIcon name="i-tabler-arrow-up-circle" aria-hidden="true" />
            可更新至 v{{ updater.availableVersion }}
          </button>
          <AppearanceMenu />
          <UDropdownMenu
            :items="accountMenuItems"
            :content="{ align: 'end', sideOffset: 6, collisionPadding: 12 }"
            :ui="{ content: 'min-w-36' }"
          >
            <button class="account-button" type="button">
              <span class="account-avatar" aria-hidden="true">
                <img
                  v-if="account.profile.avatar_url && !avatarLoadFailed"
                  :src="account.profile.avatar_url"
                  alt=""
                  referrerpolicy="no-referrer"
                  @error="avatarLoadFailed = true"
                />
                <span v-else>{{ account.avatarLabel }}</span>
              </span>
              <span class="account-name">{{ account.displayName }}</span>
              <UIcon name="i-tabler-chevron-down" class="account-chevron" aria-hidden="true" />
            </button>
          </UDropdownMenu>
          <div class="window-controls" aria-label="窗口控制">
            <button type="button" aria-label="最小化" @click.stop="minimizeWindow">
              <UIcon name="i-tabler-minus" aria-hidden="true" />
            </button>
            <button type="button" aria-label="最大化或还原" @click.stop="toggleMaximizeWindow">
              <UIcon name="i-tabler-square" aria-hidden="true" />
            </button>
            <button class="close" type="button" aria-label="关闭" @click.stop="closeWindow">
              <UIcon name="i-tabler-x" aria-hidden="true" />
            </button>
          </div>
        </div>
      </header>

      <aside class="side-nav" aria-label="主导航">
        <nav class="nav-list">
          <button
            v-for="item in navItems"
            :key="item.value"
            class="nav-item"
            :class="{ active: ui.activeTab === item.value }"
            type="button"
            :aria-current="ui.activeTab === item.value ? 'page' : undefined"
            :title="`${item.label} · Ctrl+${item.shortcut}`"
            @click="ui.setTab(item.value)"
          >
            <UIcon :name="item.icon" class="nav-icon" aria-hidden="true" />
            <span class="nav-copy">
              <strong>{{ item.label }}</strong>
              <small>{{ item.description }}</small>
            </span>
            <span v-if="item.value === 'transfer' && transferBadgeCount > 0" class="nav-badge">
              {{ transferBadgeCount > 99 ? '99+' : transferBadgeCount }}
            </span>
          </button>
        </nav>

        <div class="nav-status">
          <span class="status-beacon" :class="{ attention: attentionCount > 0 }" aria-hidden="true"></span>
          <span>
            <strong>{{ queueHealthLabel }}</strong>
            <small class="tabular-nums">{{ aggregateSpeedLabel }} · {{ account.statusLabel }}</small>
          </span>
        </div>
      </aside>

      <section class="main-region" :data-page="ui.activeTab">
        <Suspense>
          <Transition name="workspace" mode="out-in">
            <KeepAlive>
              <component :is="activePageComponent" :key="ui.activeTab" />
            </KeepAlive>
          </Transition>
          <template #fallback>
            <div class="workspace-loading" role="status">
              <span></span>
              正在准备工作区
            </div>
          </template>
        </Suspense>
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

      <UiDialog v-model="startupRecoveryDialogOpen" title="恢复未完成任务">
        <p class="dialog-copy">
          检测到 {{ startupRecoveryCount }} 个上次未完成的任务。继续后会从保留的任务状态和临时文件恢复下载。
        </p>
        <template #footer>
          <UiButton variant="secondary" :disabled="queue.startupRecoveryLoading" @click="dismissStartupRecovery">
            保持暂停
          </UiButton>
          <UiButton :disabled="queue.startupRecoveryLoading" @click="resumeStartupRecovery"> 继续任务 </UiButton>
        </template>
      </UiDialog>

      <UiToastHost />
    </main>
  </UApp>
</template>

<style scoped src="./App.css"></style>
