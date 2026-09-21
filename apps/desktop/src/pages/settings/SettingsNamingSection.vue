<script setup lang="ts">
import { namingVariables } from '../../stores/settings';
import { ref } from 'vue';
import NamingTemplatePicker from '../../ui/NamingTemplatePicker.vue';
import UiButton from '../../ui/Button.vue';
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
  formatNamingVariable,
} = form;
const presetName = ref('');
</script>

<template>
  <section class="settings-block">
    <NamingTemplatePicker v-model="settingsNamingTemplate" :presets="settings.draft.naming_presets" :extension="settings.draft.output_extension" editable />
    <div class="flex items-end gap-2">
      <UiTextField v-model="presetName" label="预设名称" placeholder="例如：收藏用命名" />
      <UiButton variant="secondary" :disabled="!presetName.trim() || Boolean(settings.namingTemplateError)" @click="settings.saveNamingPreset(presetName)">保存为预设</UiButton>
    </div>
    <p class="settings-note">同名预设会更新模板。预设和默认偏好随页面保存，下次下载时可直接选择。</p>
    <div v-if="settings.draft.naming_presets.length" class="grid gap-1">
      <div v-for="preset in settings.draft.naming_presets" :key="preset.id" class="flex items-center justify-between gap-2">
        <button type="button" class="text-left text-sm" @click="settings.setNamingTemplate(preset.template); presetName = preset.name">{{ preset.name }}</button>
        <UiButton variant="ghost" :aria-label="`删除预设 ${preset.name}`" @click="settings.removeNamingPreset(preset.id)">删除</UiButton>
      </div>
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
