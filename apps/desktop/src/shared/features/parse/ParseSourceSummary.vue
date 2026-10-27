<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import type { NormalizedSourceTree } from '../../api/dto';
import { coverUrl } from '../../utils/coverUrl';
import UiDialog from '../../ui/Dialog.vue';
import { sourceKindLabels } from './parseResultTree';

const props = defineProps<{ source: NormalizedSourceTree }>();
const item = computed(() => props.source.groups.flatMap((group) => group.items)[0]);
const countLabel = computed(() =>
  props.source.source.kind === 'video'
    ? `${props.source.groups.reduce((total, group) => total + group.items.reduce((count, item) => count + item.parts.length, 0), 0)} 个分 P`
    : `已加载 ${props.source.source.loaded_count}${props.source.source.total_count ? ' / ' + props.source.source.total_count : ''} 项`,
);
const coverFailed = ref(false);
const detailsOpen = ref(false);
watch(
  () => item.value?.cover_url,
  () => {
    coverFailed.value = false;
  },
);
</script>

<template>
  <div class="parse-source-summary">
    <span class="source-summary-artwork" aria-hidden="true">
      <img
        v-if="item?.cover_url && !coverFailed"
        :src="coverUrl(item.cover_url) ?? undefined"
        alt=""
        referrerpolicy="no-referrer"
        @error="coverFailed = true"
      />
      <UIcon v-else name="i-tabler-video" />
    </span>
    <div class="source-summary-copy">
      <UButton
        variant="link"
        color="neutral"
        class="source-summary-title"
        aria-label="查看来源详情"
        :ui="{ label: 'line-clamp-2 whitespace-normal text-left' }"
        @click="detailsOpen = true"
      >
        <span class="source-summary-title-text">{{ source.source.title }}</span>
      </UButton>
      <span class="source-summary-owner"
        >{{ item?.owner_name || sourceKindLabels[source.source.kind] }} · {{ countLabel }}</span
      >
    </div>
  </div>
  <UiDialog v-model="detailsOpen" title="来源详情">
    <p class="source-detail-title">{{ source.source.title }}</p>
    <p v-if="item?.owner_name">{{ item.owner_name }}</p>
    <p class="source-detail-input">{{ source.source.input }}</p>
  </UiDialog>
</template>

<style scoped>
.parse-source-summary {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  min-width: 0;
}

.source-summary-artwork {
  display: grid;
  width: 6rem;
  aspect-ratio: 16 / 9;
  flex-shrink: 0;
  overflow: hidden;
  place-items: center;
  border-radius: var(--radius-6);
  background: var(--color-inset);
  color: var(--color-muted);
}

.source-summary-artwork img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.source-summary-artwork > :deep(svg) {
  width: 1.25rem;
  height: 1.25rem;
}

.source-summary-copy {
  display: grid;
  min-width: 0;
  gap: 0.375rem;
}

.source-summary-title {
  display: block;
  min-width: 0;
  overflow: hidden;
  color: var(--color-text-strong);
  padding: 0;
  text-align: left;
  font-size: var(--font-14);
  font-weight: 600;
  line-height: 1.5;
}

.source-summary-title-text {
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  white-space: normal;
}

.source-summary-owner {
  overflow: hidden;
  color: var(--color-muted);
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: var(--font-12);
}

.source-detail-title,
.source-detail-input {
  overflow-wrap: anywhere;
}

@media (width <= 600px) {
  .source-summary-artwork {
    width: 4.5rem;
  }

  .source-summary-copy {
    gap: 0.25rem;
  }
}
</style>
