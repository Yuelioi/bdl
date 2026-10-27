<script setup lang="ts">
import { useSettingsStore } from '../stores/settings';
const settings = useSettingsStore();
</script>
<template>
  <label class="clipboard-preference">
    <span
      >返回应用时识别剪贴板链接<small class="clipboard-help">仅识别 Bilibili 链接，确认添加后再手动解析。</small></span
    >
    <USwitch
      :model-value="settings.saved.clipboard_monitor_enabled"
      :disabled="settings.saving"
      aria-label="返回应用时识别剪贴板链接"
      @update:model-value="settings.saveAppPreferences({ clipboard_monitor_enabled: $event === true })"
    />
  </label>
</template>
<style scoped>
.clipboard-preference {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  font-size: var(--font-13);
}

.clipboard-preference > span {
  display: grid;
  gap: 0.25rem;
}

.clipboard-help {
  margin: 0;
  font-size: var(--font-12);
  color: var(--color-muted);
}
</style>
