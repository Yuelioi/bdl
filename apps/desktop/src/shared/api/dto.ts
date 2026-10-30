export type SourceKind =
  'video' | 'bangumi' | 'cheese' | 'favorite' | 'watch_later' | 'collection' | 'series' | 'uploader' | 'unknown';

export interface SourceSummary {
  id: string;
  kind: SourceKind;
  input: string;
  title: string;
  loaded_count: number;
  total_count: number | null;
  has_more: boolean;
}

export interface NormalizedSourceTree {
  source: SourceSummary;
  groups: NormalizedGroup[];
}

export interface NormalizedGroup {
  id: string;
  kind: string;
  title: string;
  items: NormalizedItem[];
  page: PageState | null;
}

export interface PageState {
  page_number: number;
  page_size: number;
  loaded_count: number;
  total_count: number | null;
  has_more: boolean;
}

export interface NormalizedItem {
  id: string;
  title: string;
  owner_name: string | null;
  owner_mid?: number | null;
  publish_date?: string | null;
  cover_url: string | null;
  duration_seconds: number | null;
  parts: NormalizedPart[];
}

export interface NormalizedPart {
  id: string;
  title: string;
  aid: number | null;
  bvid: string | null;
  cid: number | null;
  duration_seconds: number | null;
  streams: MediaStream[];
  assets: DerivedAsset[];
}

export interface MediaStream {
  id: string;
  kind: MediaKind;
  quality: StreamQuality;
  codec: StreamCodec;
  bandwidth: number | null;
  urls: string[];
  headers: HeaderPair[];
  acquired_at: string;
}

export interface HeaderPair {
  name: string;
  value: string;
}

export type MediaKind = 'video' | 'audio';

export type StreamQuality = { kind: 'best' } | { kind: 'quality'; value: number };

export type StreamCodec = 'auto' | 'avc' | 'hevc' | 'av1' | 'aac' | 'flac' | 'eac3' | 'unknown';

export type AssetKind = 'cover' | 'subtitle' | 'danmaku' | 'nfo';

export type FetchPolicy = 'never' | 'on_demand' | 'always';

export interface DerivedAsset {
  kind: AssetKind;
  format: string | null;
  fetch_policy: FetchPolicy;
  urls: string[];
  headers: HeaderPair[];
}

export type TaskStatus =
  'waiting' | 'parsing' | 'downloading' | 'muxing' | 'completed' | 'failed' | 'paused' | 'cancelled';

export type ResourceStatus = 'pending' | 'downloading' | 'completed' | 'failed' | 'paused' | 'cancelled';

export type DownloadResourceKind = 'video' | 'audio' | 'asset';

export type DownloadResourceIntent = 'video' | 'audio' | 'cover' | 'subtitle' | 'danmaku' | 'nfo';

export interface DocumentTreeDirectory {
  tree_uri: string;
  display_name: string;
}

export type DownloadExportTarget = {
  kind: 'document_tree';
  tree_uri: string;
  relative_path: string;
  duplicate_naming_strategy: DuplicateNamingStrategy;
  document_uri: string | null;
};

export interface DownloadTask {
  id: string;
  title: string;
  source_id: string;
  status: TaskStatus;
  completed_at?: string | null;
  resources: DownloadResource[];
  output_path: string;
  export_target?: DownloadExportTarget | null;
  refresh_intent: DownloadTaskRefreshIntent | null;
  media_selection: DownloadTaskMediaSelection;
  scheduled_at: string | null;
  speed_limit_bytes_per_second: number | null;
}

export type DuplicateTaskPolicy = 'skip' | 'create' | 'ask';

export interface DuplicateTaskMatch {
  proposed_task_id: string;
  title: string;
  existing_task_id: string;
  existing_status: TaskStatus;
}

export interface SelectionCreateTasksResult {
  created: DownloadTask[];
  duplicates: DuplicateTaskMatch[];
  skipped_existing: number;
  requires_confirmation: boolean;
}

export type PreparationStatus = 'queued' | 'preparing' | 'awaiting_confirmation' | 'completed' | 'failed' | 'cancelled' | 'interrupted';
export interface PreparationJob {
  id: string;
  title: string;
  source_id: string;
  input: string;
  status: PreparationStatus;
  origin: string;
  processed: number;
  total: number;
  created: number;
  skipped_existing: number;
  duplicates: DuplicateTaskMatch[];
  error: { code: string; message: string; detail?: string } | null;
  created_at: string;
}

export interface SelectionSizeEstimate {
  estimated_bytes: number;
  estimated_parts: number;
  unknown_streams: number;
}

export interface DownloadTaskRefreshIntent {
  input: DownloadTaskRefreshInput;
  cid: number;
  page_number?: number | null;
  cover_url?: string | null;
  duration_seconds?: number | null;
}

export type DownloadTaskRefreshInput =
  | { kind: 'video_bvid'; bvid: string }
  | { kind: 'video_aid'; aid: number }
  | { kind: 'bangumi_episode'; ep_id: number }
  | { kind: 'cheese_episode'; ep_id: number };

export interface DownloadTaskMediaSelection {
  owned_document_uris?: string[];
  preserved_document_uris?: string[];
  workflow?: DownloadWorkflow | null;
  artifacts?: DownloadArtifact[];
  video_quality: string;
  audio_quality: string;
  video_codec: string;
  audio_codec?: string | null;
  container: string;
  processing?: {
    retain_raw_streams: boolean;
    embed_cover: boolean;
    embed_subtitles: boolean;
    archive_assets?: SettingsSnapshot['archive_assets'] | null;
    subtitle_format?: SettingsSnapshot['subtitle_format'] | null;
    danmaku_format?: SettingsSnapshot['danmaku_format'] | null;
  } | null;
}

export interface DownloadResource {
  id: string;
  kind: DownloadResourceKind;
  intent: DownloadResourceIntent;
  current_urls: string[];
  headers: HeaderPair[];
  target_path: string;
  temp_path: string;
  status: ResourceStatus;
}

export interface QueueRemoveResponse {
  removed: boolean;
}

export interface BulkQueueFailure {
  task_id: string;
  code: string;
  message: string;
  detail: string | null;
}

export interface BulkQueueResult {
  updated: DownloadTask[];
  removed: string[];
  failed: BulkQueueFailure[];
}

export interface QueueLogEntry {
  task_id: string;
  level: 'info' | 'warning' | 'error';
  message: string;
  created_at: string;
}

export interface DiagnosticEnvironment {
  app_version: string;
  os: string;
  arch: string;
  device: { os_version: string | null; android_sdk: number | null; webview_version: string | null };
  device_info_available: boolean;
  device_info_error?: string | null;
  segment_count: number;
  concurrent_tasks: number;
  retry_count: number;
  proxy_mode: ProxyMode;
}

export interface QueueProgressEntry {
  task_id: string;
  resource_id: string;
  downloaded_bytes: number;
  total_bytes: number | null;
  created_at: string;
}

export interface StartupRecoverySnapshot {
  task_ids: string[];
  auto_recovery_enabled: boolean;
}

export type DuplicateNamingStrategy = 'skip_existing' | 'overwrite_existing' | 'append_suffix';
export type VideoCodecPreference = 'auto' | 'avc' | 'hevc' | 'av1';
export type MissingQualityPolicy = 'lower' | 'skip' | 'ask';
export type LogLevel = 'debug' | 'info' | 'warning' | 'error';
export type ArchiveMode = 'fast' | 'complete_archive' | 'custom';
export type DownloadMediaMode = 'audio_video' | 'video_only' | 'audio_only';

export interface ArchiveAssetSelection {
  cover: boolean;
  subtitles: boolean;
  danmaku: boolean;
  nfo: boolean;
}

export interface VideoPreference {
  quality: string;
  codec: VideoCodecPreference;
}

export interface MediaPreferences {
  video: VideoPreference[];
  audio: string[];
  video_codecs?: Exclude<VideoCodecPreference, 'auto'>[];
  audio_codecs?: AudioCodecPreference[];
}

export type AudioCodecPreference = 'aac' | 'flac' | 'eac3';

export type ProxyMode = 'system' | 'direct' | 'custom';

export interface SettingsSnapshot {
  download_presets: DownloadPreset[];
  selected_download_preset: string;
  settings_schema_version: number;
  clipboard_monitor_enabled: boolean;
  usage_notice_acknowledged: boolean;
  auto_check_updates: boolean;
  theme_preference: 'system' | 'light' | 'dark';
  parse_rules: ParseRules;
  download_dir: string | null;
  document_tree_output: DocumentTreeDirectory | null;
  naming_template: string;
  naming_presets: NamingPreset[];
  quality: string;
  archive_mode: ArchiveMode;
  archive_assets: ArchiveAssetSelection;
  output_extension: 'mp4' | 'mkv';
  media_mode: DownloadMediaMode;
  audio_output_format: 'm4s' | 'mp3';
  subtitle_format: 'srt' | 'ass';
  danmaku_format: 'xml' | 'html' | 'srt' | 'ass';
  duplicate_naming_strategy: DuplicateNamingStrategy;
  audio_quality: string;
  codec: VideoCodecPreference;
  media_preferences: MediaPreferences;
  missing_quality_policy: MissingQualityPolicy;
  ffmpeg_path: string | null;
  retain_raw_streams: boolean;
  embed_cover: boolean;
  embed_subtitles: boolean;
  proxy_mode: ProxyMode;
  proxy_url: string | null;
  log_level: LogLevel;
  data_dir: string | null;
  concurrent_tasks: number;
  retry_count: number;
  segment_count: number;
  global_speed_limit_bytes_per_second: number | null;
  startup_auto_recovery: boolean;
  auto_refresh_expired_urls: boolean;
}

export interface NamingPreset {
  id: string;
  name: string;
  template: string;
}

export interface RemovalPreview {
  token: string;
  task_ids: string[];
  files: string[];
  file_count: number;
  total_bytes: number;
  preserved: string[];
  roots: string[];
}

export interface WorkflowMediaInput {
  enabled: boolean;
  save: boolean;
  format: 'm4s' | 'mp3';
  retain_original: boolean;
}
export interface WorkflowAssetOutput {
  enabled: boolean;
  save: boolean;
  embed: boolean;
  format: 'original' | 'srt' | 'ass' | 'xml' | 'html';
  retain_original: boolean;
}
export interface NfoExtraFields {
  sort_title: string;
  original_title: string;
  tagline: string;
  outline: string;
  genres: string;
  tags: string;
  collection: string;
  collection_overview: string;
  directors: string;
  writers: string;
  studios: string;
  countries: string;
}
export interface NfoOptions {
  title_template: string;
  plot_template: string;
  include_plot: boolean;
  include_date: boolean;
  include_runtime: boolean;
  include_genre: boolean;
  include_source: boolean;
  include_cover: boolean;
  uploader_as_actor: boolean;
  custom_tags: string;
  extra: NfoExtraFields;
}
export interface DownloadWorkflow {
  video: WorkflowMediaInput;
  audio: WorkflowMediaInput;
  cover: WorkflowAssetOutput;
  subtitles: WorkflowAssetOutput;
  danmaku: WorkflowAssetOutput;
  nfo: boolean;
  nfo_options?: NfoOptions;
  merge_media: boolean;
  container: 'mp4' | 'mkv';
  quality: string;
  audio_quality: string;
  codec: VideoCodecPreference;
  media_preferences: MediaPreferences;
  missing_quality_policy: MissingQualityPolicy;
}
export interface DownloadPreset {
  id: string;
  name: string;
  workflow: DownloadWorkflow;
}
export interface DownloadArtifact {
  path: string;
  intent: DownloadResourceIntent | null;
  resource_id: string | null;
  original: boolean;
  document_uri?: string | null;
}

export interface MaintenanceResult {
  removed_files: number;
  path: string;
}

export interface LogMaintenanceUsage {
  log_count: number;
  log_bytes: number;
  cleanable_log_count: number;
  diagnostic_files: number;
  diagnostic_bytes: number;
}

export interface TaskLogCleanup {
  removed_logs: number;
  task_ids: string[];
}

export interface OperationErrorRecord {
  id: number;
  command: string;
  code: string;
  message: string;
  detail: string | null;
  environment: DiagnosticEnvironment;
  created_at: string;
}

export type DownloadDirectoryStatus = 'ready' | 'missing' | 'not_directory' | 'unwritable';
export type FfmpegStatus = 'ready' | 'missing' | 'invalid';
export type FfmpegSource = 'configured' | 'system' | 'native';

export interface DownloadDirectoryHealth {
  status: DownloadDirectoryStatus;
  path: string;
  message: string;
}

export interface FfmpegHealth {
  status: FfmpegStatus;
  source: FfmpegSource;
  path: string | null;
  version: string | null;
  message: string;
}

export interface EnvironmentHealthSnapshot {
  ready: boolean;
  download_directory: DownloadDirectoryHealth;
  ffmpeg: FfmpegHealth;
}

export interface DiagnosticsExportResponse {
  path: string;
  saved: boolean;
}

export interface AccountSummary {
  logged_in: boolean;
  name: string | null;
  avatar_url: string | null;
  mid: string | null;
  vip_label: string | null;
}

export type AccountLibraryFolderKind = 'created_favorite' | 'collected_favorite';

export interface AccountLibraryFolder {
  kind: AccountLibraryFolderKind;
  media_id: string;
  title: string;
  description: string | null;
  cover_url: string | null;
  owner_name: string | null;
  owner_mid: string | null;
  media_count: number;
  source_url: string;
}

export interface AccountLibraryPage {
  items: AccountLibraryFolder[];
  total: number;
  page: number;
  page_size: number;
  has_more: boolean;
}

export type QrLoginStatus = 'waiting' | 'scanned' | 'confirmed' | 'expired' | 'unknown';

export interface QrLoginSession {
  qr_url: string;
  qrcode_key: string;
  qr_image_svg: string;
  expires_in_seconds: number;
}

export interface QrLoginPollResponse {
  status: QrLoginStatus;
  message: string;
  account: AccountSummary | null;
}

export interface ParseRules {
  interval_seconds: number;
  rest_seconds: number;
  pages_per_round: number;
}

export type LibraryCategory = AccountLibraryFolderKind | 'watch_later';
export type LibraryContentSource = Pick<AccountLibraryFolder, 'title' | 'source_url' | 'owner_name' | 'media_count'> & {
  parsed_source_id?: string;
  cover_url?: string | null;
};

export type LibraryBrowseItem = Omit<AccountLibraryFolder, 'kind'> & { kind: LibraryCategory };
export type LibraryBrowsePage = Omit<AccountLibraryPage, 'items'> & { items: LibraryBrowseItem[] };
