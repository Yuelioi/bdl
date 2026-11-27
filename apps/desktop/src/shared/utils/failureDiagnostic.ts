import rules from '../../../../../crates/bdl-core/src/diagnostic-rules.json'
import type { QueueLogEntry } from '../api/dto'

export type FailureRule = typeof rules[number]

export const failureRule = (code: string) => rules.find((rule) => rule.code === code)

// One catalog owns both backend classification and legacy-log presentation.
export const classifyFailure = (message: string) => {
  const lower = message.toLowerCase()
  for (const match of lower.matchAll(/code=([a-z_]+)\]/g)) {
    const rule = failureRule(match[1])
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

export const failureGuidance = (rule: FailureRule, stage: 'parse' | 'download', mobile: boolean) => {
  if (rule.recommended_action === 'login_then_retry') return { label: '登录 B 站', section: null, tip: '' } as const
  if (rule.recommended_action === 'configure_ffmpeg' && !mobile) {
    return { label: '配置 FFmpeg', section: 'settings-download', tip: '' } as const
  }
  if (['source_rate_limited', 'source_verification_required', 'source_access_restricted'].includes(rule.code)) {
    return stage === 'parse'
      ? { label: '调整解析节奏', section: 'settings-links', tip: '下次解析时可延长翻页间隔。' } as const
      : { label: '调整下载设置', section: 'settings-tasks', tip: '下次可减少同时下载任务数和单任务分段数，延长任务间隔。下载总限速只控制带宽。' } as const
  }
}

export const latestFailureEntry = (logs: QueueLogEntry[]): QueueLogEntry | undefined => {
  const ordered = [...logs].sort((a, b) => b.created_at.localeCompare(a.created_at))
  for (const log of ordered) {
    if (['重试任务', '开始下载任务', '刷新下载地址'].includes(log.message)) return undefined
    if (log.level === 'error') return log
  }
  return undefined
}


export const ruleForCode = (code: string) => rules.find(rule => rule.public_code === code)

export const ruleForEntry = (entry?: QueueLogEntry) => entry?.problem
  ? ruleForCode(entry.problem.code)
  : classifyFailure(entry?.message ?? '')
