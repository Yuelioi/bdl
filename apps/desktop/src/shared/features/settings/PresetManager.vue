<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import type { NamingPreset } from '../../api/dto';
import { builtinNamingPresets, previewTemplate, sameNamingTemplate, useSettingsStore } from '../../stores/settings';
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
      (p) => (p.template_version ?? 2) === settings.saved.naming_template_version && sameNamingTemplate(p.template, settings.saved.naming_template),
    )?.id,
);
const rows = computed(() => {
  const items =
    activeKind.value === 'download'
      ? settings.saved.download_presets.map((p) => ({
          id: p.id,
          name: p.name,
          number: undefined,
          summary: workflowOutputs(p.workflow).join(' · '),
          builtin: false,
          active: p.id === settings.saved.selected_download_preset,
        }))
      : [
          ...settings.saved.naming_presets.map((p) => ({ ...p, builtin: false, number: undefined })),
          ...builtins.map((p, index) => ({ ...p, builtin: true, number: index + 1 })),
        ].map((p) => ({
          id: p.id,
          name: p.name,
          summary: previewTemplate(p.template, settings.saved.output_extension, p.template_version),
          builtin: p.builtin,
          number: p.number,
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
      activeKind.value === 'download' ? { selected_download_preset: id } : { naming_template: preset!.template, naming_template_version: preset!.template_version ?? 2 },
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
        class="preset-search min-w-0 w-full"
        :ui="{ base: 'h-(--height-input) text-(length:--font-13)' }"
      />
      <UiButton :disabled="Boolean(deleting) || atLimit" @click="begin('create')"
        >新增预设</UiButton
      >
    </div>
    <UiInlineNotice v-if="error" tone="danger">{{ error }}</UiInlineNotice>
    <ul class="preset-manager-list" :aria-label="activeKind === 'download' ? '下载预设列表' : '命名预设列表'">
      <li v-for="row in rows" :key="row.id" role="listitem" :aria-label="row.name" class="preset-manager-row">
        <div class="preset-manager-name" :class="{ 'preset-manager-name-active': row.active }">
          <strong :title="row.name"><template v-if="row.number">{{ row.number }}. </template>{{ row.name }}</strong>
          <span v-if="row.active" class="preset-current">默认</span>
          <span v-if="row.builtin" class="preset-builtin">内置</span>
        </div>
        <div class="preset-manager-actions">
          <UiButton
            v-if="!row.active"
            size="compact"
            :aria-label="`设为默认 ${row.name}`"
            variant="secondary"
            :disabled="Boolean(deleting)"
            @click="useDefault(row.id)"
            >设为默认</UiButton
          >
          <UiIconButton
            v-if="!row.builtin"
            icon="pencil"
            :label="`编辑预设 ${row.name}`"
            :disabled="Boolean(deleting)"
            @click="begin('edit', row.id)"
          />
          <UiIconButton
            icon="copy"
            :label="`复制预设 ${row.name}`"
            :disabled="Boolean(deleting) || atLimit"
            @click="begin('copy', row.id)"
          />
          <UiIconButton
            v-if="!row.builtin"
            icon="trash"
            :label="`删除预设 ${row.name}`"
            :disabled="Boolean(deleting) || (activeKind === 'download' && settings.saved.download_presets.length <= 1)"
            @click="remove(row.id)"
          />
        </div>
        <p class="preset-manager-summary">{{ row.summary }}</p>
      </li>
      <li v-if="!rows.length" class="preset-manager-empty">没有找到匹配的预设</li>
    </ul>
    <section v-if="activeKind === 'naming'" class="naming-guide" aria-label="目录怎么生成？">
      <h3>目录怎么生成？</h3>
      <ol>
        <li><strong>分组</strong><p>UP 主在合集中设置的小节，例如「国产」。</p></li>
        <li><strong>父级</strong><p>分 P 取投稿标题，番剧取剧名，课堂取课程名。合集、系列、收藏夹取各自的名称，UP 主空间和稍后再看取列表名。</p></li>
        <li><strong>标题</strong><p>当前视频或条目的名称，例如「幕后花絮」。</p></li>
      </ol>
      <p class="naming-guide-footer">缺少的层级自动跳过。</p>
    </section>
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

.settings-section-body .preset-search {
  max-width: 20rem;
}

.settings-section-body .preset-manager-list {
  border: 1px solid var(--color-border);
  border-radius: var(--radius-8);
  padding-inline: var(--space-16);
}

.preset-manager-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  align-items: center;
  gap: var(--space-4) var(--space-8);
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

.settings-section-body .preset-manager-row {
  padding-block: var(--space-12);
}

.preset-manager-name {
  display: flex;
  min-width: 0;
  align-items: center;
  gap: var(--space-6);
}

.preset-manager-name strong {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  font-size: var(--font-13);
  font-weight: 500;
}

.preset-manager-name-active strong {
  color: var(--color-accent-strong);
  font-weight: 600;
}

.preset-manager-summary {
  grid-column: 1 / -1;
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
  display: flex;
  align-items: center;
  gap: var(--space-4);
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

.naming-guide {
  padding: var(--space-16);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-8);
  background: var(--color-inset);
  font-size: var(--font-13);
  line-height: 1.6;
}

.naming-guide h3, .naming-guide p {
  margin: 0;
}

.naming-guide h3 {
  font-size: var(--font-14);
  font-weight: 600;
}

.naming-guide ol {
  display: grid;
  gap: var(--space-8);
  margin-block: var(--space-12);
  padding-inline-start: 1.5em;
  list-style: decimal;
}

.naming-guide li::marker, .naming-guide strong {
  font-weight: 600;
}

.naming-guide p {
  color: var(--color-muted);
}

.naming-guide-footer {
  border-top: 1px solid var(--color-border);
  padding-top: var(--space-8);
}

</style>
