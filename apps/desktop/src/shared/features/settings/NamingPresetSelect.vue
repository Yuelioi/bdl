<script setup lang="ts">
import { nextTick, ref } from 'vue';
import type { NamingPreset } from '../../api/dto';
import { useSettingsStore } from '../../stores/settings';
import NamingTemplatePicker from '../../ui/NamingTemplatePicker.vue';
import PresetSelectActions from './PresetSelectActions.vue';
import { useUiStore } from '../../stores/ui';
import NamingPresetDialog from './NamingPresetDialog.vue';

const model = defineModel<string>({ required: true });
const { extension = 'mp4', disabled = false } = defineProps<{ extension?: string; disabled?: boolean }>();
const settings = useSettingsStore();
const picker = ref<InstanceType<typeof NamingTemplatePicker> | null>(null);
const open = ref(false);
const ui = useUiStore();
const emit = defineEmits<{ manage: [] }>();
const manage = () => { emit('manage'); ui.openPresetSettings('naming'); };
const usePreset = async (preset: NamingPreset) => {
  await nextTick();
  picker.value?.selectPreset(preset.id);
};
</script>

<template>
  <NamingTemplatePicker
    ref="picker"
    v-model="model"
    :presets="settings.saved.naming_presets"
    :extension="extension"
    :disabled="disabled"
  >
    <template #actions>
      <PresetSelectActions
        manage-label="管理命名预设"
        :disabled="disabled"
        :create-disabled="settings.saved.naming_presets.length >= 32"
        @create="open = true"
        @manage="manage"
      />
    </template>
  </NamingTemplatePicker>
  <NamingPresetDialog v-model="open" :template="model" :extension="extension" @saved="usePreset" />
</template>
