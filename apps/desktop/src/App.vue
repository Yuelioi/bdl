<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'

import ParsePage from './pages/ParsePage.vue'
import TransferPage from './pages/TransferPage.vue'
import { useAccountStore } from './stores/account'
import { useQueueStore } from './stores/queue'
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
const queue = useQueueStore()
const accountMenuOpen = ref(false)
const loginDialogOpen = ref(false)
const helpDrawerOpen = ref(false)
const startupRecoveryDialogOpen = ref(false)
const loginMode = ref<'qr' | 'cookie'>('qr')
const cookieText = ref('')

const navItems: Array<{ value: AppTab; label: string }> = [
  { value: 'parse', label: '解析' },
  { value: 'transfer', label: '传输' },
  { value: 'settings', label: '设置' },
]

const activeTitle = computed(() => navItems.find((item) => item.value === ui.activeTab)?.label ?? '解析')
const accountSubtitle = computed(() => `本地任务 · ${account.statusLabel}`)
const transferBadgeCount = computed(() =>
  queue.tasks.filter((task) => task.status !== 'completed' && task.status !== 'cancelled').length,
)
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
const settingsArchiveCover = computed({
  get: () => settings.draft.archive_assets.cover,
  set: (value: boolean) => settings.setArchiveAsset('cover', value),
})
const settingsArchiveSubtitles = computed({
  get: () => settings.draft.archive_assets.subtitles,
  set: (value: boolean) => settings.setArchiveAsset('subtitles', value),
})
const settingsArchiveDanmaku = computed({
  get: () => settings.draft.archive_assets.danmaku,
  set: (value: boolean) => settings.setArchiveAsset('danmaku', value),
})
const settingsArchiveNfo = computed({
  get: () => settings.draft.archive_assets.nfo,
  set: (value: boolean) => settings.setArchiveAsset('nfo', value),
})
const settingsOutputFormat = computed({
  get: () => settings.draft.output_extension,
  set: (value: string) => settings.setOutputExtension(value),
})
const settingsVideoQuality = computed({
  get: () => settings.draft.quality,
  set: (value: string) => settings.setVideoQuality(value),
})
const settingsAudioQuality = computed({
  get: () => settings.draft.audio_quality,
  set: (value: string) => settings.setAudioQuality(value),
})
const settingsCodec = computed({
  get: () => settings.draft.codec,
  set: (value: string) => settings.setCodec(value),
})
const settingsMissingQualityPolicy = computed({
  get: () => settings.draft.missing_quality_policy,
  set: (value: string) => settings.setMissingQualityPolicy(value),
})
const settingsDuplicateNamingStrategy = computed({
  get: () => settings.draft.duplicate_naming_strategy,
  set: (value: string) => settings.setDuplicateNamingStrategy(value),
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
const settingsSegmentCount = computed({
  get: () => String(settings.draft.segment_count),
  set: (value: string) => settings.setSegmentCount(value),
})
const settingsAutoRefreshExpiredUrls = computed({
  get: () => settings.draft.auto_refresh_expired_urls,
  set: (value: boolean) => settings.setAutoRefreshExpiredUrls(value),
})
const settingsStartupAutoRecovery = computed({
  get: () => settings.draft.startup_auto_recovery,
  set: (value: boolean) => settings.setStartupAutoRecovery(value),
})
const settingsFfmpegPath = computed({
  get: () => settings.draft.ffmpeg_path ?? '',
  set: (value: string) => settings.setFfmpegPath(value),
})
const settingsRetainRawStreams = computed({
  get: () => settings.draft.retain_raw_streams,
  set: (value: boolean) => settings.setRetainRawStreams(value),
})
const settingsEmbedCover = computed({
  get: () => settings.draft.embed_cover,
  set: (value: boolean) => settings.setEmbedCover(value),
})
const settingsEmbedSubtitles = computed({
  get: () => settings.draft.embed_subtitles,
  set: (value: boolean) => settings.setEmbedSubtitles(value),
})
const settingsProxyUrl = computed({
  get: () => settings.draft.proxy_url ?? '',
  set: (value: string) => settings.setProxyUrl(value),
})
const settingsLogLevel = computed({
  get: () => settings.draft.log_level,
  set: (value: string) => settings.setLogLevel(value),
})
const settingsDataDir = computed({
  get: () => settings.draft.data_dir ?? '',
  set: (value: string) => settings.setDataDir(value),
})
const selectedArchiveAssetLabels = computed(() => {
  const labels: string[] = []
  if (settings.draft.archive_assets.cover) labels.push('封面')
  if (settings.draft.archive_assets.subtitles) labels.push('字幕')
  if (settings.draft.archive_assets.danmaku) labels.push('弹幕')
  if (settings.draft.archive_assets.nfo) labels.push('NFO')
  return labels
})
const rawStreamCopy = computed(() => (settings.draft.retain_raw_streams ? '；保留原始视频/音频轨道' : ''))
const embeddingCopy = computed(() => {
  const items: string[] = []
  const canUseArchiveAssets = settings.draft.archive_mode === 'complete_archive' || settings.draft.archive_mode === 'custom'
  const coverSelected = settings.draft.archive_mode === 'complete_archive' || settings.draft.archive_assets.cover
  const subtitlesSelected = settings.draft.archive_mode === 'complete_archive' || settings.draft.archive_assets.subtitles
  if (canUseArchiveAssets && coverSelected && settings.draft.embed_cover) items.push('封面')
  if (canUseArchiveAssets && subtitlesSelected && settings.draft.embed_subtitles) items.push('字幕')
  return items.length > 0 ? `；容器支持时嵌入${items.join('和')}` : ''
})
const settingsArchiveDescription = computed(() => {
  if (settings.draft.archive_mode === 'complete_archive') {
    return `保存最终视频，并额外保存可用的封面、字幕、弹幕和 NFO${rawStreamCopy.value}${embeddingCopy.value}。不可用的素材会在任务日志中记录。`
  }

  if (settings.draft.archive_mode === 'custom') {
    const selected = selectedArchiveAssetLabels.value.length > 0 ? selectedArchiveAssetLabels.value.join('、') : '不额外保存素材'
    return `保存最终视频，并按自定义选择保存：${selected}${rawStreamCopy.value}${embeddingCopy.value}。`
  }

  return `保存最终视频${rawStreamCopy.value}${embeddingCopy.value}；不抓取封面、字幕、弹幕或 NFO。`
})
const settingsDuplicateDescription = computed(() =>
  settings.draft.duplicate_naming_strategy === 'overwrite_existing'
    ? '新任务会使用模板渲染出的原始路径；如果磁盘上已有同名文件，下载完成后会覆盖它。批量任务内部路径冲突仍会自动加后缀。'
    : '新任务遇到同名文件时自动生成“文件名 (1)”这类路径，不覆盖已有文件。',
)
const startupRecoveryCount = computed(() => queue.startupRecovery?.task_ids.length ?? 0)
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

const initializeApp = async () => {
  await account.startEventListeners()
  void account.load()
  void settings.load()
  await queue.startEventListeners()
  await queue.list()
  const recovery = await queue.loadStartupRecovery()
  if (!recovery || recovery.task_ids.length === 0 || queue.startupRecoveryDismissed) {
    return
  }

  if (recovery.auto_recovery_enabled) {
    await queue.resumeStartupRecovery()
    ui.pushToast(`已自动恢复 ${recovery.task_ids.length} 个任务`, 'success', {
      label: '查看传输',
      tab: 'transfer',
    })
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

onMounted(() => {
  void initializeApp()
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
          <span>{{ item.label }}</span>
          <span v-if="item.value === 'transfer' && transferBadgeCount > 0" class="nav-badge">
            {{ transferBadgeCount > 99 ? '99+' : transferBadgeCount }}
          </span>
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
              <UiButton :disabled="settings.loading || settings.saving || !settings.changed || Boolean(settings.namingTemplateError)" @click="settings.save">
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
              <UiSelect
                v-model="settingsSegmentCount"
                label="单任务分段数"
                :options="[
                  { label: '1 段', value: '1' },
                  { label: '2 段', value: '2' },
                  { label: '4 段', value: '4' },
                  { label: '8 段', value: '8' },
                ]"
              />
            </div>
            <UiCheckbox
              v-model="settingsAutoRefreshExpiredUrls"
              label="链接过期时自动刷新"
              :disabled="settings.loading || settings.saving"
            />
            <UiCheckbox
              v-model="settingsStartupAutoRecovery"
              label="启动时自动继续未完成任务"
              :disabled="settings.loading || settings.saving"
            />
          </section>

          <section class="settings-block">
            <div class="settings-block-heading">
              <h3>媒体</h3>
              <span>新建任务默认使用这些轨道选择和后处理设置</span>
            </div>
            <div class="settings-inline-grid">
              <UiSelect
                v-model="settingsVideoQuality"
                label="视频清晰度"
                :options="[
                  { label: '最佳可用', value: 'best' },
                  { label: '8K / 127', value: '127' },
                  { label: '4K / 120', value: '120' },
                  { label: '1080P60 / 116', value: '116' },
                  { label: '1080P+ / 112', value: '112' },
                  { label: '1080P / 80', value: '80' },
                  { label: '720P / 64', value: '64' },
                  { label: '480P / 32', value: '32' },
                  { label: '360P / 16', value: '16' },
                ]"
              />
              <UiSelect
                v-model="settingsAudioQuality"
                label="音频质量"
                :options="[
                  { label: '最佳可用', value: 'best' },
                  { label: '高音质 / 30280', value: '30280' },
                  { label: '中音质 / 30232', value: '30232' },
                  { label: '低音质 / 30216', value: '30216' },
                ]"
              />
            </div>
            <div class="settings-inline-grid">
              <UiSelect
                v-model="settingsCodec"
                label="视频编码偏好"
                :options="[
                  { label: '自动', value: 'auto' },
                  { label: 'AVC / H.264', value: 'avc' },
                  { label: 'HEVC / H.265', value: 'hevc' },
                  { label: 'AV1', value: 'av1' },
                ]"
              />
              <UiSelect
                v-model="settingsMissingQualityPolicy"
                label="目标质量不可用"
                :options="[
                  { label: '选择接近的可用质量', value: 'lower' },
                  { label: '阻止创建任务', value: 'skip' },
                  { label: '提示后再处理', value: 'ask' },
                ]"
              />
            </div>
            <UiSelect
              v-model="settingsOutputFormat"
              label="封装格式"
              :options="[
                { label: 'MP4', value: 'mp4' },
                { label: 'MKV', value: 'mkv' },
              ]"
            />
            <div class="directory-row">
              <UiTextField v-model="settingsFfmpegPath" label="FFmpeg 路径" placeholder="留空时使用系统 PATH 中的 ffmpeg" />
              <UiButton variant="secondary" :disabled="settings.loading || settings.saving" @click="settings.chooseFfmpegPath">
                选择
              </UiButton>
            </div>
            <div v-if="settings.draft.ffmpeg_path" class="settings-actions">
              <UiButton variant="ghost" :disabled="settings.loading || settings.saving" @click="settings.clearFfmpegPath">
                使用系统 FFmpeg
              </UiButton>
            </div>
            <p class="settings-note">
              编码是偏好而非硬性过滤；目标清晰度不存在时，默认会选择最接近的可用轨道。选择“提示后再处理”时，当前版本会阻止创建任务并显示原因。
            </p>
          </section>

          <section class="settings-block">
            <div class="settings-block-heading">
              <h3>命名</h3>
              <span>模板会先渲染预览，保存后用于新建任务</span>
            </div>
            <UiTextField v-model="settingsNamingTemplate" label="命名模板" :placeholder="defaultNamingTemplate" />
            <p v-if="settings.namingTemplateError" class="settings-field-error">{{ settings.namingTemplateError }}</p>
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
            <UiSelect
              v-model="settingsDuplicateNamingStrategy"
              label="重名处理"
              :options="[
                { label: '自动加后缀（推荐）', value: 'append_suffix' },
                { label: '覆盖已有文件', value: 'overwrite_existing' },
              ]"
            />
            <p class="settings-note">{{ settingsDuplicateDescription }}</p>
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
                { label: '自定义归档', value: 'custom' },
              ]"
            />
            <div v-if="settings.draft.archive_mode === 'custom'" class="archive-option-grid">
              <UiCheckbox
                v-model="settingsArchiveCover"
                label="保存封面"
                :disabled="settings.loading || settings.saving"
              />
              <UiCheckbox
                v-model="settingsArchiveSubtitles"
                label="保存字幕"
                :disabled="settings.loading || settings.saving"
              />
              <UiCheckbox
                v-model="settingsArchiveDanmaku"
                label="保存弹幕"
                :disabled="settings.loading || settings.saving"
              />
              <UiCheckbox
                v-model="settingsArchiveNfo"
                label="生成 NFO"
                :disabled="settings.loading || settings.saving"
              />
            </div>
            <UiCheckbox
              v-model="settingsRetainRawStreams"
              label="保留原始视频/音频轨道"
              :disabled="settings.loading || settings.saving"
            />
            <div class="archive-option-grid">
              <UiCheckbox
                v-model="settingsEmbedCover"
                label="支持时嵌入封面"
                :disabled="settings.loading || settings.saving"
              />
              <UiCheckbox
                v-model="settingsEmbedSubtitles"
                label="支持时嵌入字幕"
                :disabled="settings.loading || settings.saving"
              />
            </div>
            <p class="settings-note">{{ settingsArchiveDescription }}</p>
          </section>

          <section class="settings-block">
            <div class="settings-block-heading">
              <h3>高级</h3>
              <span>网络代理、日志、数据目录和维护工具</span>
            </div>
            <UiTextField v-model="settingsProxyUrl" label="代理地址" placeholder="例如 http://127.0.0.1:7890，留空为直连" />
            <UiSelect
              v-model="settingsLogLevel"
              label="任务日志级别"
              :options="[
                { label: '调试', value: 'debug' },
                { label: '信息', value: 'info' },
                { label: '警告', value: 'warning' },
                { label: '错误', value: 'error' },
              ]"
            />
            <div class="directory-row">
              <UiTextField v-model="settingsDataDir" label="数据目录" placeholder="留空时使用当前工作目录下的 .bdl" />
              <UiButton variant="secondary" :disabled="settings.loading || settings.saving" @click="settings.chooseDataDir">
                选择
              </UiButton>
            </div>
            <div v-if="settings.draft.data_dir" class="settings-actions">
              <UiButton variant="ghost" :disabled="settings.loading || settings.saving" @click="settings.clearDataDir">
                使用默认数据目录
              </UiButton>
            </div>
            <p class="settings-note">数据目录影响任务库、账户摘要和维护文件，修改后下次启动生效。</p>
            <div class="settings-actions">
              <UiButton variant="secondary" :disabled="settings.loading || settings.saving" @click="settings.cleanupCache">
                清理缓存
              </UiButton>
              <UiButton variant="secondary" :disabled="settings.loading || settings.saving" @click="settings.cleanupTemp">
                清理临时文件
              </UiButton>
              <UiButton variant="secondary" :disabled="settings.loading || settings.saving" @click="settings.exportDiagnostics">
                导出诊断
              </UiButton>
            </div>
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
</template>
