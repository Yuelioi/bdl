<script setup lang="ts">
import { ref } from 'vue';
import NamingTemplatePicker from '../../ui/NamingTemplatePicker.vue';
import UiButton from '../../ui/Button.vue';
import UiIconButton from '../../ui/IconButton.vue';
import UiSelect from '../../ui/Select.vue';
import UiTextField from '../../ui/TextField.vue';
import { duplicateNamingOptions } from './settingsCatalog';
import type { SettingsForm } from './useSettingsForm';

const { form } = defineProps<{ form: SettingsForm }>();
const {
  settings,
  settingsNamingTemplate,
  settingsDuplicateNamingStrategy,
  settingsDuplicateDescription,
} = form;
const presetName = ref('');
</script>

<template>
  <section class="settings-block">
    <NamingTemplatePicker v-model="settingsNamingTemplate" :presets="settings.draft.naming_presets" :extension="settings.draft.output_extension" editable />
    <div class="preset-create-row">
      <UiTextField v-model="presetName" label="预设名称" placeholder="例如：收藏用命名" />
      <UiButton variant="secondary" :disabled="!presetName.trim() || Boolean(settings.namingTemplateError)" @click="settings.saveNamingPreset(presetName)">
        <UIcon name="i-tabler-device-floppy" class="size-4" aria-hidden="true" />
        保存为预设
      </UiButton>
    </div>
    <p class="settings-note">同名预设会更新模板。预设和默认偏好随页面保存，下次下载时可直接选择。</p>
    <div v-if="settings.draft.naming_presets.length" class="preset-list">
      <div v-for="preset in settings.draft.naming_presets" :key="preset.id" class="preset-row">
        <button type="button" class="preset-select" @click="settings.setNamingTemplate(preset.template); presetName = preset.name">
          <strong>{{ preset.name }}</strong>
          <small>{{ preset.template }}</small>
        </button>
        <UiIconButton icon="trash" variant="ghost" :label="`删除预设 ${preset.name}`" @click="settings.removeNamingPreset(preset.id)" />
      </div>
    </div>
    <UiSelect v-model="settingsDuplicateNamingStrategy" label="重名处理" :options="duplicateNamingOptions" />
    <p class="settings-note">{{ settingsDuplicateDescription }}</p>
  </section>
</template>

<style scoped>
.preset-create-row {
  min-width: 0;
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  align-items: end;
  gap: var(--space-8);
}

.preset-list {
  display: grid;
  gap: var(--space-6);
}

.preset-row {
  min-width: 0;
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  align-items: center;
  gap: var(--space-8);
  padding: var(--space-6) var(--space-8) var(--space-6) var(--space-10);
  border-radius: var(--radius-6);
  background: var(--color-panel);
}

.preset-select {
  min-width: 0;
  display: grid;
  gap: 2px;
  border: 0;
  background: transparent;
  color: var(--color-text);
  padding: 0;
  text-align: left;
}

.preset-select strong,
.preset-select small {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.preset-select strong {
  font-size: var(--font-13);
}

.preset-select small {
  color: var(--color-muted);
  font-size: var(--font-11);
}
</style>
