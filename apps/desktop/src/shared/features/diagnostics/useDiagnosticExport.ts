import { onScopeDispose, ref } from 'vue';

import { diagnosticsExport, diagnosticsOpenDirectory } from '../../api/tauri';
import { isAndroidPlatform, isMobilePlatform } from '../../platform/environment';
import { useUiStore } from '../../stores/ui';

export const useDiagnosticExport = (taskId: () => string | undefined) => {
  const ui = useUiStore();
  const busy = ref(false);
  const folderReady = ref(false);
  const notice = ref('');
  const noticeTone = ref<'success' | 'info'>('success');
  let folderTimer: ReturnType<typeof setTimeout> | undefined;
  let noticeTimer: ReturnType<typeof setTimeout> | undefined;
  let revision = 0;

  const reset = () => {
    revision += 1;
    clearTimeout(folderTimer);
    clearTimeout(noticeTimer);
    busy.value = false;
    folderReady.value = false;
    notice.value = '';
  };
  const run = async () => {
    if (busy.value) return;
    const requestRevision = revision;
    busy.value = true;
    try {
      if (folderReady.value) {
        await diagnosticsOpenDirectory();
        return;
      }
      const result = await diagnosticsExport(taskId());
      if (requestRevision !== revision) return;
      notice.value = result.saved
        ? isAndroidPlatform() ? '诊断已保存到所选位置。' : `诊断已导出：${result.path}`
        : '已取消保存诊断。';
      noticeTone.value = result.saved ? 'success' : 'info';
      clearTimeout(noticeTimer);
      noticeTimer = setTimeout(() => { notice.value = ''; }, 3000);
      if (result.saved && !isMobilePlatform()) {
        folderReady.value = true;
        clearTimeout(folderTimer);
        folderTimer = setTimeout(() => { folderReady.value = false; }, 5000);
      }
    } catch (error) {
      ui.pushToast(error instanceof Error ? error.message : String(error), 'danger');
    } finally {
      if (requestRevision === revision) busy.value = false;
    }
  };
  onScopeDispose(reset);
  return { busy, folderReady, notice, noticeTone, run, reset };
};
