import { computed, ref, type Ref } from 'vue';
import type { NormalizedSourceTree } from '../../api/dto';
import { useParseStore } from '../../stores/parse';

export function useSourceParsing(source: Readonly<Ref<NormalizedSourceTree | null | undefined>>) {
  const parse = useParseStore();
  const sourceId = computed(() => source.value?.source.id ?? null);
  const loadBatchSize = ref('50');
  const hasMore = computed(() => Boolean(source.value?.source.has_more));
  const sourceRequestLoading = computed(() => Boolean(sourceId.value && parse.loadingBySource[sourceId.value]));
  const pacedParsing = computed(() => Boolean(sourceId.value && parse.pacedParsingBySource[sourceId.value]));
  const pacedStopping = computed(() => Boolean(sourceId.value && parse.pacedParsingStopRequestedBySource[sourceId.value]));
  const activeLoading = computed(() => sourceRequestLoading.value || pacedParsing.value);
  const activeError = computed(() => (sourceId.value ? parse.errorsBySource[sourceId.value] : null));
  const activeErrorDetail = computed(() => (sourceId.value ? parse.errorDetailsBySource[sourceId.value] : null));

  const loadMore = async () => {
    if (sourceId.value && hasMore.value) await parse.loadChunk(sourceId.value, Number(loadBatchSize.value));
  };
  const parseAll = async () => {
    if (sourceId.value && hasMore.value) await parse.parseAllPaced(sourceId.value, Number(loadBatchSize.value));
  };
  const stopParsing = () => {
    if (sourceId.value) parse.stopPacedParsing(sourceId.value);
  };

  return {
    sourceId,
    loadBatchSize,
    hasMore,
    sourceRequestLoading,
    pacedParsing,
    pacedStopping,
    activeLoading,
    activeError,
    activeErrorDetail,
    loadMore,
    parseAll,
    stopParsing,
  };
}
