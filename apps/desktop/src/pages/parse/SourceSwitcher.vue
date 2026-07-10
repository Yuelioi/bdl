<script setup lang="ts">
import { computed } from 'vue'

import type { NormalizedSourceTree } from '../../api/dto'
import UiButton from '../../ui/Button.vue'
import { sourcePartCount } from './parseResultTree'

const { sources, activeSourceId, selectionBySource } = defineProps<{
  sources: NormalizedSourceTree[]
  activeSourceId: string | null
  selectionBySource: Record<string, string[]>
}>()

const emit = defineEmits<{
  select: [sourceId: string]
}>()

const activeSource = computed(() => sources.find((tree) => tree.source.id === activeSourceId) ?? sources[0])
const items = computed(() => [sources.map((tree) => ({
  label: tree.source.title,
  description: `${sourcePartCount(tree)} 项 · 已选 ${selectionBySource[tree.source.id]?.length ?? 0}`,
  icon: activeSourceId === tree.source.id ? 'i-tabler-check' : 'i-tabler-link',
  onSelect: () => emit('select', tree.source.id),
}))])
</script>

<template>
  <span v-if="sources.length === 0" class="inline-flex h-7 items-center gap-2 text-xs font-semibold text-(--color-muted)">
    <span class="size-1.5 rounded-full bg-(--color-dimmed)" aria-hidden="true"></span>
    等待解析
  </span>

  <span
    v-else-if="sources.length === 1"
    class="inline-flex h-7 items-center gap-2 text-xs text-(--color-muted)"
  >
    <UIcon class="size-3.5 text-(--color-success)" name="i-tabler-circle-check" aria-hidden="true" />
    <strong class="text-(--color-text)">已解析 1 个来源</strong>
  </span>

  <UDropdownMenu
    v-else
    :items="items"
    :content="{ align: 'end', sideOffset: 4, collisionPadding: 12 }"
    :ui="{ content: 'w-96 max-w-[calc(100vw-4rem)]', itemDescription: 'truncate' }"
  >
    <UiButton class="max-w-80" variant="secondary" size="compact">
      <strong class="shrink-0">{{ sources.length }} 个来源</strong>
      <span class="min-w-0 truncate text-(--color-muted)">{{ activeSource?.source.title }}</span>
      <UIcon class="size-3.5 shrink-0 text-(--color-muted)" name="i-tabler-chevron-down" aria-hidden="true" />
    </UiButton>
  </UDropdownMenu>
</template>
