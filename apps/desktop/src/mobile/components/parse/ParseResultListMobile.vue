<script setup lang="ts">
import MediaArtwork from '../MediaArtwork.vue';
import MobileListCard from '../MobileListCard.vue';
import MobileSheet from '../MobileSheet.vue';
import UiButton from '../../../shared/ui/Button.vue';
import ExternalLinkButton from '../../../shared/ui/ExternalLinkButton.vue';
import { bilibiliVideoUrl } from '../../../shared/utils/bilibiliLinks';
import { displayPartDuration, formatDuration } from '../../../shared/utils/duration';
import { computed, ref } from 'vue';
import type { NormalizedSourceTree } from '../../../shared/api/dto';
import UiCheckbox from '../../../shared/ui/Checkbox.vue';

import type { ParseResultRow } from '../../../shared/features/parse/parseResultTree';

const props = defineProps<{
  rows: ParseResultRow[];
  source: NormalizedSourceTree;
  selectedIds: string[];
  disabled?: boolean;
  managing?: boolean;
}>();
const emit = defineEmits<{ toggle: [id: string] }>();
const menuEntry = ref<{ id: string; title: string; url?: string | null; selected: boolean } | null>(null);
const menuOpen = ref(false);
const selected = computed(() => new Set(props.selectedIds));
const entries = computed(() => {
  const metadata = new Map(
    props.source.groups.flatMap((group) =>
      group.items.flatMap((item) =>
        item.parts.map(
          (part, index) =>
            [
              part.id,
              {
                url: bilibiliVideoUrl(item),
                cover: item.cover_url,
                duration: displayPartDuration(part.duration_seconds, item.duration_seconds, item.parts.length),
                index: item.parts.length > 1 ? `P${index + 1}` : '',
              },
            ] as const,
        ),
      ),
    ),
  );
  return props.rows.map((row) => ({
    ...row,
    ...metadata.get(row.partIds[0] ?? ''),
    selected: row.partIds.length > 0 && row.partIds.every((id) => selected.value.has(id)),
  }));
});
function toggleCard(event: MouseEvent, id?: string) {
  if (
    props.disabled ||
    (event.target instanceof Element && event.target.closest('button, input, a, label, [role="checkbox"]'))
  )
    return;
  if (id) emit('toggle', id);
}
</script>

<template>
  <MobileListCard class="mobile-results" role="list" aria-label="解析结果">
    <div
      v-for="entry in entries"
      :key="entry.id"
      class="mobile-result"
      :class="{ selected: entry.selected }"
      role="listitem"
      @click="toggleCard($event, entry.id)"
    >
      <UiCheckbox
        v-if="managing"
        :model-value="entry.selected"
        :label="`选择 ${entry.title}`"
        :disabled="disabled"
        compact
        @update:model-value="emit('toggle', entry.id)"
      />
      <UButton
        variant="ghost"
        color="neutral"
        :ui="{ base: 'p-0', label: 'contents' }"
        class="mobile-result-content"
        type="button"
        :disabled="disabled"
        :aria-label="`${entry.selected ? '取消选择' : '选择'} ${entry.title}`"
        :aria-pressed="entry.selected"
        @click="emit('toggle', entry.id)"
      >
        <MediaArtwork class="result-cover" :src="entry.cover" />
        <span v-if="entry.index" class="result-index">{{ entry.index }}</span>
        <span class="result-copy">
          <strong>{{ entry.title }}</strong>
          <span v-if="!entry.index && entry.meta" class="result-author">{{ entry.meta }}</span>
        </span>
      </UButton>
      <span class="result-duration">{{ entry.duration == null ? '—' : formatDuration(entry.duration) }}</span>
      <UButton
        variant="ghost"
        color="neutral"
        :ui="{ base: 'p-0', label: 'contents' }"
        class="result-more mobile-icon-button"
        type="button"
        :aria-label="`${entry.title}：更多操作`"
        @click="
          menuEntry = entry;
          menuOpen = true;
        "
      >
        <UIcon name="i-tabler-dots" />
      </UButton>
    </div>
    <slot />
  </MobileListCard>
  <MobileSheet v-model="menuOpen" title="视频操作" :description="menuEntry?.title">
    <template v-if="menuEntry"
      ><UiButton
        :disabled="disabled"
        variant="secondary"
        @click="
          emit('toggle', menuEntry.id);
          menuOpen = false;
        "
        >{{ menuEntry.selected ? '取消选择' : '选择此视频' }}</UiButton
      ><ExternalLinkButton v-if="menuEntry.url" :href="menuEntry.url" label="在 Bilibili 打开视频"
        >在 Bilibili 打开视频</ExternalLinkButton
      ></template
    >
  </MobileSheet>
</template>

<style scoped>
.mobile-results {
  flex: 1;
  padding: 0;
  border-radius: 0;
  box-shadow: none;
}

.mobile-result {
  display: flex;
  align-items: center;
  min-height: 3rem;
  padding: 0.125rem 0.5rem;
  border-bottom: 1px solid var(--color-border);
  gap: 0.375rem;
}

.mobile-result-content {
  min-width: 0;
  flex: 1;
  display: flex;
  align-items: center;
  gap: 0.375rem;
  border: 0;
  background: transparent;
  padding: 0.125rem 0;
  text-align: left;
  color: var(--color-text);
}

.result-cover {
  flex-shrink: 0;
  width: 3.75rem;
  height: 2.25rem;
  border-radius: 0.1875rem;
}

.result-copy {
  min-width: 0;
  flex: 1;
  display: grid;
  gap: 0.125rem;
}

.result-copy strong {
  overflow: hidden;
  font-weight: 500;
  font-size: var(--font-13);
  line-height: 1.25rem;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.result-author {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  color: var(--color-muted);
  font-size: var(--font-11);
}

.result-index {
  flex-shrink: 0;
  min-width: 1.25rem;
  color: var(--color-muted);
  font-size: var(--font-11);
  font-variant-numeric: tabular-nums;
}

.result-duration {
  flex-shrink: 0;
  color: var(--color-muted);
  font-size: var(--font-12);
  font-variant-numeric: tabular-nums;
}

.result-more {
  width: 1.75rem;
  height: 1.75rem;
  flex-shrink: 0;
}

.result-more :deep(svg) {
  width: 1rem;
  height: 1rem;
}
</style>
