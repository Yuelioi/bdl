import { problemMessage, type PublicError } from './publicError'
import { redactDiagnosticMessage } from '../stores/transferView'

export const boundedDiagnostic = (text: string, limit = 65536): string => {
  const characters = Array.from(redactDiagnosticMessage(text))
  return characters.length <= limit ? characters.join('')
    : `${characters.slice(0, limit - 20).join('')}...<truncated>`
}

export const commandDiagnostic = (error: unknown, context: string, recordedAt = new Date().toISOString()): string => {
  const value = error && typeof error === 'object' ? error as Record<string, unknown> : {}
  const problem = value.problem as PublicError | undefined
  return redactDiagnosticMessage([
    `时间: ${recordedAt}`,
    context,
    problem ? `错误码: ${problem.code}\n追踪编号: ${problem.traceId}` : typeof value.code === 'string' ? `错误码: ${value.code}` : '',
    typeof value.message === 'string' ? problemMessage({ problem, message: value.message }) : String(error),
    typeof value.detail === 'string' ? value.detail : '',
  ].filter(Boolean).join('\n'))
}
