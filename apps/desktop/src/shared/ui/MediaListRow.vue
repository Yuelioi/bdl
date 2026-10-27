<script setup lang="ts">
import { ref, watch } from 'vue';
import UiCheckbox from './Checkbox.vue';
import { coverUrl } from '../utils/coverUrl';
import { formatDuration } from '../utils/duration';

const props = defineProps<{
  title: string;
  cover?: string | null;
  secondary?: string | null;
  secondaryIcon?: string;
  published?: string | null;
  duration?: number | null;
  sizeLabel?: string | null;
  selected?: boolean;
  indeterminate?: boolean;
  flat?: boolean;
  compact?: boolean;
  index?: string;
  disabled?: boolean;
  playable?: boolean;
  playbackLabel?: string;
}>();
const emit = defineEmits<{ toggle: []; play: [] }>();
const failed = ref(false);
watch(
  () => props.cover,
  () => {
    failed.value = false;
  },
);
function toggleCard(event: MouseEvent) {
  if (
    props.disabled ||
    (event.target instanceof Element && event.target.closest('button, input, a, label, [role="checkbox"]'))
  )
    return;
  emit('toggle');
}
</script>

<template>
  <article
    class="media-list-row"
    :class="{ 'is-selected': selected, 'media-list-row-flat': flat, 'media-list-row-compact': compact }"
    :data-selected="Boolean(selected)"
    role="listitem"
    @click="toggleCard($event)"
  >
    <div class="media-selection">
      <UiCheckbox
        :model-value="indeterminate ? 'indeterminate' : Boolean(selected)"
        :label="`选择 ${title}`"
        :disabled="disabled"
        compact
        @update:model-value="emit('toggle')"
      />
    </div>
    <div class="media-cover">
      <img
        v-if="cover && !failed"
        :src="coverUrl(cover) ?? undefined"
        referrerpolicy="no-referrer"
        alt=""
        loading="lazy"
        @error="failed = true"
      />
      <UIcon v-else-if="!playable" name="i-tabler-video" class="cover-placeholder" aria-hidden="true" />
      <UButton
        variant="ghost"
        color="neutral"
        :ui="{ base: 'p-0', label: 'contents' }"
        v-if="playable"
        class="media-play"
        type="button"
        :aria-label="playbackLabel || `播放 ${title}`"
        :title="playbackLabel || `播放 ${title}`"
        @click.stop="emit('play')"
      >
        <UIcon name="i-tabler-player-play" class="media-play-icon" aria-hidden="true" />
      </UButton>
    </div>
    <span v-if="index" class="media-index">{{ index }}</span>
    <div class="media-copy">
      <UTooltip
        :text="title"
        :delay-duration="350"
        :ui="{
          content: 'h-auto max-w-xl whitespace-normal break-words text-left leading-5',
          text: 'whitespace-normal overflow-visible',
        }"
      >
        <UButton
          variant="ghost"
          color="neutral"
          :ui="{ base: 'p-0', label: 'contents' }"
          class="media-title"
          type="button"
          :aria-label="`${selected ? '取消选择' : '选择'} ${title}`"
          :aria-pressed="Boolean(selected)"
          :disabled="disabled"
          @click="emit('toggle')"
        >
          {{ title }}
        </UButton>
      </UTooltip>
      <div v-if="secondary || published" class="media-metadata">
        <span v-if="secondary" class="media-author" :title="secondary"
          ><UIcon :name="secondaryIcon || 'i-tabler-user'" class="media-author-icon" aria-hidden="true" />{{
            secondary
          }}</span
        >
        <span v-if="published" class="media-date" :title="`发布于 ${published}`">{{ published }}</span>
      </div>
    </div>
    <div class="media-facts">
      <span v-if="duration && duration > 0" class="media-duration" aria-label="时长"
        ><UIcon name="i-tabler-clock" aria-hidden="true" />{{ formatDuration(duration) }}</span
      >
      <span v-if="sizeLabel && sizeLabel !== '--'" class="media-size" aria-label="大小"
        ><UIcon name="i-tabler-file" aria-hidden="true" />{{ sizeLabel }}</span
      >
      <slot name="detail" />
      <slot name="facts" />
    </div>
    <div v-if="$slots.actions" class="media-actions" @click.stop @keydown.stop><slot name="actions" /></div>
  </article>
</template>

<style scoped>
.media-list-row {
  min-width: 0;
  min-height: 4.5rem;
  display: grid;
  grid-template-columns: 1.25rem 4rem minmax(0, 1fr) auto;
  align-items: center;
  gap: 0.75rem;
  padding: 0.625rem 0.25rem;
  border: 0;
  border-bottom: 1px solid var(--color-border);
  border-radius: 0;
  background: var(--color-surface);
}

.media-list-row:has(.media-actions) {
  grid-template-columns: 1.25rem 4rem minmax(0, 1fr) auto auto;
}

.media-list-row:hover {
  background: var(--color-hover-surface);
}

.media-list-row-flat {
  width: 100%;
  height: 4.5rem;
}

.media-selection {
  display: grid;
  place-items: center;
}

.media-cover {
  position: relative;
  width: 4rem;
  height: 2.25rem;
  overflow: hidden;
  display: grid;
  place-items: center;
  border-radius: var(--radius-4);
  background: var(--color-inset);
}

.media-cover img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.cover-placeholder {
  width: 1.25rem;
  height: 1.25rem;
  color: var(--color-dimmed);
}

.media-play {
  position: absolute;
  inset: 0;
  display: grid;
  place-items: center;
  border: 1px solid transparent;
  background: transparent;
  color: white;
  cursor: pointer;
}

.media-play:hover {
  background: rgb(0 0 0 / 15%);
}

.media-play-icon {
  width: 1.5rem;
  height: 1.5rem;
  padding: 0.375rem;
  border-radius: 50%;
  background: rgb(0 0 0 / 60%);
  opacity: 0;
}

.media-play:hover .media-play-icon,
.media-play:focus-visible .media-play-icon {
  opacity: 1;
}

.media-copy {
  min-width: 0;
  display: grid;
  align-content: center;
  gap: 0.25rem;
}

.media-title {
  display: block;
  width: 100%;
  min-width: 0;
  overflow: hidden;
  border: 1px solid transparent;
  padding: 0;
  background: transparent;
  color: var(--color-text-strong);
  font-size: var(--font-13);
  font-weight: 400;
  line-height: 1.25rem;
  text-align: left;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.media-title:focus-visible,
.media-play:focus-visible {
  outline: none;
  border-color: var(--color-accent);
}

.media-metadata {
  min-width: 0;
  display: flex;
  gap: 0.5rem;
  color: var(--color-muted);
  font-size: var(--font-12);
  line-height: 1.125rem;
}

.media-author {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.media-author-icon {
  display: none;
}

.media-date {
  flex-shrink: 0;
  font-variant-numeric: tabular-nums;
}

.media-facts {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 0.5rem;
  color: var(--color-muted);
  font-size: var(--font-12);
  font-variant-numeric: tabular-nums;
}

.media-facts > span {
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
  white-space: nowrap;
}

.media-facts :deep(.iconify) {
  display: none;
}

.media-actions {
  display: flex;
  align-items: center;
  gap: 0.25rem;
}

.media-index {
  color: var(--color-muted);
  font-size: var(--font-12);
  font-variant-numeric: tabular-nums;
}

.media-list-row-compact {
  height: 3rem;
  min-height: 3rem;
  grid-template-columns: 1.25rem 3.75rem 2rem minmax(0, 1fr) auto;
  gap: 0.625rem;
  padding: 0.25rem 0.125rem;
}

.media-list-row-compact:has(.media-actions) {
  grid-template-columns: 1.25rem 3.75rem 2rem minmax(0, 1fr) auto auto;
}

.media-list-row-compact .media-cover {
  width: 3.75rem;
  height: 2.25rem;
  border-radius: 0.1875rem;
}

.media-list-row-compact .media-metadata {
  display: none;
}

@media (width <= 700px) {
  .media-date {
    display: none;
  }
}
</style>
