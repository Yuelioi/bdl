<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import type { NamingPreset } from '../../api/dto';
import { builtinNamingPresets, previewTemplate, useSettingsStore } from '../../stores/settings';
import { workflowOutputs } from '../../utils/downloadWorkflow';
import UiButton from '../../ui/Button.vue';
import UiIconButton from '../../ui/IconButton.vue';
import UiInlineNotice from '../../ui/InlineNotice.vue';
import UiTabs from '../../ui/Tabs.vue';
import DownloadPresetDialog from './DownloadPresetDialog.vue';
import NamingPresetDialog from './NamingPresetDialog.vue';

const activeKind = defineModel<'download' | 'naming'>('kind', { default: 'download' });
const settings = useSettingsStore();
const query = ref('');
const deleting = ref('');
const error = ref('');
const tabs = [
  { label: '下载预设', value: 'download' },
  { label: '命名预设', value: 'naming' },
];
const mode = ref<'create' | 'edit' | 'copy'>('create');
const editingId = ref('');
const downloadEditorOpen = ref(false);
const namingEditorOpen = ref(false);
const namingSource = ref<NamingPreset>();
const namingTemplate = ref('');
const builtins = builtinNamingPresets();
const atLimit = computed(
  () =>
    (activeKind.value === 'download' ? settings.saved.download_presets : settings.saved.naming_presets).length >= 32,
);
const activeNamingId = computed(
  () =>
    [...builtins, ...settings.saved.naming_presets].find(
      (p) => p.template === settings.saved.naming_template || `${p.template}.{ext}` === settings.saved.naming_template,
    )?.id,
);
const rows = computed(() => {
  const items =
    activeKind.value === 'download'
      ? settings.saved.download_presets.map((p) => ({
          id: p.id,
          name: p.name,
          summary: workflowOutputs(p.workflow).join(' · '),
          builtin: false,
          active: p.id === settings.saved.selected_download_preset,
        }))
      : [
          ...settings.saved.naming_presets.map((p) => ({ ...p, builtin: false })),
          ...builtins.map((p) => ({ ...p, builtin: true })),
        ].map((p) => ({
          id: p.id,
          name: p.name,
          summary: previewTemplate(p.template, settings.saved.output_extension),
          builtin: p.builtin,
          active: p.id === activeNamingId.value,
        }));
  const needle = query.value.trim().toLocaleLowerCase();
  return items.filter((p) => !needle || `${p.name} ${p.summary}`.toLocaleLowerCase().includes(needle));
});
watch(activeKind, () => {
  query.value = '';
  error.value = '';
});
const begin = (value: 'create' | 'edit' | 'copy', id = '') => {
  mode.value = value;
  editingId.value = id;
  if (activeKind.value === 'download') downloadEditorOpen.value = true;
  else {
    namingSource.value = [...settings.saved.naming_presets, ...builtins].find((p) => p.id === id);
    namingTemplate.value = namingSource.value?.template ?? settings.saved.naming_template;
    namingEditorOpen.value = true;
  }
};
const useDefault = async (id: string) => {
  if (deleting.value) return;
  deleting.value = id;
  error.value = '';
  try {
    const preset = [...settings.saved.naming_presets, ...builtins].find((p) => p.id === id);
    const saved = await settings.saveAppPreferences(
      activeKind.value === 'download' ? { selected_download_preset: id } : { naming_template: preset!.template },
    );
    if (!saved) error.value = '设置默认失败，请重试';
  } finally {
    deleting.value = '';
  }
};
const remove = async (id: string) => {
  if (deleting.value) return;
  deleting.value = id;
  error.value = '';
  try {
    const saved =
      activeKind.value === 'download' ? await settings.deleteDownloadPreset(id) : await settings.deleteNamingPreset(id);
    if (!saved) error.value = '删除失败，请重试';
  } finally {
    deleting.value = '';
  }
};
</script>

<template>
  <section class="preset-manager" aria-label="预设管理">
    <UiTabs v-model="activeKind" :tabs="tabs" />
    <div class="preset-manager-toolbar">
      <UInput
        v-model="query"
        icon="i-tabler-search"
        placeholder="搜索预设"
        aria-label="搜索预设"
        color="neutral"
        variant="outline"
        size="sm"
        class="min-w-0 w-full"
        :ui="{ base: 'h-(--height-input) text-(length:--font-13)' }"
      />
      <UiButton :disabled="Boolean(deleting) || atLimit" @click="begin('create')"
        >新增预设</UiButton
      >
    </div>
    <UiInlineNotice v-if="error" tone="danger">{{ error }}</UiInlineNotice>
    <ul class="preset-manager-list" :aria-label="activeKind === 'download' ? '下载预设列表' : '命名预设列表'">
      <li v-for="row in rows" :key="row.id" role="listitem" :aria-label="row.name" class="preset-manager-row">
        <div class="preset-manager-copy">
          <div class="preset-manager-name" :class="{ 'preset-manager-name-active': row.active }">
            <strong>{{ row.name }}</strong
            ><span v-if="row.active" class="preset-current">默认</span
            ><span v-if="row.builtin" class="preset-builtin">内置</span>
          </div>
          <p>{{ row.summary }}</p>
        </div>
        <div class="preset-manager-actions">
          <UiButton
            v-if="!row.active"
            class="preset-action-default"
            size="compact"
            :aria-label="`设为默认 ${row.name}`"
            variant="ghost"
            :disabled="Boolean(deleting)"
            @click="useDefault(row.id)"
            >设为默认</UiButton
          >
          <UiIconButton
            v-if="!row.builtin"
            class="preset-action-edit"
            icon="pencil"
            :label="`编辑预设 ${row.name}`"
            variant="ghost"
            :disabled="Boolean(deleting)"
            @click="begin('edit', row.id)"
          />
          <UiIconButton
            class="preset-action-copy"
            icon="copy"
            :label="`复制预设 ${row.name}`"
            variant="ghost"
            :disabled="Boolean(deleting) || atLimit"
            @click="begin('copy', row.id)"
          />
          <UiIconButton
            v-if="!row.builtin"
            class="preset-action-delete"
            icon="trash"
            :label="`删除预设 ${row.name}`"
            variant="ghost"
            :disabled="Boolean(deleting) || (activeKind === 'download' && settings.saved.download_presets.length <= 1)"
            @click="remove(row.id)"
          />
        </div>
      </li>
      <li v-if="!rows.length" class="preset-manager-empty">没有找到匹配的预设</li>
    </ul>
    <p v-if="activeKind === 'naming'" class="preset-manager-note">内置命名方式可以复制后修改。</p>
    <p v-else-if="settings.saved.download_presets.length === 1" class="preset-manager-note">至少保留一个下载预设。</p>
  </section>
  <DownloadPresetDialog v-model="downloadEditorOpen" :mode="mode" :preset-id="editingId" :apply-on-save="false" />
  <NamingPresetDialog
    v-model="namingEditorOpen"
    :mode="mode"
    :apply-on-save="false"
    :preset="namingSource"
    :template="namingTemplate"
    :extension="settings.saved.output_extension"
  />
</template>

<style scoped>
.preset-manager {
  min-width: 0;
  display: grid;
  gap: var(--space-16);
}

.preset-manager-toolbar {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  align-items: center;
  gap: var(--space-8);
}

.preset-manager-list {
  min-width: 0;
  margin: 0;
  padding: 0;
  list-style: none;
}

.preset-manager-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  align-items: center;
  gap: var(--space-8);
  padding-block: var(--space-12);
  border-bottom: 1px solid var(--color-border);
}

.preset-manager-row:first-child {
  padding-top: 0;
}

.preset-manager-row:last-child {
  border-bottom: 0;
  padding-bottom: 0;
}

.preset-manager-copy {
  min-width: 0;
  display: grid;
  gap: var(--space-4);
}

.preset-manager-name {
  display: flex;
  min-width: 0;
  align-items: center;
  gap: var(--space-6);
}

.preset-manager-name strong {
  min-width: 0;
  overflow-wrap: anywhere;
  font-size: var(--font-13);
  font-weight: 500;
}

.preset-manager-name-active strong {
  color: var(--color-accent-strong);
  font-weight: 600;
}

.preset-manager-copy p {
  margin: 0;
  color: var(--color-muted);
  font-size: var(--font-12);
  overflow-wrap: anywhere;
}

.preset-current,
.preset-builtin {
  flex-shrink: 0;
  padding: 0.125rem 0.375rem;
  border-radius: var(--radius-4);
  font-size: var(--font-11);
  background: var(--color-inset);
  color: var(--color-muted);
}

.preset-current {
  color: var(--color-accent-strong);
}

.preset-manager-actions {
  display: grid;
  grid-template-columns: 5rem repeat(3, var(--height-button));
  gap: var(--space-4);
}

.preset-action-default {
  grid-column: 1;
}

.preset-action-edit {
  grid-column: 2;
}

.preset-action-copy {
  grid-column: 3;
}

.preset-action-delete {
  grid-column: 4;
}

.preset-manager-empty {
  padding: 2rem 0;
  text-align: center;
  color: var(--color-muted);
  font-size: var(--font-13);
}

.preset-manager-note {
  margin: 0;
  color: var(--color-muted);
  font-size: var(--font-12);
}

@media (width <= 600px) {
  .preset-manager-row {
    grid-template-columns: minmax(0, 1fr);
    gap: var(--space-4);
  }

  .preset-manager-actions {
    justify-self: end;
    gap: 0;
  }
}
</style>
