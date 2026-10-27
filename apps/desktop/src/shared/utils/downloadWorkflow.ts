import { createNfoOptions, normalizeNfoOptions, nfoOptionsError } from './nfo'
import type { DownloadPreset, DownloadWorkflow, SettingsSnapshot, WorkflowAssetOutput } from '../api/dto'

export const cloneWorkflow = (value: DownloadWorkflow): DownloadWorkflow => {
  const result = JSON.parse(JSON.stringify(value)) as DownloadWorkflow
  result.nfo_options = normalizeNfoOptions(result.nfo_options)
  return result
}
export const cloneDownloadPresets = (presets: DownloadPreset[]): DownloadPreset[] => presets.map((preset) => ({ ...preset, workflow: cloneWorkflow(preset.workflow) }))
export const createWorkflow = (): DownloadWorkflow => ({
  video: { enabled: true, save: false, format: 'm4s', retain_original: false },
  audio: { enabled: true, save: false, format: 'm4s', retain_original: false },
  cover: { enabled: false, save: true, embed: false, format: 'original', retain_original: false },
  subtitles: { enabled: false, save: true, embed: false, format: 'srt', retain_original: false },
  danmaku: { enabled: false, save: true, embed: false, format: 'xml', retain_original: false },
  nfo: false, nfo_options: createNfoOptions(), merge_media: true, container: 'mp4', quality: 'best', audio_quality: 'best', codec: 'auto',
  media_preferences: { video: [], audio: [] }, missing_quality_policy: 'lower',
})

export const legacyBuiltinDownloadPresets = (): DownloadPreset[] => {
  const base = createWorkflow()
  const assets = createWorkflow()
  assets.video.enabled = false
  assets.audio.enabled = false
  assets.merge_media = false
  const audio = cloneWorkflow(assets)
  audio.audio = { enabled: true, save: true, format: 'mp3', retain_original: false }
  const subtitles = cloneWorkflow(assets)
  subtitles.subtitles.enabled = true
  const danmaku = cloneWorkflow(assets)
  danmaku.danmaku.enabled = true
  danmaku.danmaku.format = 'html'
  const cover = cloneWorkflow(assets)
  cover.cover.enabled = true
  const archive = cloneWorkflow(base)
  archive.container = 'mkv'
  archive.cover.enabled = archive.cover.embed = true
  archive.subtitles.enabled = archive.subtitles.embed = true
  archive.danmaku.enabled = archive.nfo = true
  return [
    { id: 'video', name: '音频+视频', workflow: base },
    { id: 'audio', name: '仅音频（MP3）', workflow: audio },
    { id: 'subtitles', name: '仅字幕（SRT）', workflow: subtitles },
    { id: 'danmaku', name: '仅弹幕（HTML）', workflow: danmaku },
    { id: 'cover', name: '仅封面', workflow: cover },
    { id: 'archive', name: '完整归档（MKV）', workflow: archive },
  ]
}

export const builtinDownloadPresets = (): DownloadPreset[] => {
  const base = createWorkflow()
  const all = cloneWorkflow(base)
  all.cover.enabled = all.subtitles.enabled = all.danmaku.enabled = all.nfo = true
  const mkv = cloneWorkflow(base)
  mkv.container = 'mkv'
  mkv.subtitles.enabled = mkv.subtitles.embed = true
  mkv.subtitles.save = false
  return [{ id: 'video', name: '快速下载', workflow: base }, { id: 'all', name: '下载全部资源', workflow: all }, { id: 'mkv', name: '封装 MKV（视频+字幕）', workflow: mkv }]
}

export const legacyWorkflow = (settings: SettingsSnapshot): DownloadWorkflow => {
  const flow = createWorkflow()
  flow.video.enabled = settings.media_mode !== 'audio_only'
  flow.audio.enabled = settings.media_mode !== 'video_only'
  flow.audio.save = !flow.video.enabled
  flow.video.retain_original = flow.audio.retain_original = settings.retain_raw_streams
  flow.audio.format = settings.audio_output_format
  flow.merge_media = flow.video.enabled
  flow.container = settings.output_extension
  const selected = settings.archive_mode === 'complete_archive'
    ? { cover: true, subtitles: true, danmaku: true, nfo: true }
    : settings.archive_mode === 'custom' ? settings.archive_assets : { cover: false, subtitles: false, danmaku: false, nfo: false }
  const asset = (save: boolean, embed: boolean, format: WorkflowAssetOutput['format']): WorkflowAssetOutput => ({ enabled: save || embed, save: !(save || embed) || save, embed, format, retain_original: false })
  flow.cover = asset(selected.cover, flow.video.enabled && settings.embed_cover, 'original')
  flow.subtitles = asset(selected.subtitles, flow.video.enabled && settings.embed_subtitles, settings.subtitle_format)
  flow.danmaku = asset(selected.danmaku, false, settings.danmaku_format)
  flow.nfo = selected.nfo
  flow.quality = settings.quality
  flow.audio_quality = settings.audio_quality
  flow.codec = settings.codec
  flow.media_preferences = JSON.parse(JSON.stringify(settings.media_preferences)) as DownloadWorkflow['media_preferences']
  flow.missing_quality_policy = settings.missing_quality_policy
  return flow
}

export const migrateDownloadPresets = (settings: SettingsSnapshot): Pick<SettingsSnapshot, 'download_presets' | 'selected_download_preset'> => {
  if (settings.download_presets?.length) return { download_presets: cloneDownloadPresets(settings.download_presets).map((p) => p.id === 'migrated' && p.name === '原下载配置' ? { ...p, name: '历史配置' } : p), selected_download_preset: settings.selected_download_preset || settings.download_presets[0].id }
  const presets = builtinDownloadPresets()
  const legacy = legacyWorkflow(settings)
  const matching = presets.find((preset) => JSON.stringify({ ...preset.workflow, quality: legacy.quality, audio_quality: legacy.audio_quality, codec: legacy.codec, media_preferences: legacy.media_preferences, missing_quality_policy: legacy.missing_quality_policy }) === JSON.stringify(legacy))
  if (!matching) presets.push({ id: 'migrated', name: '历史配置', workflow: legacy })
  return { download_presets: presets, selected_download_preset: matching?.id ?? 'migrated' }
}

export const workflowError = (flow: DownloadWorkflow): string | null => {
  if (flow.nfo) { const error = nfoOptionsError(normalizeNfoOptions(flow.nfo_options)); if (error) return error }
  const merge = flow.merge_media && flow.video.enabled
  if (![flow.video.enabled, flow.audio.enabled, flow.cover.enabled, flow.subtitles.enabled, flow.danmaku.enabled, flow.nfo].some(Boolean)) return '至少选择一种内容'
  for (const media of [flow.video, flow.audio]) if (media.enabled && !media.save && !media.retain_original && !merge) return '未生成媒体成品时，请保存独立音视频文件'
  for (const asset of [flow.cover, flow.subtitles, flow.danmaku]) {
    if (asset.enabled && !asset.save && !asset.embed && !asset.retain_original) return '所选内容需要独立保存、嵌入或保留原始文件'
    if (asset.enabled && asset.embed && (!merge || flow.container !== 'mkv')) return '嵌入需要生成 MKV 视频成品'
  }
  if (flow.subtitles.enabled && flow.subtitles.embed && flow.subtitles.format === 'original') return '嵌入字幕请选择 SRT 或 ASS'
  return null
}

export const downloadPresetsError = (presets: DownloadPreset[], selected: string): string | null => {
  if (!presets.length || presets.length > 32) return '下载预设需保留 1～32 个'
  const names = new Set<string>()
  const ids = new Set<string>()
  for (const preset of presets) {
    if (!preset.name.trim() || preset.name.length > 40 || names.has(preset.name.trim()) || !preset.id || ids.has(preset.id)) return '预设名称不能为空或重复，最多 40 个字符'
    names.add(preset.name.trim())
    ids.add(preset.id)
    const error = workflowError(preset.workflow)
    if (error) return `${preset.name}：${error}`
  }
  return ids.has(selected) ? null : '请选择默认下载预设'
}

export const workflowOutputs = (flow: DownloadWorkflow): string[] => {
  const outputs: string[] = []
  if (flow.merge_media && flow.video.enabled) {
    const embedded = [flow.cover.enabled && flow.cover.embed ? '封面' : '', flow.subtitles.enabled && flow.subtitles.embed ? '字幕' : ''].filter(Boolean)
    outputs.push(`${flow.container.toUpperCase()} ${flow.audio.enabled ? '音频+视频' : '视频'}${embedded.length ? `（内含${embedded.join('、')}）` : ''}`)
  }
  if (flow.video.enabled && flow.video.save) outputs.push('独立视频 m4s')
  if (flow.audio.enabled && flow.audio.save) outputs.push(`独立音频 ${flow.audio.format.toUpperCase()}`)
  for (const [rule, name] of [[flow.cover, '封面'], [flow.subtitles, '字幕'], [flow.danmaku, '弹幕']] as const) {
    if (rule.enabled && rule.save) outputs.push(`${name}${rule.format === 'original' ? '原文件' : ` ${rule.format.toUpperCase()}`}`)
    if (rule.enabled && rule.retain_original && ((!['original', 'xml'].includes(rule.format)) || !rule.save)) outputs.push(`原始${name}`)
  }
  if (flow.video.enabled && flow.video.retain_original && !flow.video.save) outputs.push('原始视频 m4s')
  if (flow.audio.enabled && flow.audio.retain_original && (!flow.audio.save || flow.audio.format !== 'm4s')) outputs.push('原始音频 m4s')
  if (flow.nfo) outputs.push('NFO 信息文件')
  return outputs
}

export const workflowExtension = (flow: DownloadWorkflow): string => {
  if (flow.merge_media && flow.video.enabled) return flow.container
  if (flow.video.enabled) return 'm4s'
  if (flow.audio.enabled) return flow.audio.save ? flow.audio.format : 'm4s'
  if (flow.cover.enabled) return 'jpg'
  if (flow.subtitles.enabled) return !flow.subtitles.save || flow.subtitles.format === 'original' ? 'json' : flow.subtitles.format
  if (flow.danmaku.enabled) return flow.danmaku.save ? flow.danmaku.format : 'xml'
  return 'nfo'
}
