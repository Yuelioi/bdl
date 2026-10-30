import { computed, shallowRef } from 'vue'
import { commandDiagnostic } from './commandDiagnostic'
import type { OperationErrorRecord } from '../api/dto'

const entries = shallowRef<string[]>([])
export const recentFailureReport = computed(() => entries.value.join('\n\n---\n\n'))
export const clearRecentFailures = () => { entries.value = [] }

// This is a view cache. The bounded backend journal owns retention and export.
export const replaceRecentFailures = (records: OperationErrorRecord[]) => {
  entries.value = records.map((record) => commandDiagnostic(record,
    `操作: ${record.command}\n错误记录: ${record.id}\n记录时环境: ${JSON.stringify(record.environment)}`, record.created_at))
}

// Never include request arguments: they may carry credentials or signed URLs.
export const recordCommandFailure = (command: string, error: unknown) => {
  entries.value = [commandDiagnostic(error, `操作: ${command}`), ...entries.value].slice(0, 20)
}
