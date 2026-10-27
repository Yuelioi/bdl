<script setup lang="ts">
import { openExternalUrl } from '../api/tauri';
import { useUiStore } from '../stores/ui';

const {
  href,
  label,
  compact = false,
  showIcon = true,
} = defineProps<{
  href: string;
  label: string;
  compact?: boolean;
  showIcon?: boolean;
}>();
const ui = useUiStore();

const open = async () => {
  try {
    await openExternalUrl(href);
  } catch (error) {
    ui.pushToast(error instanceof Error ? error.message : String(error), 'danger');
  }
};
</script>

<template>
  <UButton
    color="primary"
    variant="link"
    :size="compact ? 'xs' : 'sm'"
    :ui="{ label: 'contents' }"
    type="button"
    class="min-w-0 gap-1 p-0 text-left"
    :class="compact ? 'text-xs' : 'text-[0.8125rem]'"
    :aria-label="label"
    :title="label"
    @click.stop="open"
  >
    <slot />
    <UIcon v-if="showIcon" name="i-tabler-external-link" class="size-3.5 shrink-0" aria-hidden="true" />
  </UButton>
</template>
