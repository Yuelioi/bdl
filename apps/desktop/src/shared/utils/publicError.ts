import catalogData from '../../../../../crates/bdl-core/src/error-catalog.json'

export interface PublicError {
  code: string
  traceId: string
  params?: { retryAfterSeconds?: number; arguments?: string[] }
}

type Language = 'zh-CN' | 'en'
type Text = Record<Language, string>
const catalog: Readonly<Record<string, { message?: Text; advice?: Text }>> = catalogData

export function describeProblem(problem: PublicError, language: Language = 'zh-CN') {
  let message = catalog[problem.code]?.message?.[language]
  // A newer producer can supply a leaf absent from this version of the catalog.
  message ??= language === 'en' ? 'The operation failed.' : '操作失败。'
  if (problem.params?.retryAfterSeconds != null) {
    message = message.replace('{retryAfterSeconds}', String(problem.params.retryAfterSeconds))
  }
  if (problem.params?.arguments) message = message.replace('{arguments}', problem.params.arguments.join(', '))
  let key = problem.code
  while (key) {
    const advice = catalog[key]?.advice?.[language]
    if (advice) return { message, advice, matchedCode: key }
    const dot = key.lastIndexOf('.')
    if (dot < 0) break
    key = key.slice(0, dot)
  }
}

export function problemMessage(error: { problem?: PublicError | null; message: string }) {
  const text = error.problem && describeProblem(error.problem)
  return text ? `${text.message}${text.advice}` : error.message
}
