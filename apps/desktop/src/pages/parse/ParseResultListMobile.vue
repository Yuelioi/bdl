<script setup lang="ts">
import MediaArtwork from '../mobile/MediaArtwork.vue';
import MobileSheet from '../mobile/MobileSheet.vue';
import UiButton from '../../ui/Button.vue';
import ExternalLinkButton from '../../ui/ExternalLinkButton.vue';
import { bilibiliVideoUrl } from '../../utils/bilibiliLinks';
import { computed, ref } from 'vue';
import type { NormalizedSourceTree } from '../../api/dto';
import UiCheckbox from '../../ui/Checkbox.vue';

import type { ParseResultRow } from './parseResultTree';

const props = defineProps<{
  rows: ParseResultRow[];
  source: NormalizedSourceTree;
  selectedIds: string[];
  disabled?: boolean;
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
          (part) =>
            [
              part.id,
              {
                url: bilibiliVideoUrl(item),
                cover: item.cover_url,
                duration: part.duration_seconds ?? item.duration_seconds,
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
</script>

<template>
  <div class="mobile-results" role="list" aria-label="解析结果">
    <div
      v-for="entry in entries"
      :key="entry.id"
      class="mobile-result"
      :class="{ selected: entry.selected }"
      role="listitem"
    >
      <UiCheckbox
        :model-value="entry.selected"
        :label="`选择 ${entry.title}`"
        :disabled="disabled"
        compact
        @update:model-value="emit('toggle', entry.id)"
      />
      <button
        class="mobile-result-content"
        type="button"
        :disabled="disabled"
        :aria-label="`选择 ${entry.title}`"
        :aria-pressed="entry.selected"
        @click="emit('toggle', entry.id)"
      >
        <MediaArtwork class="result-cover" :src="entry.cover" :duration="entry.duration" />
        <span class="result-copy">
          <strong>{{ entry.title }}</strong>
          <span class="result-meta">
            <span v-if="entry.meta" class="result-author"><UIcon name="i-tabler-user" />{{ entry.meta }}</span>
          </span>
        </span>
      </button>
      <button
        class="result-more mobile-icon-button"
        type="button"
        :aria-label="`${entry.title}：更多操作`"
        @click="
          menuEntry = entry;
          menuOpen = true;
        "
      >
        <UIcon name="i-tabler-dots" />
      </button>
    </div>
    <slot />
  </div>
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
  min-height: 0;
  overflow-y: auto;
  flex: 1;
  padding: 0;
}

.mobile-result {
  display: flex;
  align-items: center;
  position: relative;
  height: 62px;
  padding: 7px 30px 7px 0;
  border: 0;
  border-bottom: 1px solid var(--color-border);
  border-radius: 0;
  background: transparent;
  gap: 8px;
}

.mobile-result.selected {
  background: var(--color-accent-faint);
}

.mobile-result-content {
  min-width: 0;
  flex: 1;
  display: flex;
  align-items: center;
  gap: 8px;
  border: 0;
  background: transparent;
  padding: 0;
  text-align: left;
  color: var(--color-text);
}

.result-cover {
  flex-shrink: 0;
  display: grid;
  place-items: center;
  overflow: hidden;
  background: var(--color-panel);
  color: var(--color-muted);
  width: 72px;
  height: 48px;
  border-radius: 4px;
}

.result-cover img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.result-copy {
  min-width: 0;
  flex: 1;
  display: grid;
  grid-template-rows: repeat(2, 24px);
  align-items: center;
  height: 48px;
}

.result-copy strong {
  min-width: 0;
  overflow: hidden;
  display: block;
  font-weight: var(--mobile-weight-item);
  font-size: var(--mobile-font-item-title);
  line-height: 24px;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.result-meta {
  display: flex;
  align-items: center;
  min-width: 0;
  gap: 6px;
  color: var(--color-muted);
  font-size: var(--mobile-font-caption);
  line-height: 20px;
  font-variant-numeric: tabular-nums;
}

.result-author {
  min-width: 0;
  flex: 1;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  display: flex;
  align-items: center;
  gap: 5px;
}

.result-author :deep(svg) {
  width: 14px;
  height: 14px;
  flex-shrink: 0;
}

.result-index {
  color: var(--color-dimmed);
}

.result-more {
  position: absolute;
  right: 0;
  top: 7px;
  width: 30px;
  height: 30px;
}

.result-more :deep(svg) {
  width: 18px;
  height: 18px;
}
</style>
