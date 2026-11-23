import { ref } from 'vue';
import { computed } from 'vue';
import { useParseStore } from '../../stores/parse';
import { sourceMediaByPart } from '../../utils/sourceMedia';
import { collectionTree, hasCollectionSections } from './collectionTree';
import { useSourceParsing } from './useSourceParsing';

import { flattenResultRows, sourceKindLabels, sourcePartCount, toTreeNodes } from './parseResultTree';

export function useParseResultWorkspace() {
  const parse = useParseStore();

  const activeSource = computed(() => parse.activeSource);
  const parsing = useSourceParsing(activeSource);
  const { activeLoading } = parsing;
  const selectedIds = computed(() => parse.activeSelection);
  const selectedSet = computed(() => new Set(selectedIds.value));
  const selectedCount = computed(() => selectedIds.value.length);
  const totalPartCount = computed(() => (activeSource.value ? sourcePartCount(activeSource.value) : 0));
  const query = ref('');
  const sort = ref('original');
  const media = computed(() => (activeSource.value ? sourceMediaByPart(activeSource.value) : new Map()));
  const tableRows = computed(() => {
    const rows = activeSource.value ? flattenResultRows(toTreeNodes(activeSource.value)) : [];
    const term = query.value.trim().toLocaleLowerCase();
    const groupedMatches =
      activeSource.value && hasCollectionSections(activeSource.value)
        ? new Set(collectionTree(activeSource.value, undefined, term).flatMap((node) => node.partIds))
        : null;
    const filtered = groupedMatches
      ? rows.filter((row) => row.partIds.some((id) => groupedMatches.has(id)))
      : term
        ? rows.filter((row) =>
            `${row.title} ${row.meta ?? ''} ${media.value.get(row.partIds[0])?.part.bvid ?? ''}`
              .toLocaleLowerCase()
              .includes(term),
          )
        : rows;
    if (sort.value === 'title')
      return [...filtered].sort((a, b) => a.title.localeCompare(b.title, 'zh-CN', { numeric: true }));
    if (sort.value === 'duration')
      return [...filtered].sort(
        (a, b) =>
          (media.value.get(a.partIds[0])?.duration ?? Number.MAX_SAFE_INTEGER) -
          (media.value.get(b.partIds[0])?.duration ?? Number.MAX_SAFE_INTEGER),
      );
    return filtered;
  });
  const allRowsSelected = computed(
    () =>
      tableRows.value.length > 0 &&
      tableRows.value.flatMap((row) => row.partIds).every((partId) => selectedSet.value.has(partId)),
  );
  const canCreateTasks = computed(() => Boolean(activeSource.value && selectedCount.value > 0 && !activeLoading.value));
  const toggleAllResults = () => {
    if (!activeSource.value) return;
    if (allRowsSelected.value) {
      const matching = new Set(tableRows.value.flatMap((row) => row.partIds));
      parse.selectPartIds(
        activeSource.value.source.id,
        selectedIds.value.filter((id) => !matching.has(id)),
        'replace',
      );
      return;
    }
    parse.selectPartIds(
      activeSource.value.source.id,
      tableRows.value.flatMap((row) => row.partIds),
    );
  };
  const toggleNode = (nodeId: string) => {
    if (activeSource.value) parse.toggleNode(activeSource.value.source.id, nodeId);
  };
  const selectRange = (text: string): string | null => {
    const tokens = text
      .trim()
      .split(/[,，\s]+/)
      .filter(Boolean);
    if (!tokens.length) return '请输入序号，例如 1-5,7,9-12';
    const indices = new Set<number>();
    for (const token of tokens) {
      const match = /^(\d+)(?:-(\d+))?$/.exec(token);
      if (!match) return '请使用序号或范围，例如 1-5,7,9-12';
      const start = Number(match[1]);
      const end = Number(match[2] ?? match[1]);
      if (start < 1 || end < start || end > tableRows.value.length)
        return `序号应在 1–${tableRows.value.length} 内，范围从小到大填写`;
      for (let index = start - 1; index < end; index++) indices.add(index);
    }
    if (!activeSource.value || activeLoading.value) return '当前来源正在处理，请稍后再选择';
    parse.selectPartIds(
      activeSource.value.source.id,
      [...indices].flatMap((index) => tableRows.value[index].partIds),
    );
    return null;
  };
  return {
    ...parsing,
    parse,
    activeSource,
    selectedIds,
    selectedCount,
    totalPartCount,
    tableRows,
    query,
    sort,
    selectRange,
    allRowsSelected,
    canCreateTasks,
    toggleAllResults,
    toggleNode,
    sourceKindLabels,
  };
}
