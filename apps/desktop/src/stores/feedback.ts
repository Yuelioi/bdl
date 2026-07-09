export type NoticeTone = 'info' | 'success' | 'warning' | 'danger'

export interface InlineNotice {
  message: string
  tone: NoticeTone
  actionLabel?: string
}

export const NOTICE_CLEAR_DELAY = 3000
