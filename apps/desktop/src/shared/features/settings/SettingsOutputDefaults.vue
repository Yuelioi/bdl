<script setup lang="ts">
import { computed } from 'vue';
import { useUiStore } from '../../stores/ui';
import UiButton from '../../ui/Button.vue';
import UiSelect from '../../ui/Select.vue';
import NamingTemplatePicker from '../../ui/NamingTemplatePicker.vue';
import SettingsCard from './SettingsCard.vue';
import { duplicateNamingOptions } from './settingsCatalog';
import type { SettingsForm } from './useSettingsForm';
const { form } = defineProps<{ form: SettingsForm }>();
const { settings, settingsNamingTemplate, settingsDuplicateNamingStrategy, settingsDuplicateDescription } = form;
const ui = useUiStore();
const disabled = computed(() => settings.loading || settings.saving);
</script>

<template>
  <SettingsCard title="默认下载方式">
    <div class="output-default-row">
      <UiSelect
        v-model="settings.draft.selected_download_preset"
        label="下载预设"
        :options="settings.draft.download_presets.map((preset) => ({ label: preset.name, value: preset.id }))"
        :disabled="disabled"
      />
      <UiButton
        variant="ghost"
        aria-label="管理下载预设"
        :disabled="disabled"
        @click="ui.openPresetSettings('download')"
        >管理</UiButton
      >
    </div>
    <NamingTemplatePicker
      v-model="settingsNamingTemplate"
      :presets="settings.saved.naming_presets"
      :extension="settings.draft.output_extension"
      :disabled="disabled"
    >
      <template #actions>
        <UiButton
          variant="ghost"
          aria-label="管理命名预设"
          :disabled="disabled"
          @click="ui.openPresetSettings('naming')"
          >管理</UiButton
        >
      </template>
    </NamingTemplatePicker>
    <div class="output-default-row">
      <UiSelect
        v-model="settingsDuplicateNamingStrategy"
        label="重名处理"
        :options="duplicateNamingOptions"
        :disabled="disabled"
      />
      <UPopover>
        <UiButton variant="ghost" aria-label="重名处理说明"><UIcon name="i-tabler-help" /></UiButton>
        <template #content
          ><p class="max-w-72 m-0 p-3 text-xs text-(--color-muted)">{{ settingsDuplicateDescription }}</p></template
        >
      </UPopover>
    </div>
  </SettingsCard>
</template>

<style scoped>
.output-default-row {
  min-width: 0;
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  align-items: end;
  gap: var(--space-8);
}
</style>
