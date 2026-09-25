export type SourceKind =
  | 'video'
  | 'bangumi'
  | 'cheese'
  | 'favorite'
  | 'collection'
  | 'series'
  | 'uploader'
  | 'unknown'

export interface SourceSummary {
  id: string
  kind: SourceKind
  input: string
  title: string
  loaded_count: number
  total_count: number | null
  has_more: boolean
}

export interface NormalizedSourceTree {
  source: SourceSummary
  groups: NormalizedGroup[]
}

export interface NormalizedGroup {
  id: string
  kind: string
  title: string
  items: NormalizedItem[]
  page: PageState | null
}

export interface PageState {
  page_number: number
  page_size: number
  loaded_count: number
  total_count: number | null
  has_more: boolean
}

export interface NormalizedItem {
  id: string
  title: string
  owner_name: string | null
  owner_mid?: number | null
  publish_date?: string | null
  cover_url: string | null
  duration_seconds: number | null
  parts: NormalizedPart[]
}

export interface NormalizedPart {
  id: string
  title: string
  aid: number | null
  bvid: string | null
  cid: number | null
  duration_seconds: number | null
  streams: MediaStream[]
  assets: DerivedAsset[]
}

export interface MediaStream {
  id: string
  kind: MediaKind
  quality: StreamQuality
  codec: StreamCodec
  bandwidth: number | null
  urls: string[]
  headers: HeaderPair[]
  acquired_at: string
}

export interface HeaderPair {
  name: string
  value: string
}

export type MediaKind = 'video' | 'audio'

export type StreamQuality = { kind: 'best' } | { kind: 'quality'; value: number }

export type StreamCodec = 'auto' | 'avc' | 'hevc' | 'av1' | 'unknown'

export type AssetKind = 'cover' | 'subtitle' | 'danmaku' | 'nfo'

export type FetchPolicy = 'never' | 'on_demand' | 'always'

export interface DerivedAsset {
  kind: AssetKind
  format: string | null
  fetch_policy: FetchPolicy
  urls: string[]
  headers: HeaderPair[]
}

export type TaskStatus =
  | 'waiting'
  | 'parsing'
  | 'downloading'
  | 'muxing'
  | 'completed'
  | 'failed'
  | 'paused'
  | 'cancelled'

export type ResourceStatus = 'pending' | 'downloading' | 'completed' | 'failed' | 'paused' | 'cancelled'

export type DownloadResourceKind = 'video' | 'audio' | 'asset'

export type DownloadResourceIntent = 'video' | 'audio' | 'cover' | 'subtitle' | 'danmaku' | 'nfo'

export interface DocumentTreeDirectory {
  tree_uri: string
  display_name: string
}

export type DownloadExportTarget = {
  kind: 'document_tree'
  tree_uri: string
  relative_path: string
  duplicate_naming_strategy: DuplicateNamingStrategy
  document_uri: string | null
}

export interface DownloadTask {
  id: string
  title: string
  source_id: string
  status: TaskStatus
  resources: DownloadResource[]
  output_path: string
  export_target?: DownloadExportTarget | null
  refresh_intent: DownloadTaskRefreshIntent | null
  media_selection: DownloadTaskMediaSelection
  scheduled_at: string | null
  speed_limit_bytes_per_second: number | null
}

export type DuplicateTaskPolicy = 'skip' | 'create' | 'ask'

export interface DuplicateTaskMatch {
  proposed_task_id: string
  title: string
  existing_task_id: string
  existing_status: TaskStatus
}

export interface SelectionCreateTasksResult {
  created: DownloadTask[]
  duplicates: DuplicateTaskMatch[]
  skipped_existing: number
  requires_confirmation: boolean
}

export interface SelectionSizeEstimate {
  estimated_bytes: number
  estimated_parts: number
  unknown_streams: number
}

export interface DownloadTaskRefreshIntent {
  input: DownloadTaskRefreshInput
  cid: number
  page_number?: number | null
  cover_url?: string | null
  duration_seconds?: number | null
}

export type DownloadTaskRefreshInput =
  | { kind: 'video_bvid'; bvid: string }
  | { kind: 'video_aid'; aid: number }
  | { kind: 'bangumi_episode'; ep_id: number }
  | { kind: 'cheese_episode'; ep_id: number }

export interface DownloadTaskMediaSelection {
  video_quality: string
  audio_quality: string
  video_codec: string
  container: string
  processing?: {
    retain_raw_streams: boolean
    embed_cover: boolean
    embed_subtitles: boolean
  } | null
}

export interface DownloadResource {
  id: string
  kind: DownloadResourceKind
  intent: DownloadResourceIntent
  current_urls: string[]
  headers: HeaderPair[]
  target_path: string
  temp_path: string
  status: ResourceStatus
}

export interface QueueRemoveResponse {
  removed: boolean
}

export interface BulkQueueFailure {
  task_id: string
  message: string
}

export interface BulkQueueResult {
  updated: DownloadTask[]
  removed: string[]
  failed: BulkQueueFailure[]
}

export interface QueueLogEntry {
  task_id: string
  level: 'info' | 'warning' | 'error'
  message: string
  created_at: string
}

export interface QueueProgressEntry {
  task_id: string
  resource_id: string
  downloaded_bytes: number
  total_bytes: number | null
  created_at: string
}

export interface StartupRecoverySnapshot {
  task_ids: string[]
  auto_recovery_enabled: boolean
}

export type DuplicateNamingStrategy = 'skip_existing' | 'overwrite_existing' | 'append_suffix'
export type VideoCodecPreference = 'auto' | 'avc' | 'hevc' | 'av1'
export type MissingQualityPolicy = 'lower' | 'skip' | 'ask'
export type LogLevel = 'debug' | 'info' | 'warning' | 'error'
export type ArchiveMode = 'fast' | 'complete_archive' | 'custom'
export type DownloadMediaMode = 'audio_video' | 'video_only' | 'audio_only'

export interface ArchiveAssetSelection {
  cover: boolean
  subtitles: boolean
  danmaku: boolean
  nfo: boolean
}

export interface VideoPreference {
  quality: string
  codec: VideoCodecPreference
}

export interface MediaPreferences {
  video: VideoPreference[]
  audio: string[]
  fallback: 'error' | 'best'
}

export interface SettingsSnapshot {
  settings_schema_version: number
  usage_notice_acknowledged: boolean
  auto_check_updates: boolean
  theme_preference: 'system' | 'light' | 'dark'
  parse_rules: ParseRules
  download_dir: string | null
  document_tree_output: DocumentTreeDirectory | null
  naming_template: string
  naming_presets: NamingPreset[]
  quality: string
  archive_mode: ArchiveMode
  archive_assets: ArchiveAssetSelection
  output_extension: 'mp4' | 'mkv'
  duplicate_naming_strategy: DuplicateNamingStrategy
  audio_quality: string
  codec: VideoCodecPreference
  media_preferences: MediaPreferences
  missing_quality_policy: MissingQualityPolicy
  ffmpeg_path: string | null
  retain_raw_streams: boolean
  embed_cover: boolean
  embed_subtitles: boolean
  proxy_url: string | null
  log_level: LogLevel
  data_dir: string | null
  concurrent_tasks: number
  retry_count: number
  segment_count: number
  global_speed_limit_bytes_per_second: number | null
  startup_auto_recovery: boolean
  auto_refresh_expired_urls: boolean
}

export interface NamingPreset {
  id: string
  name: string
  template: string
}

export interface MaintenanceResult {
  removed_files: number
  path: string
}

export type DownloadDirectoryStatus = 'ready' | 'missing' | 'not_directory' | 'unwritable'
export type FfmpegStatus = 'ready' | 'missing' | 'invalid'
export type FfmpegSource = 'configured' | 'system' | 'native'

export interface DownloadDirectoryHealth {
  status: DownloadDirectoryStatus
  path: string
  message: string
}

export interface FfmpegHealth {
  status: FfmpegStatus
  source: FfmpegSource
  path: string | null
  version: string | null
  message: string
}

export interface EnvironmentHealthSnapshot {
  ready: boolean
  download_directory: DownloadDirectoryHealth
  ffmpeg: FfmpegHealth
}

export interface DiagnosticsExportResponse {
  path: string
}

export interface AccountSummary {
  logged_in: boolean
  name: string | null
  avatar_url: string | null
  mid: string | null
  vip_label: string | null
}

export type AccountLibraryFolderKind = 'created_favorite' | 'collected_favorite'

export interface AccountLibraryFolder {
  kind: AccountLibraryFolderKind
  media_id: string
  title: string
  description: string | null
  cover_url: string | null
  owner_name: string | null
  owner_mid: string | null
  media_count: number
  source_url: string
}

export interface AccountLibraryPage {
  items: AccountLibraryFolder[]
  total: number
  page: number
  page_size: number
  has_more: boolean
}

export type QrLoginStatus = 'waiting' | 'scanned' | 'confirmed' | 'expired' | 'unknown'

export interface QrLoginSession {
  qr_url: string
  qrcode_key: string
  qr_image_svg: string
  expires_in_seconds: number
}

export interface QrLoginPollResponse {
  status: QrLoginStatus
  message: string
  account: AccountSummary | null
}


export interface ParseRules {
  interval_seconds: number
  rest_seconds: number
  pages_per_round: number
}
