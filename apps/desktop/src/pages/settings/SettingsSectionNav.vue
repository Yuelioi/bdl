<script setup lang="ts">
import type { SettingsSection, SettingsSectionId } from './settingsSections';

const activeSection = defineModel<SettingsSectionId>({ required: true });
defineProps<{ sections: readonly SettingsSection[] }>();
</script>

<template>
  <nav
    class="grid min-w-0 content-start gap-1 max-[840px]:flex max-[840px]:overflow-x-auto max-[840px]:pb-1"
    aria-label="设置分区"
  >
    <button
      v-for="section in sections"
      :key="section.id"
      type="button"
      class="grid min-h-13 min-w-0 grid-cols-[18px_minmax(0,1fr)] items-center gap-2.5 rounded-lg border border-transparent bg-transparent px-2.5 py-2 text-left text-(--color-muted) hover:bg-(--color-panel) hover:text-(--color-text) focus-visible:bg-(--color-panel) focus-visible:text-(--color-text) max-[840px]:min-w-38 max-[840px]:shrink-0"
      :class="
        activeSection === section.id
          ? 'border-(--color-accent) bg-(--color-accent-soft) text-(--color-accent-strong) hover:bg-(--color-accent-soft) hover:text-(--color-accent-strong) focus-visible:bg-(--color-accent-soft) focus-visible:text-(--color-accent-strong)'
          : ''
      "
      :aria-current="activeSection === section.id ? 'page' : undefined"
      @click="activeSection = section.id"
    >
      <UIcon :name="section.icon" class="size-[17px]" aria-hidden="true" />
      <span class="grid min-w-0 gap-px">
        <strong class="truncate text-[13px] font-bold text-current">{{ section.label }}</strong>
        <small class="truncate text-[11px] text-(--color-muted) max-[620px]:hidden">{{ section.description }}</small>
      </span>
    </button>
  </nav>
</template>
