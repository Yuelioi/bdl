import type { WorkflowStep } from '../ui/WorkflowSteps.vue';
import { computed, ref, useTemplateRef, watch } from 'vue';
import { useParseStore } from '../stores/parse';
import { useUiStore } from '../stores/ui';
import { extractBilibiliInputs } from '../utils/bilibiliLinks';
import { readClipboardText } from '../utils/clipboard';

export function useParsePage() {
  const parse = useParseStore();

  const inputMode = ref<'batch' | 'single'>('batch');
  const singleInput = ref('');
  const ui = useUiStore();
  const downloadPlanner = useTemplateRef<{ openDialog: () => Promise<void> }>('download-planner');
  const createLoading = computed(() => Boolean(parse.loadingBySource.__create__));
  const hasResults = computed(() => Boolean(parse.activeSource));
  const activeStage = ref<'source' | 'content'>(hasResults.value ? 'content' : 'source');
  const workflowSteps = computed<WorkflowStep[]>(() => [
    { value: 'source', label: '解析来源', description: '输入链接', complete: hasResults.value },
    {
      value: 'content',
      label: '选择内容',
      description: hasResults.value ? '筛选并下载' : '等待解析',
      disabled: !hasResults.value,
    },
  ]);
  const submitInput = async () => {
    if (createLoading.value) return;

    const parsed =
      inputMode.value === 'single'
        ? await parse.createSource(singleInput.value, { singleVideo: true })
        : await parse.createSource();
    if (parsed) activeStage.value = 'content';
  };
  const setSingleInput = (text: string) => {
    const extracted = extractBilibiliInputs(text);
    if (extracted.length > 1 || (extracted.length === 0 && text.trim().includes('\n'))) {
      parse.setNotice('单个视频模式一次只解析一个链接，请只粘贴一个视频或切换到批量解析。', 'warning');
      return;
    }
    singleInput.value = extracted[0] ?? text.trim();
    parse.clearNotice();
  };
  const switchInputMode = (mode: 'batch' | 'single') => {
    inputMode.value = mode;
    parse.clearNotice();
  };
  const pasteSingleInput = (event: ClipboardEvent) => {
    setSingleInput(event.clipboardData?.getData('text/plain') ?? '');
  };
  const pasteInput = async () => {
    try {
      const text = (await readClipboardText()).trim();
      if (!text) {
        parse.setNotice('剪贴板里没有可粘贴的链接', 'warning');
        return;
      }

      if (inputMode.value === 'single') {
        setSingleInput(text);
      } else {
        const extracted = extractBilibiliInputs(text);
        parse.input = extracted.length > 0 ? extracted.join('\n') : text;
        parse.clearNotice();
      }
    } catch (error) {
      parse.setNotice(`读取剪贴板失败：${error instanceof Error ? error.message : String(error)}`, 'danger');
    }
  };
  const openDownloadSettings = () => {
    void downloadPlanner.value?.openDialog();
  };
  watch(hasResults, (available) => {
    activeStage.value = available ? 'content' : 'source';
  });
  const runNoticeAction = () => {
    if (parse.notice?.actionLabel !== '查看传输') return;
    ui.setTab('transfer');
    parse.clearNotice();
  };
  return {
    parse,
    inputMode,
    singleInput,
    createLoading,
    activeStage,
    workflowSteps,
    submitInput,
    switchInputMode,
    pasteSingleInput,
    pasteInput,
    openDownloadSettings,
    runNoticeAction,
  };
}
