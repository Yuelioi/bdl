import rules from '../../../../../crates/bdl-core/src/diagnostic-rules.json'
import type { QueueLogEntry } from '../api/dto'

// One catalog owns both backend classification and legacy-log presentation.
export const classifyFailure = (message: string) => {
  const lower = message.toLowerCase()
  for (const match of lower.matchAll(/code=([a-z_]+)\]/g)) {
    const rule = rules.find((rule) => rule.code === match[1])
    if (rule) return rule
  }
  return rules.find((rule) =>
    rule.contains.some((text) => lower.includes(text)) || rule.http_statuses.some((status) =>
      ['http ', 'status ', 'status code ', '状态码: ', '状态码：'].some((prefix) =>
        new RegExp(`${prefix}${status}(?![0-9])`).test(lower),
      ),
    ),
  )
}

export const latestFailure = (logs: QueueLogEntry[]): string | undefined => {
  const ordered = [...logs].sort((a, b) => b.created_at.localeCompare(a.created_at))
  for (const log of ordered) {
    if (['重试任务', '开始下载任务', '刷新下载地址'].includes(log.message)) return undefined
    if (log.level === 'error') return log.message
  }
  return undefined
}
