import { ref, watch } from 'vue';
import type { RemovalPreview } from '../../api/dto';
import { redactDiagnosticMessage } from '../../stores/transferView';
import { BdlCommandError } from '../../api/tauri';
import { captureDiagnosticEnvironment } from '../../utils/diagnosticEnvironment';

export function useFileRemoval(
  kind: 'tasks' | 'temp',
  previewFiles: (ids: string[]) => Promise<RemovalPreview>,
  execute: (token: string) => Promise<unknown>,
) {
  const open = ref(false);
  const busy = ref(false);
  const executing = ref(false);
  const preview = ref<RemovalPreview | null>(null);
  const error = ref('');
  const errorDetails = ref('');
  const copied = ref(false);
  const copyNotice = ref('');
  const canConfirm = ref(false);
  let ids: string[] = [];
  let generation = 0;
  const setFailure = (failure: unknown) => {
    error.value = redactDiagnosticMessage(failure instanceof Error ? failure.message : String(failure));
    errorDetails.value = failure instanceof BdlCommandError
      ? redactDiagnosticMessage(`code=${failure.code}\n${failure.detail ?? ''}`) : '';
  };
  watch(open, (value) => { if (!value) { generation++; if (!executing.value) busy.value = false; } });
  const refresh = async () => {
    const current = ++generation;
    busy.value = true;
    error.value = '';
    errorDetails.value = '';
    copied.value = false;
    copyNotice.value = '';
    canConfirm.value = false;
    try {
      const result = await previewFiles(ids);
      if (current === generation && open.value) {
        preview.value = result;
        canConfirm.value = true;
      }
    } catch (failure) {
      if (current === generation) setFailure(failure);
    } finally {
      if (current === generation) busy.value = false;
    }
  };
  const show = (taskIds: string[] = []) => {
    if (busy.value || (kind === 'tasks' && taskIds.length === 0)) return;
    ids = [...new Set(taskIds)];
    preview.value = null;
    open.value = true;
    void refresh();
  };
  const confirm = async () => {
    if (!preview.value || !canConfirm.value || busy.value) return;
    busy.value = true;
    canConfirm.value = false;
    executing.value = true;
    error.value = '';
    errorDetails.value = '';
    copied.value = false;
    copyNotice.value = '';
    try {
      await execute(preview.value.token);
      open.value = false;
    } catch (failure) {
      setFailure(failure);
    } finally { busy.value = false; executing.value = false; }
  };
  const copyError = async () => {
    if (!error.value) return;
    copyNotice.value = '';
    try {
      const environment = await captureDiagnosticEnvironment();
      await writeClipboardText(`${environment}\n\n${kind === 'tasks' ? '删除任务及文件' : '清理临时文件'}\n任务：${ids.join(', ')}\n${error.value}\n${errorDetails.value}`);
      copied.value = true;
    } catch { copied.value = false; copyNotice.value = '复制失败，请重试'; }
  };
  return { kind, open, busy, executing, preview, error, errorDetails, copied, copyNotice, copyError, canConfirm, show, refresh, confirm };
}
export type FileRemovalController = ReturnType<typeof useFileRemoval>;
import { writeClipboardText } from '../../utils/clipboard';
