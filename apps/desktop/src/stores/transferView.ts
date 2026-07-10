import type { DownloadResourceIntent, DownloadTask, QueueLogEntry, TaskStatus } from '../api/dto'
import { formatSpeedLimit } from '../utils/speedLimit'

export type QueueFilter = 'active' | 'failed' | 'completed' | 'all'

export type TaskActionKind =
  | 'pause'
  | 'resume'
  | 'retry'
  | 'refresh_retry'
  | 'cancel'
  | 'remove'
  | 'open_file'
  | 'open_dir'
  | 'copy_source'
  | 'unschedule'
  | 'schedule'
  | 'speed_limit'
  | 'none'

export interface TaskActionDescriptor {
  kind: Exclude<TaskActionKind, 'none'>
  label: string
  icon: string
  tone?: 'normal' | 'danger'
}

export type DiagnosticTone = 'normal' | 'success' | 'warning' | 'danger'

export interface TaskDiagnosticView {
  summary: string
  detail: string
  impact: string
  recommendedAction: Exclude<TaskActionKind, 'none'> | null
  recommendedActionLabel: string
  tone: DiagnosticTone
}

export interface TaskTimelineEvent {
  id: string
  time: string
  title: string
  detail: string
  tone: DiagnosticTone
}

export interface TransferProgressSnapshot {
  downloadedBytes: number
  totalBytes: number | null
  speedBytesPerSecond: number
  updatedAt: number
}

export interface TransferTaskView {
  id: string
  isCompleted: boolean
  displayTitle: string
  subtitle: string
  statusLabel: string
  statusBadge: 'ready' | 'downloading' | 'queued' | 'done' | 'warning' | 'error' | 'paused'
  progressValue: number
  progressLabel: string
  speedLabel: string
  etaLabel: string
  sizeLabel: string
  issueLabel: string
  shortLocation: string
  fullLocation: string
  outputPath: string
  sourceId: string
  primaryAction: TaskActionKind
  primaryActionLabel: string
  primaryActionIcon: string
  secondaryActions: TaskActionDescriptor[]
}

export const createTransferTaskView = (
  task: DownloadTask,
  progress: number,
  logs: QueueLogEntry[] = [],
  transferProgress: TransferProgressSnapshot | null = null,
): TransferTaskView => {
  const titleParts = splitTaskTitle(task.title)
  const issue = classifyTaskIssue(task, logs)
  const primaryAction = primaryActionForTask(task, issue)
  const completedWithWarnings = task.status === 'completed' && hasWarningLogs(logs)
  const scheduled = isScheduledTask(task)
  const activelyTransferring = task.status === 'downloading'

  return {
    id: task.id,
    isCompleted: task.status === 'completed',
    displayTitle: titleParts.displayTitle,
    subtitle: titleParts.subtitle,
    statusLabel: completedWithWarnings ? '部分失败' : scheduled ? '已定时' : statusLabel(task.status),
    statusBadge: completedWithWarnings ? 'warning' : statusBadge(task.status),
    progressValue: progress,
    progressLabel: `${progress}%`,
    speedLabel:
      activelyTransferring && transferProgress ? formatSpeedLabel(transferProgress.speedBytesPerSecond) : '--',
    etaLabel: scheduled
      ? scheduleLabel(task.scheduled_at)
      : activelyTransferring && transferProgress
        ? etaLabel(transferProgress)
        : '--',
    sizeLabel: transferProgress ? sizeLabel(transferProgress) : '--',
    issueLabel: issue.label,
    shortLocation: shortLocation(task.output_path),
    fullLocation: outputDir(task.output_path),
    outputPath: task.output_path,
    sourceId: task.source_id,
    primaryAction,
    primaryActionLabel: actionLabel(primaryAction),
    primaryActionIcon: actionIcon(primaryAction),
    secondaryActions: secondaryActionsForTask(task, primaryAction),
  }
}

export const formatSpeedLabel = (bytesPerSecond: number): string => {
  if (bytesPerSecond <= 0 || !Number.isFinite(bytesPerSecond)) {
    return '--'
  }

  return `${formatBytes(bytesPerSecond)}/s`
}

const sizeLabel = (progress: TransferProgressSnapshot): string => {
  if (progress.totalBytes && progress.totalBytes > 0) {
    return `${formatBytes(progress.downloadedBytes)} / ${formatBytes(progress.totalBytes)}`
  }

  return progress.downloadedBytes > 0 ? formatBytes(progress.downloadedBytes) : '--'
}

const etaLabel = (progress: TransferProgressSnapshot): string => {
  if (!progress.totalBytes || progress.totalBytes <= 0 || progress.speedBytesPerSecond <= 0) {
    return '--'
  }

  const remainingBytes = Math.max(0, progress.totalBytes - progress.downloadedBytes)
  if (remainingBytes === 0) {
    return '0s'
  }

  const seconds = Math.ceil(remainingBytes / progress.speedBytesPerSecond)
  if (seconds < 60) {
    return `${seconds}s`
  }
  if (seconds < 3600) {
    return `${Math.ceil(seconds / 60)}m`
  }

  return `${Math.ceil(seconds / 3600)}h`
}

const formatBytes = (value: number): string => {
  if (!Number.isFinite(value) || value <= 0) {
    return '0 B'
  }

  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  let size = value
  let unitIndex = 0
  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024
    unitIndex += 1
  }

  const precision = unitIndex === 0 || size >= 10 ? 0 : 1
  return `${size.toFixed(precision)} ${units[unitIndex]}`
}

export const createTaskDiagnosticView = (task: DownloadTask, logs: QueueLogEntry[] = []): TaskDiagnosticView => {
  const issue = classifyTaskIssue(task, logs)
  const failedCount = task.resources.filter(
    (resource) => resource.status === 'failed' || resource.status === 'cancelled',
  ).length
  const completedCount = task.resources.filter((resource) => resource.status === 'completed').length
  const taskLimit = task.speed_limit_bytes_per_second
    ? ` · 限速 ${formatSpeedLimit(task.speed_limit_bytes_per_second)}`
    : ''
  const trackImpact = `轨道 ${task.resources.length} 个 · 已完成 ${completedCount} · 失败 ${failedCount}${taskLimit}`
  const warningLogs = completionWarningLogs(logs)

  if (isScheduledTask(task)) {
    return {
      summary: '任务已定时',
      detail: `将在 ${scheduleLabel(task.scheduled_at, true)} 自动进入下载队列。`,
      impact: trackImpact,
      recommendedAction: 'unschedule',
      recommendedActionLabel: actionLabel('unschedule'),
      tone: 'normal',
    }
  }

  if (task.status === 'completed' && warningLogs.length > 0) {
    return {
      summary: '任务已完成，但部分附加内容未处理',
      detail: warningLogs.map((log) => redactLogMessage(log.message)).join('；'),
      impact: trackImpact,
      recommendedAction: 'open_file',
      recommendedActionLabel: actionLabel('open_file'),
      tone: 'warning',
    }
  }

  if (task.status === 'completed') {
    return {
      summary: '任务已完成',
      detail: '输出文件已生成，可以直接打开文件或所在文件夹。',
      impact: trackImpact,
      recommendedAction: 'open_file',
      recommendedActionLabel: actionLabel('open_file'),
      tone: 'success',
    }
  }

  if (task.status === 'cancelled') {
    return {
      summary: '任务已取消',
      detail: '任务已停止，已下载的临时文件会保留给后续恢复逻辑使用。',
      impact: trackImpact,
      recommendedAction: 'retry',
      recommendedActionLabel: actionLabel('retry'),
      tone: 'warning',
    }
  }

  if (task.status === 'paused') {
    return {
      summary: '任务已暂停',
      detail: '任务暂时不会继续下载，点击继续可以重新放回队列。',
      impact: trackImpact,
      recommendedAction: 'resume',
      recommendedActionLabel: actionLabel('resume'),
      tone: 'warning',
    }
  }

  if (task.status === 'failed') {
    return {
      summary: `${issue.trackLabel}失败：${issue.label}`,
      detail: issue.detail,
      impact: trackImpact,
      recommendedAction: issue.recommendedAction,
      recommendedActionLabel: issue.recommendedActionLabel,
      tone: 'danger',
    }
  }

  return {
    summary: statusLabel(task.status),
    detail: '任务正在按队列流程执行。速度、剩余时间和大小会在下载引擎提供数据后显示。',
    impact: trackImpact,
    recommendedAction:
      task.status === 'waiting' || task.status === 'parsing' || task.status === 'downloading' ? 'pause' : null,
    recommendedActionLabel:
      task.status === 'waiting' || task.status === 'parsing' || task.status === 'downloading'
        ? actionLabel('pause')
        : '',
    tone: 'normal',
  }
}

export const createTaskTimeline = (task: DownloadTask, logs: QueueLogEntry[] = []): TaskTimelineEvent[] => {
  if (!logs.length) {
    return [
      {
        id: `${task.id}:status`,
        time: '',
        title: statusLabel(task.status),
        detail: '暂无事件记录。',
        tone: task.status === 'failed' || task.status === 'cancelled' ? 'danger' : 'normal',
      },
    ]
  }

  return logs.map((log, index) => {
    const issue = log.level === 'error' ? classifyTaskIssue(task, [log]) : null
    const title = issue ? `下载失败：${issue.label}` : humanLogTitle(log.message)

    return {
      id: `${log.created_at}:${index}:${log.message}`,
      time: log.created_at,
      title,
      detail: redactLogMessage(log.message),
      tone: log.level === 'error' ? 'danger' : log.level === 'warning' ? 'warning' : 'normal',
    }
  })
}

export const redactLogMessage = (message: string): string =>
  truncateLogMessage(
    message
      .replace(/\b(Cookie|Authorization|Proxy-Authorization):\s*[^\n\r]+/gi, '$1: <redacted>')
      .replace(
        /\b(SESSDATA|bili_jct|DedeUserID|DedeUserID__ckMd5|buvid3|sid|access_key|token|deadline|expires|bili_ticket|bili_ticket_expires|sign)=([^;&\s]+)/gi,
        '$1=<redacted>',
      )
      .replace(/https?:\/\/[^\s`"')\]}<>]+/gi, (rawUrl) => redactUrl(rawUrl)),
  )

export const filterTaskByWorkflow = (status: TaskStatus, filter: QueueFilter): boolean => {
  switch (filter) {
    case 'active':
      return status !== 'completed'
    case 'failed':
      return status === 'failed' || status === 'cancelled'
    case 'completed':
      return status === 'completed'
    case 'all':
      return true
  }
}

export const defaultQueueFilter = (tasks: DownloadTask[]): QueueFilter => {
  if (tasks.some((task) => task.status !== 'completed')) {
    return 'active'
  }

  if (tasks.some((task) => task.status === 'completed')) {
    return 'completed'
  }

  return 'active'
}

export const statusLabel = (status: TaskStatus): string => {
  const labels: Record<TaskStatus, string> = {
    waiting: '队列中',
    parsing: '解析中',
    downloading: '下载中',
    muxing: '合并中',
    completed: '已完成',
    failed: '失败',
    paused: '已暂停',
    cancelled: '已取消',
  }

  return labels[status]
}

export const statusBadge = (
  status: TaskStatus,
): 'ready' | 'downloading' | 'queued' | 'done' | 'warning' | 'error' | 'paused' => {
  if (status === 'completed') {
    return 'done'
  }
  if (status === 'failed' || status === 'cancelled') {
    return 'error'
  }
  if (status === 'paused') {
    return 'paused'
  }
  if (status === 'waiting') {
    return 'queued'
  }
  if (status === 'downloading' || status === 'parsing' || status === 'muxing') {
    return 'downloading'
  }

  return 'ready'
}

const splitTaskTitle = (title: string): { displayTitle: string; subtitle: string } => {
  const bracketMatch = title.match(/^(.*?)\s+-\s+\[([^\]]+)\]\s+-\s+(.+)$/)
  if (bracketMatch) {
    return {
      displayTitle: `${bracketMatch[2]} · ${bracketMatch[3]}`,
      subtitle: bracketMatch[1],
    }
  }

  const parts = title.split(/\s+-\s+/)
  if (parts.length >= 2) {
    const displayTitle = parts.pop() ?? title
    return {
      displayTitle,
      subtitle: parts.join(' - '),
    }
  }

  return {
    displayTitle: title,
    subtitle: '',
  }
}

interface ClassifiedIssue {
  label: string
  detail: string
  trackLabel: string
  recommendedAction: Exclude<TaskActionKind, 'none'> | null
  recommendedActionLabel: string
}

const classifyTaskIssue = (task: DownloadTask, logs: QueueLogEntry[]): ClassifiedIssue => {
  const failedResource = task.resources.find(
    (resource) => resource.status === 'failed' || resource.status === 'cancelled',
  )
  const trackLabel = failedResource ? `${resourceIntentText(failedResource.intent)}轨道` : '任务'

  if (task.status === 'cancelled') {
    return {
      label: '已取消',
      detail: '任务被取消，可以重新入队后继续尝试。',
      trackLabel,
      recommendedAction: 'retry',
      recommendedActionLabel: actionLabel('retry'),
    }
  }

  const warningLogs = completionWarningLogs(logs)
  if (task.status === 'completed' && warningLogs.length > 0) {
    return {
      label: '附加内容失败',
      detail: warningLogs.map((log) => redactLogMessage(log.message)).join('；'),
      trackLabel: '附加内容',
      recommendedAction: 'open_file',
      recommendedActionLabel: actionLabel('open_file'),
    }
  }

  if (task.status !== 'failed') {
    return {
      label: '-',
      detail: '',
      trackLabel,
      recommendedAction: null,
      recommendedActionLabel: '',
    }
  }

  const lastErrors = logs
    .filter((log) => log.level === 'error')
    .map((log) => log.message)
    .join('\n')
    .toLowerCase()

  if (
    lastErrors.includes('404') ||
    lastErrors.includes('资源长度失败') ||
    (lastErrors.includes('not found') && !lastErrors.includes('ffmpeg'))
  ) {
    return {
      label: '链接可能已过期',
      detail: 'B 站的媒体直链有时效性，资源长度请求返回 404 时，通常需要重新获取下载地址后再下载。',
      trackLabel,
      recommendedAction: 'refresh_retry',
      recommendedActionLabel: actionLabel('refresh_retry'),
    }
  }
  if (
    lastErrors.includes('permission denied') ||
    lastErrors.includes('access is denied') ||
    lastErrors.includes('拒绝访问') ||
    lastErrors.includes('保存目录不可写') ||
    lastErrors.includes('readonly')
  ) {
    return {
      label: '保存目录不可写',
      detail: '下载器无法写入目标目录。请检查目录权限、磁盘状态，或在设置中更换保存目录。',
      trackLabel,
      recommendedAction: null,
      recommendedActionLabel: '查看原始日志',
    }
  }
  if (
    lastErrors.includes('private') ||
    lastErrors.includes('私密') ||
    lastErrors.includes('不可见') ||
    lastErrors.includes('无权访问') ||
    lastErrors.includes('访问受限')
  ) {
    return {
      label: '资源不可访问',
      detail: '资源可能是私密稿件、已失效，或当前账号没有访问权限。登录后再重试。',
      trackLabel,
      recommendedAction: 'retry',
      recommendedActionLabel: '登录后重试',
    }
  }
  if (
    lastErrors.includes('403') ||
    lastErrors.includes('401') ||
    lastErrors.includes('forbidden') ||
    lastErrors.includes('unauthorized') ||
    lastErrors.includes('登录') ||
    lastErrors.includes('cookie') ||
    lastErrors.includes('权限')
  ) {
    return {
      label: '权限或登录异常',
      detail: '当前账号可能未登录、Cookie 已失效，或该清晰度需要登录/VIP 权限。登录后再重试。',
      trackLabel,
      recommendedAction: 'retry',
      recommendedActionLabel: '登录后重试',
    }
  }
  if (lastErrors.includes('ffmpeg not found') || lastErrors.includes('未找到 ffmpeg')) {
    return {
      label: '未找到 FFmpeg',
      detail: '未检测到可用的 FFmpeg。请在设置中配置 FFmpeg 路径后再重试。',
      trackLabel,
      recommendedAction: null,
      recommendedActionLabel: '查看设置',
    }
  }
  if (lastErrors.includes('ffmpeg') || lastErrors.includes('mux') || lastErrors.includes('合并')) {
    return {
      label: '合并失败',
      detail: '下载轨道已经进入后处理阶段，但 ffmpeg 或封装命令失败。需要检查 ffmpeg 配置和原始日志。',
      trackLabel,
      recommendedAction: null,
      recommendedActionLabel: '查看原始日志',
    }
  }
  if (lastErrors.includes('timeout') || lastErrors.includes('timed out') || lastErrors.includes('超时')) {
    return {
      label: '网络超时',
      detail: '请求在限定时间内没有完成，通常可以直接重试。',
      trackLabel,
      recommendedAction: 'retry',
      recommendedActionLabel: actionLabel('retry'),
    }
  }

  if (failedResource) {
    return {
      label: `${resourceIntentText(failedResource.intent)}轨道失败`,
      detail: '某个媒体轨道下载失败，具体原因请查看事件或原始日志。',
      trackLabel,
      recommendedAction: 'retry',
      recommendedActionLabel: actionLabel('retry'),
    }
  }

  return {
    label: '任务失败',
    detail: '任务执行失败，具体原因请查看事件或原始日志。',
    trackLabel,
    recommendedAction: 'retry',
    recommendedActionLabel: actionLabel('retry'),
  }
}

const transientWarningMessages = new Set(['任务已停止', '任务已暂停或取消', '自动刷新过期链接'])

const completionWarningLogs = (logs: QueueLogEntry[]): QueueLogEntry[] =>
  logs.filter((log) => log.level === 'warning' && !transientWarningMessages.has(log.message.trim()))

const hasWarningLogs = (logs: QueueLogEntry[]): boolean => completionWarningLogs(logs).length > 0

const primaryActionForTask = (task: DownloadTask, issue: ClassifiedIssue): TaskActionKind => {
  if (isScheduledTask(task)) {
    return 'unschedule'
  }
  if (task.status === 'completed') {
    return 'open_file'
  }
  if (task.status === 'paused') {
    return 'resume'
  }
  if (task.status === 'failed') {
    return issue.recommendedAction ?? 'retry'
  }
  if (task.status === 'cancelled') {
    return 'retry'
  }
  if (task.status === 'waiting' || task.status === 'parsing' || task.status === 'downloading') {
    return 'pause'
  }

  if (task.status === 'muxing') {
    return 'none'
  }

  return 'none'
}

const secondaryActionsForTask = (task: DownloadTask, primaryAction: TaskActionKind): TaskActionDescriptor[] => {
  const actions: TaskActionDescriptor[] = []

  if (isScheduledTask(task)) {
    actions.push(actionDescriptor('schedule', '修改时间'), actionDescriptor('speed_limit'), actionDescriptor('cancel'))
    return actions
  }

  if (task.status === 'completed') {
    actions.push(
      actionDescriptor('open_dir'),
      actionDescriptor('retry', '重新下载'),
      actionDescriptor('copy_source'),
      actionDescriptor('remove'),
    )
    return actions.filter((action) => action.kind !== primaryAction)
  }

  if (task.status === 'failed' || task.status === 'cancelled') {
    actions.push(actionDescriptor('retry'))
    if (task.status === 'failed') actions.push(actionDescriptor('speed_limit'))
    actions.push(actionDescriptor('open_dir'), actionDescriptor('remove'))
    return uniqueActions(actions).filter((action) => action.kind !== primaryAction)
  }

  if (task.status === 'paused') {
    actions.push(
      actionDescriptor('schedule'),
      actionDescriptor('speed_limit'),
      actionDescriptor('cancel'),
      actionDescriptor('remove'),
    )
    return actions.filter((action) => action.kind !== primaryAction)
  }

  if (task.status === 'muxing') {
    return actions
  }

  if (task.status === 'waiting') actions.push(actionDescriptor('speed_limit'))
  actions.push(actionDescriptor('cancel'))
  return actions.filter((action) => action.kind !== primaryAction)
}

const actionDescriptor = (kind: Exclude<TaskActionKind, 'none'>, label = actionLabel(kind)): TaskActionDescriptor => ({
  kind,
  label,
  icon: actionIcon(kind),
  tone: kind === 'remove' || kind === 'cancel' ? 'danger' : 'normal',
})

const uniqueActions = (actions: TaskActionDescriptor[]): TaskActionDescriptor[] => {
  const seen = new Set<TaskActionKind>()
  return actions.filter((action) => {
    if (seen.has(action.kind)) {
      return false
    }

    seen.add(action.kind)
    return true
  })
}

const actionLabel = (action: TaskActionKind): string => {
  const labels: Record<TaskActionKind, string> = {
    pause: '暂停',
    resume: '继续',
    retry: '重试',
    refresh_retry: '刷新链接并重试',
    cancel: '取消',
    remove: '移除',
    open_file: '打开文件',
    open_dir: '打开文件夹',
    copy_source: '复制来源',
    unschedule: '立即开始',
    schedule: '定时开始',
    speed_limit: '设置限速',
    none: '',
  }

  return labels[action]
}

const actionIcon = (action: TaskActionKind): string => {
  const icons: Record<TaskActionKind, string> = {
    pause: 'pause',
    resume: 'play',
    retry: 'refresh',
    refresh_retry: 'refresh',
    cancel: 'x',
    remove: 'trash',
    open_file: 'file',
    open_dir: 'folder',
    copy_source: 'copy',
    unschedule: 'play',
    schedule: 'clock',
    speed_limit: 'gauge',
    none: 'more',
  }

  return icons[action]
}

const isScheduledTask = (task: DownloadTask): boolean =>
  task.status === 'waiting' && Boolean(task.scheduled_at) && Date.parse(task.scheduled_at ?? '') > Date.now()

const scheduleLabel = (scheduledAt: string | null, includeDate = false): string => {
  if (!scheduledAt) return '--'
  return new Intl.DateTimeFormat('zh-CN', {
    ...(includeDate ? { month: 'numeric', day: 'numeric' } : {}),
    hour: '2-digit',
    minute: '2-digit',
  }).format(new Date(scheduledAt))
}

const shortLocation = (path: string): string => {
  const dir = outputDir(path)
  if (dir === '--' || dir === '.') {
    return dir
  }

  const normalized = dir.replaceAll('\\', '/')
  const segments = normalized.split('/').filter(Boolean)
  return segments.at(-1) ?? dir
}

const outputDir = (path: string | null): string => {
  if (!path) {
    return '--'
  }

  const separatorIndex = Math.max(path.lastIndexOf('/'), path.lastIndexOf('\\'))
  return separatorIndex >= 0 ? path.slice(0, separatorIndex) : '.'
}

export const resourceIntentText = (intent: DownloadResourceIntent): string => {
  const labels: Record<DownloadResourceIntent, string> = {
    video: '视频',
    audio: '音频',
    cover: '封面',
    subtitle: '字幕',
    danmaku: '弹幕',
    nfo: 'NFO',
  }

  return labels[intent]
}

const humanLogTitle = (message: string): string => {
  if (message.includes('开始下载任务')) {
    return '开始下载任务'
  }
  if (message.includes('下载资源 音频')) {
    return '下载音频轨道'
  }
  if (message.includes('下载资源 视频')) {
    return '下载视频轨道'
  }
  if (message.includes('合并')) {
    return '合并音视频'
  }
  if (message.includes('重试任务')) {
    return '重试任务'
  }
  if (message.includes('刷新下载地址')) {
    return '刷新下载地址'
  }

  return '任务事件'
}

const redactUrl = (rawUrl: string): string => {
  try {
    const parsed = new URL(rawUrl)
    return parsed.search ? `${parsed.origin}${parsed.pathname}?<redacted>` : rawUrl
  } catch {
    return rawUrl
  }
}

const truncateLogMessage = (message: string): string => {
  const maxLength = 4000
  if (message.length <= maxLength) {
    return message
  }

  return `${message.slice(0, maxLength - 15)}...<truncated>`
}
