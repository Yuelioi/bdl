export type NoticeTone = 'info' | 'success' | 'warning' | 'danger'

export interface InlineNotice {
  message: string
  tone: NoticeTone
  actionLabel?: string
}

export const FEEDBACK_AUTO_DISMISS_MS = 5_000
export const NOTICE_CLEAR_DELAY = FEEDBACK_AUTO_DISMISS_MS
