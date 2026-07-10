<script setup lang="ts">
import { computed } from 'vue'

import { useThemeStore, type ThemePreference } from '../stores/theme'
import UiIconButton from './IconButton.vue'

const theme = useThemeStore()
const items = computed(() => [[
  ...([
    { value: 'system', label: '跟随系统', icon: 'i-tabler-device-desktop' },
    { value: 'light', label: '浅色', icon: 'i-tabler-sun' },
    { value: 'dark', label: '深色', icon: 'i-tabler-moon' },
  ] as Array<{ value: ThemePreference; label: string; icon: string }>).map((item) => ({
    type: 'checkbox' as const,
    label: item.label,
    icon: item.icon,
    checked: theme.preference === item.value,
    onUpdateChecked: (checked: boolean) => {
      if (checked) theme.setPreference(item.value)
    },
  })),
]])
</script>

<template>
  <UDropdownMenu
    :items="items"
    :content="{ align: 'end', sideOffset: 6, collisionPadding: 12 }"
    :ui="{ content: 'min-w-36' }"
  >
    <UiIconButton :icon="theme.icon" :label="`外观：${theme.preferenceLabel}`" />
  </UDropdownMenu>
</template>
