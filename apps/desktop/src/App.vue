<script setup lang="ts">
import { computed, defineAsyncComponent, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'

import { useAccountStore } from './stores/account'
import { useQueueStore } from './stores/queue'
import { useUiStore, type AppTab } from './stores/ui'
import UiButton from './ui/Button.vue'
import UiDialog from './ui/Dialog.vue'
import UiDrawer from './ui/Drawer.vue'
import UiIconButton from './ui/IconButton.vue'
import UiTabs from './ui/Tabs.vue'
import UiTextarea from './ui/Textarea.vue'
import UiToastHost from './ui/ToastHost.vue'

const ParsePage = defineAsyncComponent(() => import('./pages/ParsePage.vue'))
const TransferPage = defineAsyncComponent(() => import('./pages/TransferPage.vue'))
const SettingsPage = defineAsyncComponent(() => import('./pages/SettingsPage.vue'))

const ui = useUiStore()
const account = useAccountStore()
const queue = useQueueStore()
const loginDialogOpen = ref(false)
const helpDrawerOpen = ref(false)
const startupRecoveryDialogOpen = ref(false)
const loginMode = ref<'qr' | 'cookie'>('qr')
const cookieText = ref('')

const navItems: Array<{ value: AppTab; label: string; description: string; icon: string; shortcut: string }> = [
  { value: 'parse', label: '解析', description: '添加与选择', icon: 'i-tabler-link', shortcut: '1' },
  { value: 'transfer', label: '传输', description: '队列与恢复', icon: 'i-tabler-transfer', shortcut: '2' },
  { value: 'settings', label: '设置', description: '偏好与维护', icon: 'i-tabler-adjustments', shortcut: '3' },
]

const activeTitle = computed(() => navItems.find((item) => item.value === ui.activeTab)?.label ?? '解析')
const activePageComponent = computed(() => {
  if (ui.activeTab === 'transfer') return TransferPage
  if (ui.activeTab === 'settings') return SettingsPage
  return ParsePage
})
const transferBadgeCount = computed(() =>
  queue.tasks.filter((task) => task.status !== 'completed' && task.status !== 'cancelled').length,
)
const attentionCount = computed(
  () => queue.tasks.filter((task) => task.status === 'failed' || task.status === 'cancelled').length,
)
const scheduledTaskCount = computed(
  () => queue.tasks.filter((task) => task.status === 'waiting' && task.scheduled_at && Date.parse(task.scheduled_at) > Date.now()).length,
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
  const bytesPerSecond = queue.tasks.reduce(
    (total, task) => total + (queue.taskTransferProgress(task.id)?.speedBytesPerSecond ?? 0),
    0,
  )
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
  return [
    [accountAction],
    [{ label: '退出登录', icon: 'i-tabler-logout', onSelect: signOut }],
  ]
})

const openLoginDialog = () => {
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
  await account.logout()
}

const initializeApp = async () => {
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
  if (event.key.toLowerCase() === 'l') {
    event.preventDefault()
    ui.setTab('parse')
    void nextTick(() => document.querySelector<HTMLTextAreaElement>('.parse-form textarea')?.focus())
    return
  }
  const tabByKey: Partial<Record<string, AppTab>> = { '1': 'parse', '2': 'transfer', '3': 'settings' }
  const tab = tabByKey[event.key]
  if (!tab) return
  event.preventDefault()
  ui.setTab(tab)
}

onMounted(() => {
  void initializeApp()
  window.addEventListener('keydown', handleAppShortcut)
})

onBeforeUnmount(() => window.removeEventListener('keydown', handleAppShortcut))

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
      <aside class="side-nav" aria-label="主导航">
        <div class="brand-block">
          <span class="brand-mark" aria-hidden="true"><i></i><i></i><i></i></span>
          <span class="brand-copy">
            <strong>BDL</strong>
            <small>Media transfer desk</small>
          </span>
        </div>

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
            <kbd v-else>⌘{{ item.shortcut }}</kbd>
          </button>
        </nav>

        <div class="nav-status">
          <span class="status-beacon" :class="{ attention: attentionCount > 0 }" aria-hidden="true"></span>
          <span>
            <strong>{{ queueHealthLabel }}</strong>
            <small>{{ aggregateSpeedLabel }} · {{ account.statusLabel }}</small>
          </span>
        </div>
      </aside>

    <section class="main-region" :data-page="ui.activeTab">
      <header class="top-bar">
        <div class="page-identity">
          <h1>{{ activeTitle }}</h1>
        </div>
        <div class="top-actions">
          <UiIconButton icon="help" label="帮助" @click="helpDrawerOpen = true" />
          <UDropdownMenu
            :items="accountMenuItems"
            :content="{ align: 'end', sideOffset: 6, collisionPadding: 12 }"
            :ui="{ content: 'min-w-36' }"
          >
            <button class="account-button" type="button">
              <span class="account-avatar" aria-hidden="true">
                <img v-if="account.profile.avatar_url" :src="account.profile.avatar_url" alt="" />
                <span v-else>{{ account.avatarLabel }}</span>
              </span>
              <span>{{ account.displayName }}</span>
              <UIcon name="i-tabler-chevron-down" class="account-chevron" aria-hidden="true" />
            </button>
          </UDropdownMenu>
        </div>
      </header>

      <Suspense>
        <Transition name="workspace" mode="out-in">
          <component :is="activePageComponent" :key="ui.activeTab" />
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
        <UiButton :disabled="queue.startupRecoveryLoading" @click="resumeStartupRecovery">
          继续任务
        </UiButton>
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
  </UApp>
</template>
