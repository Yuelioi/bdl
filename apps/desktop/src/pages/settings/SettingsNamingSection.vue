<script setup lang="ts">
import { defaultNamingTemplate, namingTemplatePresets, namingVariables } from '../../stores/settings';
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
  resetNamingTemplate,
  formatNamingVariable,
} = form;
</script>

<template>
  <section class="settings-block">
    <UiTextField v-model="settingsNamingTemplate" label="命名模板" :placeholder="defaultNamingTemplate" />
    <p v-if="settings.namingTemplateError" class="settings-field-error">{{ settings.namingTemplateError }}</p>
    <div class="template-presets" aria-label="命名模板预设">
      <button
        v-for="preset in namingTemplatePresets"
        :key="preset.label"
        type="button"
        @click="settings.setNamingTemplate(preset.value)"
      >
        {{ preset.label }}
      </button>
      <button type="button" @click="resetNamingTemplate">恢复默认</button>
    </div>
    <div class="settings-preview">
      <span>文件名预览</span><code>{{ settings.namingPreview }}</code>
    </div>
    <UiSelect v-model="settingsDuplicateNamingStrategy" label="重名处理" :options="duplicateNamingOptions" />
    <p class="settings-note">{{ settingsDuplicateDescription }}</p>
    <details class="template-help">
      <summary>查看可用变量</summary>
      <div>
        <span v-for="variable in namingVariables" :key="variable.name" :title="variable.desc"
          ><code>{{ formatNamingVariable(variable.name) }}</code
          ><small>{{ variable.desc }}</small></span
        >
      </div>
    </details>
  </section>
</template>
