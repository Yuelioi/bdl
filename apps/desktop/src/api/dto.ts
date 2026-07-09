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

export interface DownloadTask {
  id: string
  title: string
  source_id: string
  status: TaskStatus
  resources: DownloadResource[]
  output_path: string
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

export interface QueueLogEntry {
  task_id: string
  level: 'info' | 'warning' | 'error'
  message: string
  created_at: string
}

export interface AccountSummary {
  logged_in: boolean
  name: string | null
  avatar_url: string | null
  mid: string | null
  vip_label: string | null
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
