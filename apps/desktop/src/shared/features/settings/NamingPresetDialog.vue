<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import type { NamingPreset } from '../../api/dto';
import { previewTemplate, validateNamingTemplate, useSettingsStore } from '../../stores/settings';
import NamingTemplateField from '../../ui/NamingTemplateField.vue';
import UiButton from '../../ui/Button.vue';
import UiDialog from '../../ui/Dialog.vue';
import UiTextField from '../../ui/TextField.vue';
import UiInlineNotice from '../../ui/InlineNotice.vue';

const open = defineModel<boolean>({ default: false });
const props = withDefaults(
  defineProps<{
    mode?: 'create' | 'edit' | 'copy';
    preset?: NamingPreset;
    template: string;
    extension: string;
    applyOnSave?: boolean;
  }>(),
  { mode: 'create', applyOnSave: true },
);
const emit = defineEmits<{ saved: [preset: NamingPreset] }>();
const settings = useSettingsStore();
const edited = ref<NamingPreset>({ id: '', name: '', template: '' });
const saving = ref(false);
const saveError = ref('');
const creating = computed(() => props.mode !== 'edit');
const nameError = computed(() => {
  const name = edited.value.name.trim();
  if ([...name].length > 40) return '预设名称最多 40 个字符';
  if (
    [...settings.saved.naming_presets, ...settings.draft.naming_presets].some(
      (preset) => preset.id !== edited.value.id && preset.name.trim() === name,
    )
  )
    return '此名称已存在，请换一个名称';
  return undefined;
});
const templateError = computed(() => validateNamingTemplate(edited.value.template));
watch(open, (value) => {
  if (!value) return;
  let name = props.mode === 'copy' ? `${(props.preset?.name ?? '命名规则').slice(0, 32)} 副本` : '';
  const baseName = name;
  for (let suffix = 2; name && settings.saved.naming_presets.some((p) => p.name === name); suffix++)
    name = `${baseName} ${suffix}`;
  edited.value = {
    id: creating.value ? crypto.randomUUID() : props.preset!.id,
    name: creating.value ? name : props.preset!.name,
    template: props.template,
  };
  saveError.value = '';
});
const save = async () => {
  if (saving.value || !edited.value.name.trim() || nameError.value || templateError.value) return;
  if (creating.value && (settings.saved.naming_presets.length >= 32 || settings.draft.naming_presets.length >= 32)) {
    saveError.value = '命名预设最多保存 32 个，请删除不需要的预设后再添加';
    return;
  }
  saving.value = true;
  saveError.value = '';
  const item = { ...edited.value, name: edited.value.name.trim() };
  try {
    if (!(await settings.persistNamingPreset(item, creating.value))) {
      saveError.value = '预设保存失败，请重试';
      return;
    }
    emit('saved', item);
    open.value = false;
  } finally {
    saving.value = false;
  }
};
</script>

<template>
  <UiDialog
    v-model="open"
    :title="mode === 'edit' ? '编辑命名预设' : mode === 'copy' ? '复制命名预设' : '新增命名预设'"
    description="保存后可在下载时直接选择"
    :dismissible="!saving"
    :show-close="!saving"
  >
    <UiTextField
      v-model="edited.name"
      label="预设名称"
      placeholder="例如：按 UP 主归档"
      :error="nameError"
      :disabled="saving"
    />
    <NamingTemplateField
      v-model="edited.template"
      :quick-variables="false"
      :error="templateError ?? undefined"
      :disabled="saving"
    />
    <p class="m-0 text-xs leading-5 wrap-anywhere text-(--color-muted)">
      文件名预览：<code>{{ previewTemplate(edited.template, extension) }}</code>
    </p>
    <UiInlineNotice v-if="saveError" tone="danger">{{ saveError }}</UiInlineNotice>
    <template #footer>
      <div class="flex flex-1 justify-end gap-2">
        <UiButton variant="secondary" :disabled="saving" @click="open = false">取消</UiButton>
        <UiButton :disabled="saving || !edited.name.trim() || Boolean(nameError || templateError)" @click="save">{{
          saving ? '正在保存…' : applyOnSave ? '保存并使用' : '保存预设'
        }}</UiButton>
      </div>
    </template>
  </UiDialog>
</template>
