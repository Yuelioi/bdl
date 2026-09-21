<script setup lang="ts">
import { computed, ref } from 'vue'
import type { NamingPreset } from '../api/dto'
import { namingTemplatePresets, previewTemplate, validateNamingTemplate } from '../stores/settings'
import NamingTemplateField from './NamingTemplateField.vue'
import UiSelect from './Select.vue'

const model = defineModel<string>({ required: true })
const props = withDefaults(defineProps<{ presets: NamingPreset[]; extension?: string; editable?: boolean }>(), { extension: 'mp4', editable: false })
const custom = ref(false)
const selectedId = ref('')
const choices = computed(() => [
  ...namingTemplatePresets.map((preset, index) => ({ label: preset.label, value: `builtin-${index}`, template: preset.value })),
  { label: '直接保存到下载目录', value: 'direct', template: '{title} - P{part_index} - {part_title}.{ext}' },
  ...props.presets.map((preset) => ({ label: preset.name, value: preset.id, template: preset.template })),
])
const selected = computed({
  get: () => custom.value ? 'custom' : choices.value.find((item) => item.value === selectedId.value && item.template === model.value)?.value ?? choices.value.find((item) => item.template === model.value)?.value ?? 'custom',
  set: (value: string) => {
    custom.value = value === 'custom'
    selectedId.value = value
    const preset = choices.value.find((item) => item.value === value)
    if (preset) model.value = preset.template
  },
})
const options = computed(() => [...choices.value, { label: '自定义模板', value: 'custom' }])
</script>

<template>
  <div class="grid min-w-0 gap-3">
    <UiSelect v-model="selected" label="命名预设" :options="options" />
    <NamingTemplateField
      v-if="editable || selected === 'custom'"
      v-model="model"
      :error="validateNamingTemplate(model) ?? undefined"
    />
    <p class="m-0 text-xs leading-5 wrap-anywhere text-(--color-muted)">文件名预览：<code>{{ previewTemplate(model, extension) }}</code></p>
  </div>
</template>
