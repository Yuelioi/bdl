import { computed, ref, watch } from 'vue';
import type { AccountLibraryFolder, NormalizedItem } from '../../api/dto';
import { useParseStore } from '../../stores/parse';
import { bilibiliVideoUrl } from '../../utils/bilibiliLinks';

export function useLibraryFolderDetail(
  props: { folder: AccountLibraryFolder; loadingInitial?: boolean },
  onDownload: () => void,
) {
  const parse = useParseStore();
  const pageSize = 20;
  const currentPage = ref(1);
  const pageItemIds = ref<Record<number, string[]>>({});
  const failedCoverIds = ref<string[]>([]);
  const loadBatchSize = ref('50');
  const source = computed(() => {
    const activeSource = parse.activeSource;
    return !props.loadingInitial && activeSource?.source.input === props.folder.source_url ? activeSource : null;
  });
  const sourceId = computed(() => source.value?.source.id ?? null);
  const items = computed(() => source.value?.groups.flatMap((group) => group.items) ?? []);
  const totalCount = computed(() => source.value?.source.total_count ?? props.folder.media_count);
  const totalPages = computed(() => Math.max(1, Math.ceil(totalCount.value / pageSize)));
  const pageItems = computed(() => {
    const byId = new Map(items.value.map((item) => [item.id, item]));
    return (pageItemIds.value[currentPage.value] ?? []).flatMap((id) => {
      const item = byId.get(id);
      return item ? [item] : [];
    });
  });
  const selectedSet = computed(() => new Set(parse.activeSelection));
  const selectedCount = computed(() => parse.activeSelection.length);
  const sourceRequestLoading = computed(() => Boolean(sourceId.value && parse.loadingBySource[sourceId.value]));
  const pacedParsing = computed(() => Boolean(sourceId.value && parse.pacedParsingBySource[sourceId.value]));
  const pacedWaiting = computed(() => Boolean(sourceId.value && parse.pacedParsingWaitingBySource[sourceId.value]));
  const pacedStopping = computed(() =>
    Boolean(sourceId.value && parse.pacedParsingStopRequestedBySource[sourceId.value]),
  );
  const loading = computed(() => props.loadingInitial || sourceRequestLoading.value || pacedParsing.value);
  const activeError = computed(() => (sourceId.value ? parse.errorsBySource[sourceId.value] : null));
  const hasMore = computed(() => Boolean(source.value?.source.has_more));
  watch(
    sourceId,
    () => {
      currentPage.value = 1;
      pageItemIds.value = sourceId.value ? { 1: items.value.slice(0, pageSize).map((item) => item.id) } : {};
      failedCoverIds.value = [];
    },
    { immediate: true },
  );
  const itemPartIds = (item: NormalizedItem): string[] => item.parts.map((part) => part.id);
  const itemOwnerName = (item: NormalizedItem): string =>
    item.owner_name?.trim() || props.folder.owner_name?.trim() || '未知 UP 主';
  const itemSelected = (item: NormalizedItem): boolean => {
    const partIds = itemPartIds(item);
    return partIds.length > 0 && partIds.every((partId) => selectedSet.value.has(partId));
  };
  const currentPageSelected = computed(
    () => pageItems.value.length > 0 && pageItems.value.every((item) => itemSelected(item)),
  );
  const toggleItem = (item: NormalizedItem) => {
    if (loading.value) return;
    if (sourceId.value) parse.toggleNode(sourceId.value, item.id);
  };
  const toggleCurrentPageSelection = () => {
    if (!sourceId.value) return;
    if (currentPageSelected.value) {
      pageItems.value.forEach((item) => parse.toggleNode(sourceId.value!, item.id));
      return;
    }
    parse.selectPartIds(sourceId.value, pageItems.value.flatMap(itemPartIds));
  };
  const clearSelection = () => {
    if (sourceId.value) parse.clearSelection(sourceId.value);
  };
  const downloadItem = (item: NormalizedItem) => {
    if (!sourceId.value) return;
    parse.selectPartIds(sourceId.value, itemPartIds(item), 'replace');
    onDownload();
  };
  const downloadSelected = () => {
    if (selectedCount.value > 0) onDownload();
  };
  const parseMore = async () => {
    if (sourceId.value && hasMore.value) await parse.loadChunk(sourceId.value, Number(loadBatchSize.value));
  };
  const parseAll = async () => {
    if (sourceId.value && hasMore.value) {
      await parse.parseAllPaced(sourceId.value, Number(loadBatchSize.value));
    }
  };
  const stopParsing = () => {
    if (sourceId.value) parse.stopPacedParsing(sourceId.value);
  };
  const downloadAll = async () => {
    if (!sourceId.value) return;
    if (hasMore.value) {
      const result = await parse.parseAllPaced(sourceId.value, Number(loadBatchSize.value));
      if (result !== 'completed') return;
    }
    parse.selectAllLoaded(sourceId.value);
    onDownload();
  };
  const goToPage = async (targetPage: number) => {
    if (!sourceId.value || loading.value) return;
    const requestedSource = sourceId.value;
    const nextPage = Math.min(Math.max(1, targetPage), totalPages.value);
    if (!pageItemIds.value[nextPage]) {
      const ids = await parse.loadPage(requestedSource, nextPage);
      if (ids === null || sourceId.value !== requestedSource) return;
      pageItemIds.value[nextPage] = ids;
    }
    currentPage.value = nextPage;
  };
  const formatDuration = (seconds: number | null): string => {
    if (!seconds || seconds <= 0) return '--:--';
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const remaining = Math.floor(seconds % 60);
    return hours > 0
      ? `${hours}:${String(minutes).padStart(2, '0')}:${String(remaining).padStart(2, '0')}`
      : `${minutes}:${String(remaining).padStart(2, '0')}`;
  };
  return {
    parse,
    pageSize,
    currentPage,
    failedCoverIds,
    loadBatchSize,
    source,
    sourceId,
    items,
    totalCount,
    totalPages,
    pageItems,
    selectedCount,
    sourceRequestLoading,
    pacedParsing,
    pacedWaiting,
    pacedStopping,
    loading,
    activeError,
    hasMore,
    itemOwnerName,
    itemSelected,
    currentPageSelected,
    toggleItem,
    toggleCurrentPageSelection,
    clearSelection,
    downloadItem,
    downloadSelected,
    parseMore,
    parseAll,
    stopParsing,
    downloadAll,
    goToPage,
    formatDuration,
    bilibiliVideoUrl,
  };
}
