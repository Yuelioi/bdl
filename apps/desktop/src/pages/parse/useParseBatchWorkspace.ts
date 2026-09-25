import { ref } from 'vue';
import { computed } from 'vue';
import { useParseStore } from '../../stores/parse';
import { allBatchEntriesSelected, filterParseBatchEntries } from '../../stores/parseBatch';

export function useParseBatchWorkspace() {
  const parse = useParseStore();

  const query = ref('');
  const entries = computed(() => filterParseBatchEntries(parse.batchEntries, query.value));
  const selectedSet = computed(() => new Set(parse.selectedBatchEntryIds));
  const selectedCount = computed(() => parse.selectedBatchEntryIds.length);
  const allSelected = computed(() => allBatchEntriesSelected(parse.batchEntries, parse.selectedBatchEntryIds));
  const loading = computed(() => parse.sourceOrder.some((sourceId) => parse.loadingBySource[sourceId]));
  const toggleAll = () => {
    if (allSelected.value) {
      parse.clearBatchSelection();
    } else {
      parse.selectAllBatchEntries();
    }
  };
  const handleRowKeydown = (event: KeyboardEvent, entryId: string) => {
    if (event.key !== 'Enter' && event.key !== ' ') return;
    event.preventDefault();
    parse.toggleBatchEntry(entryId);
  };
  const returnToSource = () => {
    if (!loading.value) void parse.clearWorkspace();
  };
  return {
    parse,
    query,
    entries,
    selectedSet,
    selectedCount,
    allSelected,
    loading,
    toggleAll,
    handleRowKeydown,
    returnToSource,
  };
}
