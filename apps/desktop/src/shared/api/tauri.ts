import { invoke } from '@tauri-apps/api/core'
import { recordCommandFailure, replaceRecentFailures, clearRecentFailures } from '../utils/recentFailures'
import { redactDiagnosticMessage } from '../stores/transferView'
import { boundedDiagnostic } from '../utils/commandDiagnostic'

import type {
  AccountSummary,
  OperationErrorRecord,
  AccountLibraryFolderKind,
  AccountLibraryPage,
  BulkQueueResult,
  DiagnosticsExportResponse,
  DiagnosticEnvironment,
  DocumentTreeDirectory,
  DownloadTask,
  DownloadDirectoryHealth,
  DuplicateTaskPolicy,
  MaintenanceResult,
  LogMaintenanceUsage,
  TaskLogCleanup,
  EnvironmentHealthSnapshot,
  NormalizedSourceTree,
  QrLoginPollResponse,
  QrLoginSession,
  QueueLogEntry,
  QueueRemoveResponse,
  RemovalPreview,
  ArchiveMode,
  DownloadMediaMode,
  SettingsSnapshot,
  SelectionCreateTasksResult,
  PreparationJob,
  SelectionSizeEstimate,
  StartupRecoverySnapshot,
  VideoCodecPreference,
} from './dto'

export interface CommandErrorShape {
  code: string
  message: string
  detail?: string
}

export class BdlCommandError extends Error {
  readonly code: string
  readonly detail?: string

  constructor(error: CommandErrorShape) {
    super(error.message)
    this.name = 'BdlCommandError'
    this.code = error.code
    this.detail = typeof error.detail === 'string' ? error.detail : undefined
  }
}

export interface ParseCreateSourceRequest {
  input: string
  fetch_streams?: boolean
  expand_video_collection?: boolean
}

export interface ParseSourcePageRequest {
  source_id: string
  snapshot: NormalizedSourceTree
  operation_id: string
}

export interface ParseLoadAllRequest {
  source_id: string
  snapshot: NormalizedSourceTree
  operation_id: string
  limit?: number
}

export interface SelectionCreateTasksRequest {
  download_preset_id?: string
  source_id: string
  part_ids: string[]
  output_dir?: string
  document_tree_output?: DocumentTreeDirectory
  archive_mode?: ArchiveMode
  output_extension?: string
  subtitle_format?: SettingsSnapshot['subtitle_format']
  danmaku_format?: SettingsSnapshot['danmaku_format']
  naming_template?: string
  duplicate_naming_strategy?: SettingsSnapshot['duplicate_naming_strategy']
  archive_assets?: SettingsSnapshot['archive_assets']
  retain_raw_streams?: boolean
  embed_cover?: boolean
  embed_subtitles?: boolean
  missing_quality_policy?: SettingsSnapshot['missing_quality_policy']
  media_mode?: DownloadMediaMode
  quality?: string
  audio_quality?: string
  media_preferences?: SettingsSnapshot['media_preferences']
  codec?: VideoCodecPreference
  duplicate_policy?: DuplicateTaskPolicy
  scheduled_at?: string
  speed_limit_bytes_per_second?: number
}

export interface SelectionEstimateSizeRequest {
  download_preset_id?: string
  source_id: string
  part_ids: string[]
  missing_quality_policy?: SettingsSnapshot['missing_quality_policy']
  media_mode?: DownloadMediaMode
  quality?: string
  audio_quality?: string
  media_preferences?: SettingsSnapshot['media_preferences']
  codec?: VideoCodecPreference
}

export interface AccountImportCookieRequest {
  cookie: string
}

export interface AccountLoginQrPollRequest {
  qrcode_key: string
}

export interface AccountLibraryRequest {
  kind: AccountLibraryFolderKind
  page: number
  page_size: number
}

export interface BulkQueueRequest {
  task_ids: string[]
}

export interface EnvironmentHealthRequest {
  download_dir: string | null
  ffmpeg_path: string | null
}

export type NotificationPermissionState =
  | 'granted'
  | 'denied'
  | 'prompt'
  | 'prompt-with-rationale'
  | 'unsupported'

export const parseCreateSource = (request: ParseCreateSourceRequest, operationId: string) =>
  invokeCommand<NormalizedSourceTree>('parse_create_source', { request, operationId })

export const parseLoadMore = (request: ParseSourcePageRequest) =>
  invokeCommand<NormalizedSourceTree>('parse_load_more', { request })

export const parseLoadPage = (request: ParseSourcePageRequest & { page_number: number }) =>
  invokeCommand<{ tree: NormalizedSourceTree; item_ids: string[] }>('parse_load_page', { request })

export type ParseCancelReason = 'stop_requested' | 'workspace_cleared' | 'library_deactivated' | 'library_folder_closed' | 'account_changed'
export const parseCancel = (sourceId: string, reason: ParseCancelReason) => invokeCommand<void>('parse_cancel', { sourceId, reason })

export const parseProgress = (sourceId: string) =>
  invokeCommand<{ active: boolean; waiting_seconds: number; queued: boolean }>('parse_progress', { sourceId })

export const parseLoadAll = (request: ParseLoadAllRequest) =>
  invokeCommand<NormalizedSourceTree>('parse_load_all', { request })

export const parseRefreshSource = (request: ParseSourcePageRequest) =>
  invokeCommand<NormalizedSourceTree>('parse_refresh_source', { request })

export const selectionCreateTasks = (request: SelectionCreateTasksRequest, snapshot: NormalizedSourceTree, operationId: string) =>
  invokeCommand<SelectionCreateTasksResult>('selection_create_tasks', { request, snapshot, operationId })

export const selectionSubmitTasks = (request: SelectionCreateTasksRequest, snapshot: NormalizedSourceTree, origin: string) =>
  invokeCommand<PreparationJob>('selection_submit_tasks', { request, snapshot, origin })
export const preparationList = () => invokeCommand<PreparationJob[]>('preparation_list')
export const preparationCancel = (jobId: string) => invokeCommand<void>('preparation_cancel', { jobId })
export const preparationConfirm = (jobId: string, policy: DuplicateTaskPolicy) => invokeCommand<void>('preparation_confirm', { jobId, policy })
export const preparationDismiss = (jobId: string) => invokeCommand<void>('preparation_dismiss', { jobId })

export const selectionEstimateSize = (request: SelectionEstimateSizeRequest, snapshot: NormalizedSourceTree) =>
  invokeCommand<SelectionSizeEstimate>('selection_estimate_size', { request, snapshot })

export const selectionPrepare = (sourceId: string, partIds: string[], snapshot: NormalizedSourceTree, operationId: string) =>
  invokeCommand<NormalizedSourceTree>('selection_prepare', { sourceId, partIds, snapshot, operationId })

export const mobilePickExportDirectory = () =>
  invokeCommand<DocumentTreeDirectory>('mobile_pick_export_directory')

export const writeNativeClipboardText = (text: string, android: boolean) =>
  invokeCommand<void>(android ? 'mobile_write_clipboard_text' : 'desktop_write_clipboard_text', { text })

export const mobileSaveImageToGallery = (fileName: string, imageBase64: string) =>
  invokeCommand<string>('mobile_save_image_to_gallery', { fileName, imageBase64 })

export const mobilePrepareNotifications = () =>
  invokeCommand<NotificationPermissionState>('mobile_prepare_notifications')

export const queueList = () => invokeCommand<DownloadTask[]>('queue_list')

export const queueStartupRecovery = () => invokeCommand<StartupRecoverySnapshot>('queue_startup_recovery')

export const queueDismissStartupRecovery = () =>
  invokeCommand<StartupRecoverySnapshot>('queue_dismiss_startup_recovery')

export const queueLogs = (taskId: string, limit = 200) =>
  invokeCommand<QueueLogEntry[]>('queue_logs', { taskId, limit })

export const queuePause = (taskId: string) => invokeCommand<DownloadTask>('queue_pause', { taskId })

export const queueResume = (taskId: string) => invokeCommand<DownloadTask>('queue_resume', { taskId })

export const queueSchedule = (taskId: string, scheduledAt: string) =>
  invokeCommand<DownloadTask>('queue_schedule', { request: { task_id: taskId, scheduled_at: scheduledAt } })

export const queueUnschedule = (taskId: string) =>
  invokeCommand<DownloadTask>('queue_unschedule', { taskId })

export const queueSetSpeedLimit = (taskId: string, speedLimitBytesPerSecond?: number) =>
  invokeCommand<DownloadTask>('queue_set_speed_limit', {
    request: { task_id: taskId, speed_limit_bytes_per_second: speedLimitBytesPerSecond },
  })

export const queueCancel = (taskId: string) => invokeCommand<DownloadTask>('queue_cancel', { taskId })

export const queueRetry = (taskId: string) => invokeCommand<DownloadTask>('queue_retry', { taskId })

export const queueRefreshUrlsAndRetry = (taskId: string) =>
  invokeCommand<DownloadTask>('queue_refresh_urls_and_retry', { taskId })

export const queueBulkPause = (request: BulkQueueRequest) =>
  invokeCommand<BulkQueueResult>('queue_bulk_pause', { request })

export const queueBulkCancel = (request: BulkQueueRequest) =>
  invokeCommand<BulkQueueResult>('queue_bulk_cancel', { request })

export const queueBulkResume = (request: BulkQueueRequest) =>
  invokeCommand<BulkQueueResult>('queue_bulk_resume', { request })

export const queueBulkRetry = (request: BulkQueueRequest) =>
  invokeCommand<BulkQueueResult>('queue_bulk_retry', { request })

export const queueBulkRefreshUrlsAndRetry = (request: BulkQueueRequest) =>
  invokeCommand<BulkQueueResult>('queue_bulk_refresh_urls_and_retry', { request })

export const queueBulkRemove = (request: BulkQueueRequest) =>
  invokeCommand<BulkQueueResult>('queue_bulk_remove', { request })

export const queueClearCompleted = () => invokeCommand<BulkQueueResult>('queue_clear_completed')

export const queueRemove = (taskId: string) => invokeCommand<QueueRemoveResponse>('queue_remove', { taskId })

export const queueDeletePreview = (taskIds: string[]) =>
  invokeCommand<RemovalPreview>('queue_delete_preview', { request: { task_ids: taskIds } })
export const queueDeleteFiles = (token: string) => invokeCommand<BulkQueueResult>('queue_delete_files', { token })
export const maintenanceTempPreview = () => invokeCommand<RemovalPreview>('maintenance_temp_preview')

export const queueOpenFile = (taskId: string) => invokeCommand<void>('queue_open_file', { taskId })

export const queueOutputSizes = (taskIds: string[]) =>
  invokeCommand<Record<string, number | null>>('queue_output_sizes', { taskIds })

export const queueOpenDir = (taskId: string) => invokeCommand<void>('queue_open_dir', { taskId })

export const accountGet = () => invokeCommand<AccountSummary>('account_get')

export const accountLoginQrStart = (requestId: string) => invokeCommand<QrLoginSession>('account_login_qr_start', { requestId })
export const accountLoginQrCancel = (requestId: string) => invokeCommand<void>('account_login_qr_cancel', { requestId })

export const accountLoginQrPoll = (request: AccountLoginQrPollRequest) =>
  invokeCommand<QrLoginPollResponse>('account_login_qr_poll', { request })

export const accountImportCookie = (request: AccountImportCookieRequest) =>
  invokeCommand<AccountSummary>('account_import_cookie', { request })

export const accountLogout = () => invokeCommand<AccountSummary>('account_logout')

export const accountVerify = () => invokeCommand<AccountSummary>('account_verify')

export const accountLibraryList = (request: AccountLibraryRequest) =>
  invokeCommand<AccountLibraryPage>('account_library_list', { request })

export const settingsGet = () => invokeCommand<SettingsSnapshot>('settings_get')

export const settingsUpdate = (settings: SettingsSnapshot) =>
  invokeCommand<SettingsSnapshot>('settings_update', { settings })

export const environmentHealth = (request: EnvironmentHealthRequest) =>
  invokeCommand<EnvironmentHealthSnapshot>('environment_health', { request })

export const environmentCreateDownloadDirectory = (path: string) =>
  invokeCommand<DownloadDirectoryHealth>('environment_create_download_directory', { request: { path } })

export const maintenanceCleanupCache = () => invokeCommand<MaintenanceResult>('maintenance_cleanup_cache')

export const maintenanceCleanupTemp = (token: string) => invokeCommand<MaintenanceResult>('maintenance_cleanup_temp', { token })

export const maintenanceLogUsage = () => invokeCommand<LogMaintenanceUsage>('maintenance_log_usage')
export const maintenanceCleanupLogs = async () => {
  const result = await invokeCommand<TaskLogCleanup>('maintenance_cleanup_logs')
  clearRecentFailures()
  return result
}
export const maintenanceCleanupDiagnostics = () => invokeCommand<number>('maintenance_cleanup_diagnostics')
export const diagnosticsExport = (taskId?: string) => invokeCommand<DiagnosticsExportResponse>('diagnostics_export', { taskId: taskId ?? null })
export const refreshOperationErrors = async () => {
  const records = await invokeCommand<OperationErrorRecord[]>('diagnostics_operation_errors')
  replaceRecentFailures(records)
}
export const diagnosticsEnvironment = () => invokeCommand<DiagnosticEnvironment>('diagnostics_environment')

export const openExternalUrl = (url: string) => invokeCommand<void>('open_external_url', { url })

const invokeCommand = async <T>(command: string, args?: Record<string, unknown>): Promise<T> => {
  try {
    return await invoke<T>(command, args)
  } catch (error) {
    let normalized = normalizeCommandError(error, command)
    if (normalized.code === 'parse_cancelled') throw normalized
    const safe = { code: normalized.code, message: boundedDiagnostic(normalized.message, 4000),
      detail: normalized.detail ? boundedDiagnostic(normalized.detail) : undefined }
    try {
      // Raw invoke avoids recursively trying to journal a journal-write failure.
      const id = await invoke<number>('diagnostics_record_operation_error', { command, error: safe })
      normalized = new BdlCommandError({ ...safe, detail: `错误记录: ${id}\n${safe.detail ?? safe.message}` })
    } catch (journalError) {
      const cause = normalizeCommandError(journalError, 'diagnostics_record_operation_error')
      normalized = new BdlCommandError({ ...safe, detail: `${safe.detail ?? safe.message}\n[stage=diagnostic_persist] 操作错误未能写入日志: ${redactDiagnosticMessage(cause.detail ?? cause.message)}` })
    }
    recordCommandFailure(command, normalized)
    throw normalized
  }
}

const normalizeCommandError = (error: unknown, command: string): BdlCommandError => {
  const value = error && typeof error === 'object' ? error as Record<string, unknown> : {}
  const normalized = isCommandErrorShape(error)
    ? error
    : { code: 'frontend_error', message: typeof value.message === 'string' ? value.message : String(error), detail: typeof value.detail === 'string' ? value.detail : undefined }
  const internalBpiError = ['bilibili_response_decode_failed', 'bilibili_api_error'].includes(normalized.code)
    || /bpi error:|failed to decode response/i.test(normalized.message)
  if (internalBpiError) {
    const action = command.startsWith('parse_') || command.startsWith('selection_')
      ? '解析失败'
      : command === 'account_library_list'
        ? '收藏夹加载失败'
        : command.startsWith('queue_')
          ? '下载操作失败'
          : '操作失败'
    return new BdlCommandError({ code: normalized.code, message: `${action}，请稍后重试；如果持续失败，请反馈问题。`, detail: normalized.detail ?? normalized.message })
  }
  return new BdlCommandError(normalized)
}

const isCommandErrorShape = (value: unknown): value is CommandErrorShape => {
  if (!value || typeof value !== 'object') {
    return false
  }

  const candidate = value as Record<string, unknown>
  return typeof candidate.code === 'string' && typeof candidate.message === 'string'
}
