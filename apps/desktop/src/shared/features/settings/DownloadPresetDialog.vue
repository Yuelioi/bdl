<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import type { DownloadPreset } from '../../api/dto';
import { useSettingsStore } from '../../stores/settings';
import { builtinDownloadPresets, cloneWorkflow, downloadPresetsError } from '../../utils/downloadWorkflow';
import UiButton from '../../ui/Button.vue';
import UiDialog from '../../ui/Dialog.vue';
import UiSelect from '../../ui/Select.vue';
import UiTextField from '../../ui/TextField.vue';
import UiInlineNotice from '../../ui/InlineNotice.vue';
import DownloadWorkflowEditor from './DownloadWorkflowEditor.vue';

const open = defineModel<boolean>({ default: false });
const props = withDefaults(
  defineProps<{ mode?: 'create' | 'edit' | 'copy'; presetId?: string; applyOnSave?: boolean }>(),
  {
    mode: 'create',
    presetId: '',
    applyOnSave: true,
  },
);
const emit = defineEmits<{ saved: [id: string] }>();
const settings = useSettingsStore();
const saving = ref(false);
const edited = ref<DownloadPreset | null>(null);
const templateId = ref('video');
const templates = builtinDownloadPresets();
const saveError = ref('');
const creating = computed(() => props.mode !== 'edit');
const modalError = computed(() => {
  if (!edited.value) return '请选择下载预设';
  const next = creating.value
    ? [...settings.saved.download_presets, edited.value]
    : settings.saved.download_presets.map((p) => (p.id === edited.value?.id ? edited.value : p));
  return downloadPresetsError(next, edited.value.id);
});
watch(open, (value) => {
  if (!value) return;
  const savedPreset = settings.saved.download_presets.find((p) => p.id === props.presetId);
  if (!savedPreset && !creating.value) {
    edited.value = null;
    return;
  }
  const source = savedPreset ?? templates[0];
  let name = props.mode === 'copy' ? `${source.name.slice(0, 32)} 副本` : '新预设';
  const baseName = name;
  for (let suffix = 2; settings.saved.download_presets.some((p) => p.name.trim() === name); suffix++)
    name = `${baseName} ${suffix}`;
  edited.value = {
    id: creating.value ? `custom-${crypto.randomUUID()}` : source.id,
    name: creating.value ? name : source.name,
    workflow: cloneWorkflow(source.workflow),
  };
  templateId.value = settings.saved.download_presets.some((p) => p.id === props.presetId) ? 'current' : 'video';
  saveError.value = '';
});
const templateOptions = computed(() => {
  const source = settings.saved.download_presets.find((p) => p.id === props.presetId);
  return [
    ...(source ? [{ label: `当前预设：${source.name}`, value: 'current' }] : []),
    ...templates.map((p) => ({ label: p.name, value: p.id })),
  ];
});
const chooseTemplate = (id: string) => {
  const source =
    id === 'current'
      ? settings.saved.download_presets.find((p) => p.id === props.presetId)
      : templates.find((p) => p.id === id);
  if (source && edited.value) edited.value.workflow = cloneWorkflow(source.workflow);
};
const save = async () => {
  if (!edited.value || modalError.value || saving.value) return;
  saving.value = true;
  saveError.value = '';
  const item = { ...edited.value, name: edited.value.name.trim(), workflow: cloneWorkflow(edited.value.workflow) };
  try {
    if (!(await settings.persistDownloadPreset(item, creating.value, props.applyOnSave))) {
      saveError.value = '预设保存失败，请重试';
      return;
    }
    emit('saved', item.id);
    open.value = false;
  } finally {
    saving.value = false;
  }
};
</script>

<template>
  <UiDialog
    v-model="open"
    :title="mode === 'edit' ? '编辑预设' : mode === 'copy' ? '复制预设' : '新增预设'"
    description="选择下载内容与输出方式"
    :dismissible="!saving"
    :show-close="!saving"
  >
    <div v-if="edited" class="grid min-w-0 gap-4">
      <div class="preset-editor-fields">
        <UiTextField v-model="edited.name" label="预设名称" :disabled="saving" :maxlength="40" />
        <UiSelect
          v-if="creating"
          v-model="templateId"
          label="使用模板"
          :options="templateOptions"
          :disabled="saving"
          @update:model-value="chooseTemplate"
        />
      </div>
      <DownloadWorkflowEditor :key="edited.id + templateId" v-model="edited.workflow" :disabled="saving" />
      <UiInlineNotice v-if="modalError || saveError" tone="danger">{{ saveError || modalError }}</UiInlineNotice>
    </div>
    <UiInlineNotice v-else tone="danger">该预设已不存在，请关闭后重新选择。</UiInlineNotice>
    <template #footer>
      <div class="flex flex-1 justify-end gap-2">
        <UiButton variant="secondary" :disabled="saving" @click="open = false">取消</UiButton>
        <UiButton :disabled="saving || !!modalError" @click="save">{{ saving ? '正在保存…' : '保存预设' }}</UiButton>
      </div>
    </template>
  </UiDialog>
</template>

<style scoped>
.preset-editor-fields {
  min-width: 0;
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(min(100%, 12rem), 1fr));
  align-items: start;
  gap: var(--space-12);
}
</style>
