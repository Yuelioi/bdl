import { ref } from 'vue';
import { computed } from 'vue';
import { useParseStore } from '../../stores/parse';

import { flattenResultRows, sourceKindLabels, sourcePartCount, toTreeNodes } from './parseResultTree';

export function useParseResultWorkspace(onDownload: () => void) {
  const parse = useParseStore();

  const loadBatchSize = ref('50');
  const activeSource = computed(() => parse.activeSource);
  const selectedIds = computed(() => parse.activeSelection);
  const selectedCount = computed(() => selectedIds.value.length);
  const totalPartCount = computed(() => (activeSource.value ? sourcePartCount(activeSource.value) : 0));
  const tableRows = computed(() => (activeSource.value ? flattenResultRows(toTreeNodes(activeSource.value)) : []));
  const sourceRequestLoading = computed(() =>
    Boolean(activeSource.value && parse.loadingBySource[activeSource.value.source.id]),
  );
  const pacedParsing = computed(() =>
    Boolean(activeSource.value && parse.pacedParsingBySource[activeSource.value.source.id]),
  );
  const pacedWaiting = computed(() =>
    Boolean(activeSource.value && parse.pacedParsingWaitingBySource[activeSource.value.source.id]),
  );
  const pacedStopping = computed(() =>
    Boolean(activeSource.value && parse.pacedParsingStopRequestedBySource[activeSource.value.source.id]),
  );
  const activeLoading = computed(() => sourceRequestLoading.value || pacedParsing.value);
  const activeError = computed(() => (activeSource.value ? parse.errorsBySource[activeSource.value.source.id] : null));
  const hasMore = computed(() => Boolean(activeSource.value?.source.has_more));
  const allRowsSelected = computed(
    () =>
      tableRows.value.length > 0 &&
      tableRows.value.flatMap((row) => row.partIds).every((partId) => selectedIds.value.includes(partId)),
  );
  const canCreateTasks = computed(() => Boolean(activeSource.value && selectedCount.value > 0 && !activeLoading.value));
  const createTaskLabel = computed(() => `下载所选 (${selectedCount.value})`);
  const toggleAllResults = () => {
    if (!activeSource.value) return;
    if (allRowsSelected.value) {
      parse.clearSelection(activeSource.value.source.id);
      return;
    }
    parse.selectPartIds(
      activeSource.value.source.id,
      tableRows.value.flatMap((row) => row.partIds),
    );
  };
  const clearSelection = () => {
    if (activeSource.value) parse.clearSelection(activeSource.value.source.id);
  };
  const loadMore = () => {
    if (activeSource.value?.source.has_more) {
      void parse.loadChunk(activeSource.value.source.id, Number(loadBatchSize.value));
    }
  };
  const parseAll = () => {
    if (activeSource.value?.source.has_more) {
      void parse.parseAllPaced(activeSource.value.source.id, Number(loadBatchSize.value));
    }
  };
  const stopParsing = () => {
    if (activeSource.value) parse.stopPacedParsing(activeSource.value.source.id);
  };
  const downloadAllLoaded = () => {
    if (!activeSource.value) return;
    parse.selectAllLoaded(activeSource.value.source.id);
    onDownload();
  };
  const parseAndDownload = () => {
    if (activeSource.value) void parse.startBackgroundDownload(activeSource.value.source.id);
  };
  const returnToSource = () => {
    if (!activeLoading.value) void parse.clearWorkspace();
  };
  const toggleNode = (nodeId: string) => {
    if (activeSource.value) parse.toggleNode(activeSource.value.source.id, nodeId);
  };
  return {
    parse,
    loadBatchSize,
    activeSource,
    selectedIds,
    selectedCount,
    totalPartCount,
    tableRows,
    sourceRequestLoading,
    pacedParsing,
    pacedWaiting,
    pacedStopping,
    activeLoading,
    activeError,
    hasMore,
    allRowsSelected,
    canCreateTasks,
    createTaskLabel,
    toggleAllResults,
    clearSelection,
    loadMore,
    parseAll,
    stopParsing,
    downloadAllLoaded,
    parseAndDownload,
    returnToSource,
    toggleNode,
    sourceKindLabels,
  };
}
