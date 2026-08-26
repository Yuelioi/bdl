<script setup lang="ts">
import { openExternalUrl } from '../api/tauri'
import { useUiStore } from '../stores/ui'

const {
  href,
  label,
  compact = false,
  showIcon = true,
} = defineProps<{
  href: string
  label: string
  compact?: boolean
  showIcon?: boolean
}>()
const ui = useUiStore()

const open = async () => {
  try {
    await openExternalUrl(href)
  } catch (error) {
    ui.pushToast(error instanceof Error ? error.message : String(error), 'danger')
  }
}
</script>

<template>
  <button
    type="button"
    class="inline-flex min-w-0 items-center gap-1 rounded text-left text-(--color-accent-strong) hover:underline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-(--color-focus-outline)"
    :class="compact ? 'text-xs' : 'text-[13px]'"
    :aria-label="label"
    :title="label"
    @click.stop="open"
  >
    <slot />
    <UIcon v-if="showIcon" name="i-tabler-external-link" class="size-3.5 shrink-0" aria-hidden="true" />
  </button>
</template>
