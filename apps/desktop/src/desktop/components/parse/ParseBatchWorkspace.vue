<script setup lang="ts">
import { computed } from 'vue';
import SourceMediaList from './SourceMediaList.vue';
import type { NormalizedSourceTree } from '../../../shared/api/dto';
import UiButton from '../../../shared/ui/Button.vue';
import UiIconButton from '../../../shared/ui/IconButton.vue';
import UiTextField from '../../../shared/ui/TextField.vue';
import SelectionActionBar from '../../../shared/ui/SelectionActionBar.vue';
import { useParseBatchWorkspace } from '../../../shared/features/parse/useParseBatchWorkspace';
const { embedded = false, inlineDownload = false } = defineProps<{ embedded?: boolean; inlineDownload?: boolean }>();
const emit = defineEmits<{ download: [] }>();
const { parse, query, entries, selectedCount, loading, toggleAll, returnToSource } = useParseBatchWorkspace();
const rows = computed(() =>
  entries.value.map((entry) => ({ id: entry.id, title: entry.title, meta: entry.input, partIds: [entry.partId] })),
);
const source = computed<NormalizedSourceTree>(() => ({
  source: {
    id: 'batch',
    kind: 'video',
    input: '',
    title: '批量视频',
    loaded_count: entries.value.length,
    total_count: entries.value.length,
    has_more: false,
  },
  groups: Object.values(parse.sources).flatMap((tree) =>
    tree.groups.map((group) => ({
      ...group,
      items: group.items.map((item) => ({ ...item, owner_name: item.owner_name || tree.source.title })),
    })),
  ),
}));
</script>
<template>
  <section class="min-h-0 overflow-hidden" :class="embedded ? 'flex flex-1 flex-col gap-3' : 'panel'">
    <header
      class="batch-result-header grid min-w-0 grid-cols-[minmax(0,1fr)_auto] gap-4 border-b border-(--color-border) pb-3"
    >
      <div class="flex min-w-0 items-center gap-3">
        <UButton
          variant="ghost"
          color="neutral"
          :ui="{ base: 'p-0', label: 'contents' }"
          type="button"
          class="grid size-8 shrink-0 place-items-center rounded-md text-(--color-muted) hover:bg-(--color-panel) hover:text-(--color-text) disabled:cursor-not-allowed disabled:opacity-55"
          aria-label="返回解析首页"
          :disabled="loading"
          @click="returnToSource"
        >
          <UIcon name="i-tabler-arrow-left" class="size-4" aria-hidden="true" />
        </UButton>
        <div class="grid min-w-0 gap-1">
          <div class="flex items-center gap-2">
            <strong class="text-base text-(--color-text)">批量视频</strong>
            <span class="text-xs font-bold text-(--color-muted)">{{ parse.batchEntries.length }} 个链接</span>
          </div>
        </div>
      </div>
      <div v-if="!inlineDownload" class="source-function-toolbar flex items-start justify-end">
        <UiButton size="compact" :disabled="selectedCount === 0 || loading" @click="emit('download')">
          下载所选 ({{ selectedCount }})
        </UiButton>
      </div>
      <UiTextField
        v-model="query"
        class="col-span-full"
        label="搜索视频"
        placeholder="标题或原始链接"
        :disabled="loading"
      />
    </header>

    <SourceMediaList
      compact
      class="min-h-0 flex-1"
      label="批量解析结果"
      selection-label="全选全部"
      :source="source"
      :rows="rows"
      :selected-ids="[]"
      :selected-row-ids="parse.selectedBatchEntryIds"
      :disabled="loading"
      @toggle="parse.toggleBatchEntry"
      @toggle-all="toggleAll"
    >
      <template #actions="{ row }">
        <UDropdownMenu
          :items="[{ label: '移除这个链接', icon: 'i-tabler-trash', onSelect: () => parse.removeBatchEntry(row.id) }]"
          :disabled="loading"
        >
          <UiIconButton icon="more" label="更多操作" variant="ghost" size="compact" :disabled="loading" />
        </UDropdownMenu>
      </template>
    </SourceMediaList>

    <SelectionActionBar
      v-if="!inlineDownload"
      :selected-count="selectedCount"
      :total-count="parse.batchEntries.length"
      unit="个视频"
    >
      <template #selection>
        <UiButton
          v-if="selectedCount > 0"
          size="compact"
          variant="ghost"
          :disabled="loading"
          @click="parse.clearBatchSelection()"
        >
          取消选择
        </UiButton>
      </template>
    </SelectionActionBar>
  </section>
</template>
