<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'

import { useAccountStore } from './stores/account'
import { useQueueStore } from './stores/queue'
import { useLibraryStore } from './stores/library'
import { useSettingsStore } from './stores/settings'
import { useThemeStore } from './stores/theme'
import { useUpdateStore } from './stores/update'
import { useUiStore, type AppTab } from './stores/ui'
import { mobilePrepareNotifications, mobileSaveImageToGallery, openExternalUrl } from './api/tauri'
import { isAndroidPlatform, isMobilePlatform } from './utils/platform'
import UiButton from './ui/Button.vue'
import AppDesktop from './app/AppDesktop.vue'
import AppMobile from './app/AppMobile.vue'
import AppWorkspace from './app/AppWorkspace.vue'
import UiDialog from './ui/Dialog.vue'
import UiEnvironmentHealthPanel from './ui/EnvironmentHealthPanel.vue'
import UiTabs from './ui/Tabs.vue'
import UiTextarea from './ui/Textarea.vue'
import UiToastHost from './ui/ToastHost.vue'
import UsageNotice from './ui/UsageNotice.vue'

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
const usageNoticeOpen = ref(false)
const startupRecoveryPending = ref(false)
const loginMode = ref<'qr' | 'cookie'>('qr')
const cookieText = ref('')
const qrImageSaving = ref(false)
const isMobile = isMobilePlatform()
const isAndroid = isAndroidPlatform()

const cookieSaveDisabled = computed(() => account.saving || !cookieText.value.trim())
const startupRecoveryCount = computed(() => queue.startupRecovery?.task_ids.length ?? 0)
const openUsageNoticeLink = async (url: string) => {
  try {
    await openExternalUrl(url)
  } catch (error) {
    ui.pushToast(error instanceof Error ? error.message : String(error), 'danger')
  }
}

const acknowledgeUsageNotice = () => {
  void settings.saveAppPreferences({ usage_notice_acknowledged: true })
  usageNoticeOpen.value = false
  void prepareMobileNotifications()
  if (startupRecoveryPending.value) {
    startupRecoveryPending.value = false
    startupRecoveryDialogOpen.value = true
  }
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
  await settings.ensureLoaded()
  theme.restorePreference()
  usageNoticeOpen.value = !settings.saved.usage_notice_acknowledged
  if (!usageNoticeOpen.value) void prepareMobileNotifications()
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
    <component :is="isMobile ? AppMobile : AppDesktop">
      <AppWorkspace :mobile="isMobile" />
    </component>
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
  </UApp>
</template>

<style scoped src="./App.css"></style>
