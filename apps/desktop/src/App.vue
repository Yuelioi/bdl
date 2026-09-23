<script setup lang="ts">
import { getCurrentWindow } from '@tauri-apps/api/window'
import { computed, defineAsyncComponent, onBeforeUnmount, onMounted, ref, watch } from 'vue'

import { useAccountStore } from './stores/account'
import { useQueueStore } from './stores/queue'
import { useLibraryStore } from './stores/library'
import { useSettingsStore } from './stores/settings'
import { useThemeStore } from './stores/theme'
import { useUpdateStore } from './stores/update'
import { useUiStore, type AppTab } from './stores/ui'
import { mobilePrepareNotifications, mobileSaveImageToGallery, openExternalUrl } from './api/tauri'
import { bilibiliUserUrl } from './utils/bilibiliLinks'
import { isAndroidPlatform, isMacPlatform, isMobilePlatform } from './utils/platform'
import BackgroundParseStatus from './ui/BackgroundParseStatus.vue'
import UiButton from './ui/Button.vue'
import AppearanceMenu from './ui/AppearanceMenu.vue'
import UiDialog from './ui/Dialog.vue'
import UiEnvironmentHealthPanel from './ui/EnvironmentHealthPanel.vue'
import UiTabs from './ui/Tabs.vue'
import UiTextarea from './ui/Textarea.vue'
import UiToastHost from './ui/ToastHost.vue'
import UsageNotice from './ui/UsageNotice.vue'

const ParsePage = defineAsyncComponent(() => import('./pages/ParsePage.vue'))
const LibraryPage = defineAsyncComponent(() => import('./pages/LibraryPage.vue'))
const TransferPage = defineAsyncComponent(() => import('./pages/TransferPage.vue'))
const SettingsPage = defineAsyncComponent(() => import('./pages/SettingsPage.vue'))
const AboutPage = defineAsyncComponent(() => import('./pages/AboutPage.vue'))

const ui = useUiStore()
const account = useAccountStore()
const queue = useQueueStore()
const library = useLibraryStore()
const settings = useSettingsStore()
const theme = useThemeStore()
const updater = useUpdateStore()
const loginDialogOpen = computed({
  get: () => ui.loginDialogOpen,
  set: (value: boolean) => {
    ui.loginDialogOpen = value
  },
})
const environmentDialogOpen = computed({
  get: () => ui.environmentDialogOpen,
  set: (value: boolean) => {
    ui.environmentDialogOpen = value
  },
})
const startupRecoveryDialogOpen = ref(false)
const usageNoticeStorageKey = 'bdl.usage-notice.v1'
const usageNoticeOpen = ref(window.localStorage.getItem(usageNoticeStorageKey) !== 'acknowledged')
const startupRecoveryPending = ref(false)
const loginMode = ref<'qr' | 'cookie'>('qr')
const cookieText = ref('')
const qrImageSaving = ref(false)
const avatarLoadFailed = ref(false)
const isMacOs = isMacPlatform()
const isMobile = isMobilePlatform()
const isAndroid = isAndroidPlatform()

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
const cookieSaveDisabled = computed(() => account.saving || !cookieText.value.trim())
const startupRecoveryCount = computed(() => queue.startupRecovery?.task_ids.length ?? 0)
const accountMenuItems = computed(() => {
  const accountAction = {
    label: account.profile.logged_in ? '切换账号' : '登录',
    icon: 'i-tabler-user',
    onSelect: openLoginDialog,
  }
  if (!account.profile.logged_in) return [[accountAction]]
  return [
    [
      {
        label: 'Bilibili 主页',
        icon: 'i-tabler-external-link',
        onSelect: openAccountProfile,
      },
      accountAction,
    ],
    [{ label: '退出登录', icon: 'i-tabler-logout', onSelect: signOut }],
  ]
})
const openAccountProfile = async () => {
  const url = bilibiliUserUrl(account.profile.mid)
  if (!url) return
  try {
    await openExternalUrl(url)
  } catch (error) {
    ui.pushToast(error instanceof Error ? error.message : String(error), 'danger')
  }
}
const openLoginDialog = () => {
  ui.openLoginDialog()
}

const openUsageNoticeLink = async (url: string) => {
  try {
    await openExternalUrl(url)
  } catch (error) {
    ui.pushToast(error instanceof Error ? error.message : String(error), 'danger')
  }
}

const acknowledgeUsageNotice = () => {
  window.localStorage.setItem(usageNoticeStorageKey, 'acknowledged')
  usageNoticeOpen.value = false
  void prepareMobileNotifications()
  if (startupRecoveryPending.value) {
    startupRecoveryPending.value = false
    startupRecoveryDialogOpen.value = true
  }
}

const minimizeWindow = () => {
  if (isMobile) return
  void appWindow.minimize()
}
const toggleMaximizeWindow = () => {
  if (isMobile) return
  void appWindow.toggleMaximize()
}
const closeWindow = () => {
  if (isMobile) return
  void appWindow.close()
}

const saveCookieLogin = async () => {
  const saved = await account.importCookie(cookieText.value)
  if (saved) {
    cookieText.value = ''
    loginDialogOpen.value = false
  }
}

const startQrLogin = () => {
  void account.startQrLogin()
}

const qrSvgToPngBase64 = async (svg: string): Promise<string> => {
  const image = new Image()
  const source = `data:image/svg+xml;charset=utf-8,${encodeURIComponent(svg)}`
  await new Promise<void>((resolve, reject) => {
    image.onload = () => resolve()
    image.onerror = () => reject(new Error('二维码图片转换失败'))
    image.src = source
  })
  const canvas = document.createElement('canvas')
  canvas.width = 1024
  canvas.height = 1024
  const context = canvas.getContext('2d')
  if (!context) throw new Error('当前设备无法生成二维码图片')
  context.fillStyle = '#ffffff'
  context.fillRect(0, 0, canvas.width, canvas.height)
  context.imageSmoothingEnabled = false
  context.drawImage(image, 0, 0, canvas.width, canvas.height)
  const dataUrl = canvas.toDataURL('image/png')
  const separator = dataUrl.indexOf(',')
  if (separator < 0) throw new Error('二维码图片编码失败')
  return dataUrl.slice(separator + 1)
}

const saveQrImageToGallery = async () => {
  if (!isAndroid || !account.qrSession || qrImageSaving.value) return
  qrImageSaving.value = true
  try {
    const imageBase64 = await qrSvgToPngBase64(account.qrSession.qr_image_svg)
    await mobileSaveImageToGallery(`BDL-Bilibili-login-${Date.now()}.png`, imageBase64)
    ui.pushToast('二维码已保存到相册', 'success')
  } catch (error) {
    ui.pushToast(error instanceof Error ? error.message : String(error), 'danger')
  } finally {
    qrImageSaving.value = false
  }
}

const signOut = async () => {
  await account.logout()
}

const openEnvironmentSettings = () => {
  environmentDialogOpen.value = false
  ui.setTab('settings')
}

const chooseDownloadDirectoryFromEnvironment = async () => {
  if (await settings.chooseDownloadDir()) await settings.save()
}

const chooseFfmpegFromEnvironment = async () => {
  if (await settings.chooseFfmpegPath()) await settings.save()
}

const useSystemFfmpegFromEnvironment = async () => {
  settings.clearFfmpegPath()
  await settings.save()
}

const prepareMobileNotifications = async () => {
  if (!isAndroid) return

  try {
    const permission = await mobilePrepareNotifications()
    if (permission === 'denied' || permission === 'prompt-with-rationale') {
      ui.pushToast(
        '未开启通知权限，后台下载状态和定时任务提醒可能不会显示。可在系统设置中开启 BDL 通知。',
        'warning',
      )
    }
  } catch (error) {
    ui.pushToast(error instanceof Error ? error.message : String(error), 'warning')
  }
}

const initializeApp = async () => {
  await settings.initializeEnvironment()

  await updater.initialize(!isMobile)
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

  if (usageNoticeOpen.value) {
    startupRecoveryPending.value = true
  } else {
    startupRecoveryDialogOpen.value = true
  }
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

const preventNativeContextMenu = (event: MouseEvent) => {
  event.preventDefault()
}

onMounted(() => {
  void initializeApp()
  if (!usageNoticeOpen.value) void prepareMobileNotifications()
  window.addEventListener('keydown', handleAppShortcut)
  document.addEventListener('contextmenu', preventNativeContextMenu)
})

onBeforeUnmount(() => {
  window.removeEventListener('keydown', handleAppShortcut)
  document.removeEventListener('contextmenu', preventNativeContextMenu)
  theme.dispose()
})

watch(loginDialogOpen, (open) => {
  if (!open) account.resetQrLogin()
})

watch(startupRecoveryDialogOpen, (open, previous) => {
  if (!open && previous && startupRecoveryCount.value > 0 && !queue.startupRecoveryDismissed) {
    void queue.dismissStartupRecovery(false)
  }
})

watch(loginMode, (mode) => {
  if (mode !== 'qr') account.resetQrLogin()
})

watch(
  () => settings.environmentReady,
  (ready) => {
    if (ready) environmentDialogOpen.value = false
  },
)

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
    <main class="app-shell" :class="{ 'platform-macos': isMacOs, 'platform-mobile': isMobile }">
      <header class="app-titlebar" data-tauri-drag-region @dblclick="toggleMaximizeWindow">
        <div class="titlebar-brand" data-tauri-drag-region>
          <span class="titlebar-mark" aria-hidden="true"><i></i><i></i></span>
          <strong data-tauri-drag-region>BDL</strong>
          <span class="titlebar-name" data-tauri-drag-region>Bilibili Download Lab</span>
          <span class="titlebar-version tabular-nums" data-tauri-drag-region>v{{ updater.currentVersion }}</span>
        </div>
        <div class="titlebar-actions">
          <button
            v-if="!isMobile && updater.hasUpdate"
            class="titlebar-update"
            type="button"
            @click="ui.setTab('settings')"
          >
            <UIcon name="i-tabler-arrow-up-circle" aria-hidden="true" />
            可更新至 v{{ updater.availableVersion }}
          </button>
          <AppearanceMenu />
          <button
            v-if="isMobile && !account.profile.logged_in"
            class="account-button account-button-login"
            type="button"
            aria-label="登录 Bilibili 账号"
            @click="openLoginDialog"
          >
            <UIcon name="i-tabler-user" aria-hidden="true" />
            <span class="account-name">登录</span>
          </button>
          <UDropdownMenu
            v-else
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
          <div v-if="!isMacOs && !isMobile" class="window-controls" aria-label="窗口控制">
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
        <BackgroundParseStatus />
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

      <UiDialog v-model="loginDialogOpen" title="登录" :dismissible="false">
        <UiTabs
          v-model="loginMode"
          :tabs="[
            { label: '扫码', value: 'qr' },
            { label: 'Cookie', value: 'cookie' },
          ]"
        />
        <div v-if="loginMode === 'qr'" class="qr-panel" :aria-busy="account.qrLoading">
          <div v-if="account.qrImageSrc" class="qr-stage">
            <div class="qr-box active">
              <img :src="account.qrImageSrc" alt="用于登录 Bilibili 的二维码" />
            </div>
            <button
              class="qr-refresh-button"
              type="button"
              aria-label="刷新二维码"
              title="刷新二维码"
              @click="startQrLogin"
            >
              <UIcon name="i-tabler-refresh" class="qr-refresh-icon" aria-hidden="true" />
            </button>
          </div>
          <UiButton
            v-if="isAndroid && account.qrImageSrc"
            variant="secondary"
            size="compact"
            :disabled="qrImageSaving"
            @click="saveQrImageToGallery"
          >
            <UIcon :name="qrImageSaving ? 'i-tabler-loader-2' : 'i-tabler-photo-down'" :class="['size-4', { 'qr-spinner': qrImageSaving }]" aria-hidden="true" />
            {{ qrImageSaving ? '保存中' : '保存到相册' }}
          </UiButton>
          <button
            v-if="!account.qrImageSrc"
            class="qr-box qr-box-action"
            :class="{ error: account.qrError }"
            type="button"
            :disabled="account.qrLoading"
            :aria-label="account.qrError ? '重新获取二维码' : '获取登录二维码'"
            @click="startQrLogin"
          >
            <UIcon
              :name="account.qrLoading ? 'i-tabler-loader-2' : 'i-tabler-refresh'"
              class="qr-action-icon"
              :class="{ 'qr-spinner': account.qrLoading }"
              aria-hidden="true"
            />
            <span>{{ account.qrLoading ? '正在获取' : account.qrError ? '重新获取' : '获取二维码' }}</span>
          </button>
          <p
            :class="{ 'qr-error-message': account.qrError }"
            :role="account.qrError ? 'alert' : 'status'"
            aria-live="polite"
            aria-atomic="true"
          >
            {{ account.qrMessage || (account.qrLoading ? '正在生成二维码' : '点击上方获取登录二维码') }}
          </p>
        </div>
        <UiTextarea v-else v-model="cookieText" label="Cookie" placeholder="SESSDATA=..." />
        <template #footer>
          <UiButton variant="secondary" @click="loginDialogOpen = false">取消</UiButton>
          <UiButton v-if="loginMode === 'cookie'" :disabled="cookieSaveDisabled" @click="saveCookieLogin">保存</UiButton>
        </template>
      </UiDialog>

      <UiDialog
        v-model="usageNoticeOpen"
        title="使用前请阅读"
        description="免费软件 · 合法使用 · 禁止滥用"
        :dismissible="false"
        :show-close="false"
      >
        <UsageNotice @open-link="openUsageNoticeLink" />
        <template #footer>
          <UiButton @click="acknowledgeUsageNotice">我已阅读并了解，继续使用</UiButton>
        </template>
      </UiDialog>

      <UiDialog v-model="environmentDialogOpen" title="下载环境未就绪">
        <p class="dialog-copy">
          目录与 FFmpeg 状态供参考，不影响解析。开始下载时会自动创建缺失的目录，失败时会在任务中提示。
        </p>
        <UiEnvironmentHealthPanel
          :health="settings.environmentHealth"
          :checking="settings.environmentChecking"
          @check="settings.checkEnvironment"
          @choose-directory="chooseDownloadDirectoryFromEnvironment"
          @choose-ffmpeg="chooseFfmpegFromEnvironment"
          @use-system-ffmpeg="useSystemFfmpegFromEnvironment"
        />
        <template #footer>
          <UiButton variant="secondary" @click="environmentDialogOpen = false">稍后处理</UiButton>
          <UiButton @click="openEnvironmentSettings">前往设置</UiButton>
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
