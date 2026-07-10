<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'

import {
  defaultNamingTemplate,
  namingTemplatePresets,
  namingVariables,
  useSettingsStore,
} from '../stores/settings'
import UiButton from '../ui/Button.vue'
import UiCheckbox from '../ui/Checkbox.vue'
import UiInlineNotice from '../ui/InlineNotice.vue'
import UiSelect from '../ui/Select.vue'
import UiTextField from '../ui/TextField.vue'
import UiEnvironmentHealthPanel from '../ui/EnvironmentHealthPanel.vue'
import { speedLimitMibError, toBytesPerSecond, toMibPerSecondInput } from '../utils/speedLimit'

const settings = useSettingsStore()

const settingsSections = [
  { id: 'settings-download', label: '下载', icon: 'i-tabler-download' },
  { id: 'settings-media', label: '默认媒体', icon: 'i-tabler-movie' },
  { id: 'settings-naming', label: '命名', icon: 'i-tabler-file-text' },
  { id: 'settings-media-advanced', label: '媒体高级', icon: 'i-tabler-adjustments-horizontal' },
  { id: 'settings-archive', label: '归档和素材', icon: 'i-tabler-archive' },
  { id: 'settings-maintenance', label: '网络和维护', icon: 'i-tabler-tool' },
]

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
const settingsGlobalSpeedLimitMib = ref('')
const settingsGlobalSpeedLimitError = computed(() => speedLimitMibError(settingsGlobalSpeedLimitMib.value))
const settingsGlobalSpeedInputDirty = computed(
  () => settingsGlobalSpeedLimitMib.value.trim() !== toMibPerSecondInput(
    settings.draft.global_speed_limit_bytes_per_second,
  ),
)
const settingsFormChanged = computed(() => settings.changed || settingsGlobalSpeedInputDirty.value)
const updateGlobalSpeedLimit = (value: string) => {
  settingsGlobalSpeedLimitMib.value = value
  if (!settingsGlobalSpeedLimitError.value) {
    settings.setGlobalSpeedLimitBytesPerSecond(toBytesPerSecond(value) ?? null)
  }
}
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

const resetNamingTemplate = () => {
  settings.setNamingTemplate(defaultNamingTemplate)
}

const resetSettingsDraft = () => {
  settings.resetDraft()
  settingsGlobalSpeedLimitMib.value = toMibPerSecondInput(
    settings.draft.global_speed_limit_bytes_per_second,
  )
}

const saveSettings = async () => {
  if (settingsGlobalSpeedLimitError.value) return
  await settings.save()
  settingsGlobalSpeedLimitMib.value = toMibPerSecondInput(
    settings.draft.global_speed_limit_bytes_per_second,
  )
}

const formatNamingVariable = (name: string): string => `{${name}}`

const scrollToSettingsSection = (id: string) => {
  document.getElementById(id)?.scrollIntoView({ behavior: 'smooth', block: 'start' })
}

onMounted(async () => {
  await settings.ensureLoaded()
  settingsGlobalSpeedLimitMib.value = toMibPerSecondInput(
    settings.draft.global_speed_limit_bytes_per_second,
  )
  await settings.checkEnvironment()
})
</script>

<template>
  <section class="page-grid settings-grid">
    <section class="panel settings-panel">
      <div class="panel-heading settings-heading">
        <div class="settings-heading-copy">
          <span>CONFIGURATION</span>
          <p>媒体与任务默认值用于新任务；全局下载限速会即时作用于当前传输。</p>
        </div>
        <div class="settings-actions">
          <span v-if="settingsFormChanged" class="dirty-indicator">未保存更改</span>
          <UiButton variant="ghost" :disabled="settings.loading || settings.saving || !settingsFormChanged" @click="resetSettingsDraft">
            撤销
          </UiButton>
          <UiButton :disabled="settings.loading || settings.saving || !settingsFormChanged || Boolean(settings.namingTemplateError) || Boolean(settingsGlobalSpeedLimitError)" @click="saveSettings">
            {{ settings.saving ? '保存中' : '保存' }}
          </UiButton>
        </div>
      </div>

      <UiInlineNotice v-if="settings.notice" :tone="settings.notice.tone">
        {{ settings.notice.message }}
      </UiInlineNotice>

      <div class="settings-workspace">
        <nav class="settings-section-nav" aria-label="设置分区">
          <button
            v-for="section in settingsSections"
            :key="section.id"
            type="button"
            @click="scrollToSettingsSection(section.id)"
          >
            <UIcon :name="section.icon" aria-hidden="true" />
            <span>{{ section.label }}</span>
          </button>
        </nav>

        <div class="settings-content">
      <UiEnvironmentHealthPanel
        :health="settings.environmentHealth"
        :checking="settings.environmentChecking"
        @check="settings.checkEnvironment"
        @create-directory="settings.createDownloadDirectory"
        @choose-directory="settings.chooseDownloadDir"
        @choose-ffmpeg="settings.chooseFfmpegPath"
        @use-system-ffmpeg="settings.clearFfmpegPath"
      />
      <section id="settings-download" class="settings-block">
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
        <UiTextField
          :model-value="settingsGlobalSpeedLimitMib"
          label="全局下载限速（MiB/s）"
          placeholder="留空时不限速"
          :error="settingsGlobalSpeedLimitError ?? undefined"
          helper="所有并发任务共享此带宽额度"
          @update:model-value="updateGlobalSpeedLimit"
        />
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

      <section id="settings-media" class="settings-block">
        <div class="settings-block-heading">
          <h3>默认媒体</h3>
          <span>新建任务默认使用的质量和封装</span>
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
        <UiSelect
          v-model="settingsOutputFormat"
          label="封装格式"
          :options="[
            { label: 'MP4', value: 'mp4' },
            { label: 'MKV', value: 'mkv' },
          ]"
        />
      </section>

      <section id="settings-naming" class="settings-block">
        <div class="settings-block-heading">
          <h3>命名</h3>
          <span>模板保存后用于新建任务</span>
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

      <details id="settings-media-advanced" class="settings-disclosure">
        <summary>
          <div>
            <strong>媒体高级</strong>
            <span>编码偏好、目标质量策略、FFmpeg 和分段</span>
          </div>
        </summary>
        <section class="settings-block disclosure-block">
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
            v-model="settingsSegmentCount"
            label="单任务分段数"
            :options="[
              { label: '1 段', value: '1' },
              { label: '2 段', value: '2' },
              { label: '4 段', value: '4' },
              { label: '8 段', value: '8' },
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
      </details>

      <details id="settings-archive" class="settings-disclosure">
        <summary>
          <div>
            <strong>归档和素材</strong>
            <span>封面、字幕、弹幕、NFO 和原始轨道</span>
          </div>
        </summary>
        <section class="settings-block disclosure-block">
          <UiSelect
            v-model="settingsArchiveMode"
            label="保存内容"
            :options="[
              { label: '只保存视频（最快）', value: 'fast' },
              { label: '视频 + 全部可用素材（归档）', value: 'complete_archive' },
              { label: '自定义素材', value: 'custom' },
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
          <div class="archive-option-grid archive-single-grid">
            <UiCheckbox
              v-model="settingsRetainRawStreams"
              label="保留原始视频/音频轨道"
              :disabled="settings.loading || settings.saving"
            />
          </div>
          <div class="archive-option-grid embed-option-grid">
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
      </details>

      <details id="settings-maintenance" class="settings-disclosure">
        <summary>
          <div>
            <strong>网络和维护</strong>
            <span>代理、日志、数据目录、缓存和诊断导出</span>
          </div>
        </summary>
        <section class="settings-block disclosure-block">
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
      </details>

      <p v-if="settings.error" class="settings-error">{{ settings.error }}</p>
        </div>
      </div>
    </section>
  </section>
</template>
