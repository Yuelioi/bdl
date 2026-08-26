<script setup lang="ts">
import { computed, ref } from 'vue'

import { useParseStore } from '../../stores/parse'
import { allBatchEntriesSelected, filterParseBatchEntries } from '../../stores/parseBatch'
import UiButton from '../../ui/Button.vue'
import UiIconButton from '../../ui/IconButton.vue'
import UiTextField from '../../ui/TextField.vue'
import SelectionActionBar from '../../ui/SelectionActionBar.vue'

const emit = defineEmits<{ download: [] }>()
const { embedded = false } = defineProps<{ embedded?: boolean }>()
const parse = useParseStore()
const query = ref('')

const entries = computed(() => filterParseBatchEntries(parse.batchEntries, query.value))
const selectedSet = computed(() => new Set(parse.selectedBatchEntryIds))
const selectedCount = computed(() => parse.selectedBatchEntryIds.length)
const allSelected = computed(() => allBatchEntriesSelected(parse.batchEntries, parse.selectedBatchEntryIds))
const loading = computed(() => parse.sourceOrder.some((sourceId) => parse.loadingBySource[sourceId]))

const toggleAll = () => {
  if (allSelected.value) {
    parse.clearBatchSelection()
  } else {
    parse.selectAllBatchEntries()
  }
}

const handleRowKeydown = (event: KeyboardEvent, entryId: string) => {
  if (event.key !== 'Enter' && event.key !== ' ') return
  event.preventDefault()
  parse.toggleBatchEntry(entryId)
}

const returnToSource = () => {
  if (!loading.value) void parse.clearWorkspace()
}
</script>

<template>
  <section class="min-h-0 overflow-hidden" :class="embedded ? 'flex flex-1 flex-col gap-3' : 'panel'">
    <header
      class="batch-result-header grid min-w-0 grid-cols-[minmax(0,1fr)_auto] gap-4 border-b border-(--color-border) pb-3"
    >
      <div class="flex min-w-0 items-center gap-3">
        <button
          type="button"
          class="grid size-8 shrink-0 place-items-center rounded-md text-(--color-muted) hover:bg-(--color-panel) hover:text-(--color-text) disabled:cursor-not-allowed disabled:opacity-55"
          aria-label="返回解析首页"
          :disabled="loading"
          @click="returnToSource"
        >
          <UIcon name="i-tabler-arrow-left" class="size-4" aria-hidden="true" />
        </button>
        <div class="grid min-w-0 gap-1">
          <div class="flex items-center gap-2">
            <strong class="text-base text-(--color-text)">批量视频</strong>
            <span class="text-xs font-bold text-(--color-muted)">{{ parse.batchEntries.length }} 个链接</span>
          </div>
          <p class="m-0 text-xs leading-5 text-(--color-muted)">每个链接作为一个视频条目，下载设置会统一应用。</p>
        </div>
      </div>
      <div class="source-function-toolbar flex items-start justify-end">
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

    <div class="grid min-h-0 flex-1 content-start gap-2 overflow-y-auto pr-0.5" role="list" aria-label="批量解析结果">
      <div
        v-for="entry in entries"
        :key="entry.id"
        class="grid min-h-18 cursor-pointer grid-cols-[28px_minmax(0,1fr)_auto] items-center gap-3 rounded-lg border px-3 py-2.5 outline-none"
        :class="
          selectedSet.has(entry.id)
            ? 'border-(--color-accent) bg-(--color-accent-soft)'
            : 'border-(--color-border) bg-(--color-surface) hover:bg-(--color-panel)'
        "
        role="checkbox"
        :aria-checked="selectedSet.has(entry.id)"
        tabindex="0"
        @click="parse.toggleBatchEntry(entry.id)"
        @keydown="handleRowKeydown($event, entry.id)"
      >
        <span
          class="grid size-7 place-items-center rounded-full border text-(--color-muted)"
          :class="
            selectedSet.has(entry.id)
              ? 'border-(--color-accent) bg-(--color-accent) text-(--color-on-accent)'
              : 'border-(--color-border) bg-(--color-surface)'
          "
          aria-hidden="true"
        >
          <UIcon class="size-4" :name="selectedSet.has(entry.id) ? 'i-tabler-check' : 'i-tabler-plus'" />
        </span>
        <span class="grid min-w-0 gap-1">
          <strong class="truncate text-[13px] text-(--color-text)" :title="entry.title">{{ entry.title }}</strong>
          <span class="truncate text-xs text-(--color-muted)" :title="entry.input">{{ entry.input }}</span>
          <span class="text-xs text-(--color-muted)">视频</span>
        </span>
        <UiIconButton
          icon="x"
          label="移除这个链接"
          variant="ghost"
          size="compact"
          :disabled="loading"
          @click.stop="parse.removeBatchEntry(entry.id)"
        />
      </div>

      <p v-if="entries.length === 0" class="m-0 py-12 text-center text-sm text-(--color-muted)">没有匹配的视频</p>
    </div>

    <SelectionActionBar :selected-count="selectedCount" :total-count="parse.batchEntries.length" unit="个视频">
      <template #selection>
        <UiButton size="compact" variant="ghost" :disabled="loading" @click="toggleAll">
          {{ allSelected ? '取消全选' : '全选全部' }}
        </UiButton>
        <UiButton
          v-if="selectedCount > 0 && !allSelected"
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
