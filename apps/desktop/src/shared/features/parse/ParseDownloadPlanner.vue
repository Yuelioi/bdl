<script setup lang="ts">
import { commandDiagnostic } from '../../utils/commandDiagnostic';
import ErrorDetails from '../../ui/ErrorDetails.vue';
import { open } from '@tauri-apps/plugin-dialog';
import { computed, ref, watch } from 'vue';

import type { SettingsSnapshot, DuplicateTaskMatch, DuplicateTaskPolicy, DocumentTreeDirectory } from '../../api/dto';
import { mobilePickExportDirectory } from '../../api/tauri';
import { useParseStore } from '../../stores/parse';
import { validateNamingTemplate, useSettingsStore } from '../../stores/settings';
import type { InlineNotice } from '../../stores/feedback';
import { statusBadge, statusLabel } from '../../stores/transferView';
import { duplicateNamingOptions } from '../settings/settingsCatalog';
import NamingPresetSelect from '../settings/NamingPresetSelect.vue';
import DownloadPresetDialog from '../settings/DownloadPresetDialog.vue';
import { useUiStore } from '../../stores/ui';
import PresetSelectActions from '../settings/PresetSelectActions.vue';
import { isAndroidPlatform, isMobilePlatform } from '../../platform/environment';
import UiCheckbox from '../../ui/Checkbox.vue';
import UiButton from '../../ui/Button.vue';
import UiDialog from '../../ui/Dialog.vue';
import UiIconButton from '../../ui/IconButton.vue';
import UiInlineNotice from '../../ui/InlineNotice.vue';
import UiSelect from '../../ui/Select.vue';
import UiStatusBadge from '../../ui/StatusBadge.vue';
import UiTextField from '../../ui/TextField.vue';
import { workflowError, workflowExtension, workflowOutputs } from '../../utils/downloadWorkflow';

const parse = useParseStore();
const settings = useSettingsStore();
const { inline = false } = defineProps<{ inline?: boolean }>();
const preparing = ref(false);
const submitting = ref(false);
let dialogTrigger: HTMLElement | null = null;
const rememberTrigger = () => {
  dialogTrigger = document.activeElement instanceof HTMLElement ? document.activeElement : null;
};
const restoreTrigger = (event: Event) => {
  if (!downloadDialogOpen.value && !duplicateDialogOpen.value && dialogTrigger?.isConnected) {
    event.preventDefault();
    dialogTrigger.focus();
  }
};
const downloadDialogOpen = ref(false);
const duplicateDialogOpen = ref(false);
const duplicateMatches = ref<DuplicateTaskMatch[]>([]);
const duplicatePendingSourceIds = ref<string[]>([]);
const downloadNotice = ref<InlineNotice | null>(null);
const downloadDir = ref('');
const documentTreeOutput = ref<DocumentTreeDirectory | null>(null);
const rememberDocumentTreeOutput = ref(false);
const downloadPresetId = ref('');
const selectedPreset = computed(() =>
  settings.saved.download_presets.find((preset) => preset.id === downloadPresetId.value),
);
const effectiveOutputExtension = computed(() =>
  selectedPreset.value ? workflowExtension(selectedPreset.value.workflow) : 'mp4',
);
const outputPreview = computed(() => (selectedPreset.value ? workflowOutputs(selectedPreset.value.workflow) : []));
const defaultsRevision = ref(0);
const namingTemplate = ref('');
const duplicateNamingStrategy = ref<SettingsSnapshot['duplicate_naming_strategy']>('skip_existing');
const namingError = computed(() => validateNamingTemplate(namingTemplate.value));
const downloadPresetOpen = ref(false);
const downloadPresetMode = ref<'create' | 'edit' | 'copy'>('create');
const openDownloadPreset = (mode: 'create' | 'edit' | 'copy' = 'create') => {
  downloadPresetMode.value = mode;
  downloadPresetOpen.value = true;
};
const ui = useUiStore();
const managePresets = () => {
  downloadDialogOpen.value = false;
  ui.openPresetSettings('download');
};
const sizeEstimate = ref<{ estimated_bytes: number; estimated_parts: number; unknown_streams: number } | null>(null);
const sizeEstimateLoading = ref(false);
const sizeEstimateError = ref('');
let sizeEstimateRevision = 0;

const androidPlatform = isAndroidPlatform();
const activeSource = computed(() => parse.activeSource);
const selectedSourceIds = computed(() =>
  parse.isBatch
    ? parse.selectedSourceIds
    : activeSource.value && parse.activeSelection.length > 0
      ? [activeSource.value.source.id]
      : [],
);
const selectedCount = computed(() =>
  parse.isBatch ? parse.selectedBatchEntryIds.length : parse.activeSelection.length,
);
const savedPreset = computed(() =>
  settings.saved.download_presets.find((preset) => preset.id === settings.saved.selected_download_preset),
);
const savedOutput = computed(() =>
  savedPreset.value ? workflowOutputs(savedPreset.value.workflow).join('、') : '请选择下载预设',
);
const savedDirectory = computed(() =>
  isMobilePlatform()
    ? settings.saved.document_tree_output?.display_name || '选择导出目录'
    : settings.saved.download_dir || 'downloads',
);
const downloadBusy = computed(() => preparing.value || submitting.value || activeLoading.value || settings.loading);
const selectionUnit = computed(() => (parse.isBatch ? '个视频' : '个分集'));
const selectionTitle = computed(() => (parse.isBatch ? '批量链接' : (activeSource.value?.source.title ?? '')));
const activeLoading = computed(() => selectedSourceIds.value.some((sourceId) => parse.loadingBySource[sourceId]));
watch(downloadPresetId, (id) => {
  if (downloadDialogOpen.value && id && id !== settings.saved.selected_download_preset)
    void settings.saveAppPreferences({ selected_download_preset: id });
});
watch(
  () => settings.saved.download_presets,
  (items) => {
    if (downloadDialogOpen.value && !items.some((p) => p.id === downloadPresetId.value))
      downloadPresetId.value = settings.saved.selected_download_preset;
  },
);
watch(
  () => settings.saved.naming_template,
  (value, previous) => {
    if (downloadDialogOpen.value && namingTemplate.value === previous) namingTemplate.value = value;
  },
);
watch(namingTemplate, (template) => {
  if (downloadDialogOpen.value && template !== settings.saved.naming_template && !validateNamingTemplate(template))
    void settings.saveAppPreferences({ naming_template: template });
});
const presetError = computed(() => {
  const flow = selectedPreset.value?.workflow;
  if (!flow) return '请选择已保存的下载预设';
  if (androidPlatform && ((flow.cover.enabled && flow.cover.embed) || (flow.subtitles.enabled && flow.subtitles.embed)))
    return 'Android 暂不支持此预设中的嵌入，请到设置中改为保存独立文件';
  return workflowError(flow);
});
const duplicatePreview = computed(() => duplicateMatches.value.slice(0, 6));
const duplicateRemaining = computed(() => Math.max(duplicateMatches.value.length - duplicatePreview.value.length, 0));
const formatEstimatedBytes = (bytes: number) => {
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  let value = Math.max(0, bytes);
  let unitIndex = 0;
  while (value >= 1024 && unitIndex < units.length - 1) {
    value /= 1024;
    unitIndex += 1;
  }
  const digits = unitIndex === 0 || value >= 100 ? 0 : value >= 10 ? 1 : 2;
  return `${value.toFixed(digits)} ${units[unitIndex]}`;
};
const sizeEstimateLabel = computed(() => {
  if (sizeEstimateLoading.value && !sizeEstimate.value) return '正在估算下载大小…';
  if (!sizeEstimate.value) return '预计大小暂不可用';
  if (sizeEstimate.value.estimated_bytes === 0 && sizeEstimate.value.unknown_streams > 0) {
    return '预计大小暂不可用';
  }
  const unknown = sizeEstimate.value.unknown_streams > 0 ? ' + 部分未知' : '';
  return `预计下载 ${formatEstimatedBytes(sizeEstimate.value.estimated_bytes)}${unknown}`;
});
const refreshSizeEstimate = async () => {
  if (!downloadDialogOpen.value || selectedSourceIds.value.length === 0) return;
  const revision = ++sizeEstimateRevision;
  sizeEstimateLoading.value = true;
  sizeEstimateError.value = '';
  try {
    const estimate = await parse.estimateDownloadSizeForSources(selectedSourceIds.value, {
      downloadPresetId: downloadPresetId.value,
    });
    if (revision === sizeEstimateRevision) sizeEstimate.value = estimate;
  } catch (error) {
    if (revision === sizeEstimateRevision) {
      sizeEstimate.value = null;
      sizeEstimateError.value = commandDiagnostic(error, `操作: 估算下载大小\n来源: ${selectionTitle.value}`);
    }
  } finally {
    if (revision === sizeEstimateRevision) sizeEstimateLoading.value = false;
  }
};
watch(
  [downloadDialogOpen, downloadPresetId, () => selectedPreset.value?.workflow],
  () => {
    if (downloadDialogOpen.value) void refreshSizeEstimate();
  },
  { deep: true },
);
const restoreDefaults = () => {
  defaultsRevision.value += 1;
  const defaults = settings.saved;
  downloadDir.value = defaults.download_dir ?? '';
  documentTreeOutput.value = defaults.document_tree_output ? { ...defaults.document_tree_output } : null;
  rememberDocumentTreeOutput.value = false;
  downloadPresetId.value = defaults.selected_download_preset || defaults.download_presets[0]?.id || '';
  namingTemplate.value = defaults.naming_template;
  duplicateNamingStrategy.value = defaults.duplicate_naming_strategy;
};

const prepareOptions = async () => {
  if (selectedSourceIds.value.length === 0 || selectedCount.value === 0) {
    parse.setNotice('请选择要下载的内容', 'warning');
    return false;
  }

  await settings.ensureLoaded();
  if (!settings.loaded) {
    parse.setNotice('下载设置未能读取，请到设置中重试。', 'warning');
    return false;
  }
  restoreDefaults();
  duplicateMatches.value = [];
  duplicatePendingSourceIds.value = [];
  downloadNotice.value = null;
  sizeEstimate.value = null;
  sizeEstimateError.value = '';
  return true;
};

const openDialog = async () => {
  if (preparing.value || submitting.value) return;
  rememberTrigger();
  preparing.value = true;
  try {
    if (await prepareOptions()) downloadDialogOpen.value = true;
  } finally {
    preparing.value = false;
  }
};

const updateDownloadDir = (value: string) => {
  downloadDir.value = value;
};

const submitTasks = async (duplicatePolicy: DuplicateTaskPolicy) => {
  downloadNotice.value = null;
  const sourceIds = duplicatePolicy === 'ask' ? selectedSourceIds.value : duplicatePendingSourceIds.value;
  if (sourceIds.length === 0) return;
  if (namingError.value || presetError.value) {
    downloadNotice.value = {
      message: namingError.value ?? presetError.value ?? '请检查下载设置',
      tone: 'warning',
    };
    return;
  }
  if (isMobilePlatform() && !documentTreeOutput.value) {
    downloadNotice.value = {
      message: '请选择 Android 导出目录。',
      tone: 'warning',
    };
    return;
  }
  downloadDialogOpen.value = false;
  parse.setNotice('正在加入传输队列…', 'info');
  if (isMobilePlatform() && rememberDocumentTreeOutput.value && documentTreeOutput.value) {
    void settings.saveDefaultDocumentTreeOutput({ ...documentTreeOutput.value });
  }
  const result = await parse.createTasksForSources(sourceIds, {
    downloadDir: isMobilePlatform() ? null : downloadDir.value.trim() || 'downloads',
    documentTreeOutput: isMobilePlatform() ? documentTreeOutput.value : null,
    downloadPresetId: downloadPresetId.value,
    namingTemplate: namingTemplate.value,
    duplicateNamingStrategy: duplicateNamingStrategy.value,
    duplicatePolicy,
  });
  if (result?.failures.length) {
    const firstFailure = result.failures[0];
    downloadNotice.value = {
      message:
        result.failures.length === 1
          ? firstFailure.message
          : `${result.failures.length} 个来源创建任务失败：${firstFailure.message}`,
      tone: result.created.length > 0 ? 'warning' : 'danger',
      detail: result.failures.map((failure) => failure.detail ?? failure.message).join('\n\n'),
    };
    downloadDialogOpen.value = true;
    return;
  }
  if (result?.requires_confirmation) {
    duplicateMatches.value = result.duplicates;
    duplicatePendingSourceIds.value = result.pendingSourceIds;
    downloadDialogOpen.value = false;
    duplicateDialogOpen.value = true;
    return;
  }
  if (!result) {
    downloadDialogOpen.value = true;
    return;
  }
  duplicateDialogOpen.value = false;
  duplicateMatches.value = [];
  duplicatePendingSourceIds.value = [];
};

const createTasks = async (duplicatePolicy: DuplicateTaskPolicy = 'ask') => {
  if (submitting.value) return;
  submitting.value = true;
  try {
    await submitTasks(duplicatePolicy);
  } finally {
    submitting.value = false;
  }
};

const downloadSaved = async () => {
  if (downloadBusy.value) return;
  rememberTrigger();
  preparing.value = true;
  try {
    if (!(await prepareOptions())) return;
    if (presetError.value || namingError.value || (isMobilePlatform() && !documentTreeOutput.value)) {
      downloadNotice.value = {
        message: presetError.value ?? namingError.value ?? '请选择 Android 导出目录。',
        tone: 'warning',
      };
      downloadDialogOpen.value = true;
      return;
    }
    await createTasks();
  } finally {
    preparing.value = false;
  }
};

const chooseDownloadDir = async () => {
  if (isMobilePlatform()) return;
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: '选择保存目录',
      defaultPath: downloadDir.value || settings.saved.download_dir || undefined,
    });
    if (typeof selected === 'string') {
      downloadDir.value = selected;
    }
  } catch (error) {
    downloadNotice.value = {
      message: error instanceof Error ? error.message : String(error),
      detail: commandDiagnostic(error, '操作: 选择下载目录'),
      tone: 'warning',
    };
  }
};

const chooseDocumentTreeOutput = async () => {
  try {
    documentTreeOutput.value = await mobilePickExportDirectory();
    if (!settings.saved.document_tree_output) rememberDocumentTreeOutput.value = true;
  } catch (error) {
    downloadNotice.value = {
      message: error instanceof Error ? error.message : String(error),
      detail: commandDiagnostic(error, '操作: 选择下载目录'),
      tone: 'warning',
    };
  }
};

defineExpose({ openDialog, downloadSaved });
</script>

<template>
  <footer
    v-if="inline"
    class="parse-download-summary"
    :class="{ 'is-mobile': isMobilePlatform() }"
    aria-label="下载所选内容"
  >
    <div class="download-summary-copy">
      <span class="download-selection"
        >已选 <strong>{{ selectedCount }}</strong> 项</span
      >
      <UButton
        class="download-summary-preset"
        variant="link"
        color="neutral"
        size="xs"
        :disabled="downloadBusy"
        :title="savedOutput"
        aria-label="调整下载设置"
        :icon="isMobilePlatform() ? undefined : 'i-tabler-adjustments-horizontal'"
        :label="isMobilePlatform() ? savedPreset?.name || '下载预设' : `${savedPreset?.name || '下载预设'} · ${savedOutput}`"
        :ui="{ label: 'truncate', leadingIcon: 'size-4' }"
        @click="openDialog"
      >
        <template #trailing><span class="summary-link">调整</span></template>
      </UButton>
      <UButton
        v-if="!isMobilePlatform()"
        class="download-summary-directory"
        variant="link"
        color="neutral"
        size="xs"
        :disabled="downloadBusy"
        :title="savedDirectory"
        aria-label="更换本次保存目录"
        icon="i-tabler-folder"
        :label="savedDirectory"
        :ui="{ label: 'truncate', leadingIcon: 'size-4' }"
        @click="openDialog"
        ><template #trailing><span class="summary-link">更换</span></template></UButton
      >
    </div>
    <UiButton
      class="download-summary-submit"
      :aria-label="`下载所选 (${selectedCount})`"
      :disabled="downloadBusy || selectedCount === 0"
      @click="downloadSaved"
    >
      <UIcon
        :name="downloadBusy ? 'i-tabler-loader-2' : 'i-tabler-download'"
        :class="{ 'animate-spin': downloadBusy }"
        aria-hidden="true"
      />
      {{ downloadBusy ? '正在创建…' : `下载 ${selectedCount} 项` }}
    </UiButton>
  </footer>
  <UiDialog
    v-model="downloadDialogOpen"
    title="下载设置"
    :content="{ onCloseAutoFocus: restoreTrigger }"
    :description="`${selectionTitle.length > (isMobilePlatform() ? 18 : 36) ? selectionTitle.slice(0, isMobilePlatform() ? 18 : 36) + '…' : selectionTitle} · ${selectedCount} ${selectionUnit}`"
  >
    <div class="download-options-scroll">
      <UiInlineNotice v-if="downloadNotice" :tone="downloadNotice.tone" :detail="downloadNotice.detail">{{
        downloadNotice.message
      }}</UiInlineNotice>
      <ErrorDetails v-if="sizeEstimateError && !downloadNotice" :detail="sizeEstimateError" />
      <div class="grid content-start gap-4">
        <div class="grid gap-4">
          <div class="planner-preset-row">
            <UiSelect
              v-model="downloadPresetId"
              label="下载预设"
              :options="settings.saved.download_presets.map((preset) => ({ label: preset.name, value: preset.id }))"
              :disabled="activeLoading"
            />
            <PresetSelectActions
              manage-label="管理下载预设"
              :disabled="activeLoading"
              :create-disabled="settings.saved.download_presets.length >= 32"
              @create="openDownloadPreset()"
              @manage="managePresets"
            />
          </div>
          <div class="planner-output-preview" aria-label="本次输出预览">
            <strong>将生成</strong>
            <ul class="m-0 grid list-none gap-1 p-0">
              <li v-for="item in outputPreview" :key="item">{{ item }}</li>
            </ul>
          </div>
          <div v-if="isMobilePlatform()" class="directory-row">
            <UiTextField
              :model-value="documentTreeOutput?.display_name ?? ''"
              label="导出目录"
              placeholder="请选择目录"
              disabled
            />
            <UiIconButton
              icon="folder-open"
              :label="documentTreeOutput ? '更换导出目录' : '选择导出目录'"
              :disabled="activeLoading"
              @click="chooseDocumentTreeOutput"
            />
          </div>
          <div v-if="isMobilePlatform() && documentTreeOutput" class="grid gap-1">
            <UiCheckbox v-model="rememberDocumentTreeOutput" label="设为默认导出目录" />
          </div>
          <div v-if="!isMobilePlatform()" class="directory-row">
            <UiTextField
              :model-value="downloadDir"
              label="保存目录"
              placeholder="留空时使用 downloads"
              @update:model-value="updateDownloadDir"
            />
            <UiButton variant="secondary" :disabled="activeLoading" @click="chooseDownloadDir">选择</UiButton>
          </div>
          <NamingPresetSelect
            @manage="downloadDialogOpen = false"
            :key="defaultsRevision"
            v-model="namingTemplate"
            :disabled="activeLoading"
            :extension="effectiveOutputExtension"
          />
          <UiSelect v-model="duplicateNamingStrategy" label="重名处理" :options="duplicateNamingOptions">
            <template #hint>
              <UPopover :content="{ side: 'top', align: 'start', collisionPadding: 16 }">
                <UButton
                  icon="i-tabler-help-circle"
                  aria-label="重名处理说明"
                  variant="ghost"
                  color="neutral"
                  size="xs"
                  class="planner-help"
                  :ui="{ leadingIcon: 'size-3.5' }"
                />
                <template #content>
                  <p class="m-0 max-w-64 p-3 text-(length:--font-12) leading-5 text-(--color-muted)">
                    目标文件已存在时使用此策略；传输记录中的重复任务会另外提示。
                  </p>
                </template>
              </UPopover>
            </template>
          </UiSelect>
        </div>
      </div>
      <UiInlineNotice v-if="presetError || namingError" tone="danger">{{ presetError ?? namingError }}</UiInlineNotice>
    </div>
    <template #footer>
      <div class="planner-dialog-footer">
        <UiButton variant="secondary" :disabled="activeLoading" @click="downloadDialogOpen = false">取消</UiButton>
        <UiButton class="planner-restore" variant="ghost" :disabled="activeLoading" @click="restoreDefaults"
          ><span class="restore-wide">恢复默认偏好</span><span class="restore-short">恢复</span></UiButton
        >
        <span class="planner-estimate" title="按媒体轨道码率与时长估算，附加文件大小可能未知。">{{
          sizeEstimateLabel
        }}</span>
        <UiButton
          :disabled="
            downloadBusy || sizeEstimateLoading || selectedSourceIds.length === 0 || Boolean(namingError || presetError)
          "
          @click="createTasks()"
          >开始下载</UiButton
        >
      </div>
    </template>
  </UiDialog>

  <DownloadPresetDialog
    v-model="downloadPresetOpen"
    :mode="downloadPresetMode"
    :preset-id="downloadPresetId"
    @saved="
      (id) => {
        downloadPresetId = id;
      }
    "
  />
  <UiDialog v-model="duplicateDialogOpen" title="发现重复任务" :content="{ onCloseAutoFocus: restoreTrigger }">
    <section class="flex items-start gap-3">
      <span
        class="grid size-9 shrink-0 place-items-center rounded-lg border border-(--color-border) bg-(--color-accent-soft) text-(--color-accent-strong)"
        aria-hidden="true"
      >
        <UIcon class="size-4" name="i-tabler-copy" />
      </span>
      <div>
        <strong>{{ duplicateMatches.length }} 个分集已在传输记录中</strong>
        <p class="mt-1 mb-0 text-(--color-muted)">可以跳过这些分集，或创建使用独立文件名的新任务。</p>
      </div>
    </section>
    <ul
      class="m-0 max-h-60 list-none overflow-auto rounded-lg border border-(--color-border) p-0"
      aria-label="重复任务"
    >
      <li
        v-for="match in duplicatePreview"
        :key="match.proposed_task_id"
        class="flex min-h-11 items-center justify-between gap-3 border-b border-(--color-border) px-2.5 py-2 last:border-b-0"
      >
        <span class="truncate">{{ match.title }}</span>
        <UiStatusBadge :status="statusBadge(match.existing_status)">{{
          statusLabel(match.existing_status)
        }}</UiStatusBadge>
      </li>
    </ul>
    <p v-if="duplicateRemaining > 0" class="mt-1 mb-0 text-(--color-muted)">另有 {{ duplicateRemaining }} 项未展开</p>
    <template #footer>
      <UiButton variant="ghost" :disabled="activeLoading" @click="duplicateDialogOpen = false">取消</UiButton>
      <UiButton variant="secondary" :disabled="activeLoading" @click="createTasks('skip')">跳过重复项</UiButton>
      <UiButton :disabled="activeLoading" @click="createTasks('create')">仍然创建</UiButton>
    </template>
  </UiDialog>
</template>

<style scoped>
.parse-download-summary {
  display: flex;
  flex-shrink: 0;
  align-items: center;
  justify-content: space-between;
  gap: 1rem;
  padding-top: 0.75rem;
  border-top: 1px solid var(--color-border);
}

.download-summary-copy {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 0.375rem 1rem;
  min-width: 0;
  font-size: var(--font-12);
}

.download-selection {
  flex-shrink: 0;
  white-space: nowrap;
}

.download-summary-copy button {
  display: inline-flex;
  align-items: center;
  gap: 0.375rem;
  min-width: 0;
  min-height: 2rem;
  padding: 0;
  text-align: left;
}

.download-summary-copy button > span:not(.summary-link) {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.download-summary-copy button > :deep(svg) {
  width: 1rem;
  height: 1rem;
  flex-shrink: 0;
}

.download-summary-preset {
  max-width: 23rem;
}

.download-summary-directory {
  max-width: 18rem;
}

.summary-link {
  white-space: nowrap;
  flex-shrink: 0;
  color: var(--color-accent-strong);
}

.download-summary-submit {
  min-width: 0;
  min-height: 1.875rem;
  flex-shrink: 0;
  border-radius: var(--radius-6);
  padding: 0.375rem 0.75rem;
  font-size: var(--font-13);
  font-weight: 500;
}

.download-summary-submit :deep(svg) {
  width: 1rem;
  height: 1rem;
}

.parse-download-summary.is-mobile {
  gap: 0.75rem;
  padding-block: 0.5rem;
}

.is-mobile .download-summary-copy {
  flex: 1;
  flex-wrap: nowrap;
  gap: 0.5rem;
}

.is-mobile .download-summary-preset {
  flex: 1;
  max-width: none;
}

.is-mobile .download-summary-copy button {
  min-height: 1.75rem;
}

.download-options-scroll {
  min-height: 0;
  display: grid;
  align-content: start;
  gap: var(--space-16);
  overflow: auto;
  overscroll-behavior: contain;
  scrollbar-gutter: stable;
  scrollbar-width: thin;
  scrollbar-color: color-mix(in oklab, var(--color-muted) 35%, transparent) transparent;
}

.download-options-scroll::-webkit-scrollbar {
  width: 0.5rem;
}

.download-options-scroll::-webkit-scrollbar-thumb {
  border: 0.125rem solid transparent;
  border-radius: 62.4375rem;
  background: color-mix(in oklab, var(--color-muted) 35%, transparent);
  background-clip: padding-box;
}

.download-options-scroll::-webkit-scrollbar-thumb:hover {
  background-color: color-mix(in oklab, var(--color-muted) 60%, transparent);
}

.planner-preset-row {
  min-width: 0;
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  align-items: end;
  gap: var(--space-8);
}

.planner-help {
  min-height: 1.25rem;
  padding: 0.125rem;
  border-radius: var(--radius-6);
}

.planner-output-preview {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 0.5rem 0.75rem;
  border-radius: var(--radius-6);
  background: var(--color-inset);
  color: var(--color-muted);
  font-size: var(--font-12);
}

.planner-output-preview strong {
  font-weight: 400;
}

@media (width >= 640px) {
  .download-options-scroll :deep(.ui-form-field) {
    display: grid;
    grid-template-columns: 5.5rem minmax(0, 1fr);
    align-items: center;
    gap: 0.5rem;
  }

  .download-options-scroll :deep(.field-control) {
    margin-top: 0;
    min-width: 0;
  }

  .planner-output-preview {
    margin-left: 6rem;
  }
}

.directory-row {
  min-width: 0;
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  align-items: end;
  gap: var(--space-8);
}

.planner-dialog-footer {
  width: 100%;
  display: grid;
  grid-template-columns: auto auto minmax(0, 1fr) auto;
  align-items: center;
  gap: 0.5rem;
}

.planner-estimate {
  min-width: 0;
  overflow: hidden;
  color: var(--color-muted);
  font-size: var(--font-12);
  text-align: right;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.restore-short {
  display: none;
}

@media (width <= 600px) {
  .planner-dialog-footer {
    grid-template-columns: auto minmax(0, 1fr) auto;
  }

  .planner-restore {
    justify-self: start;
  }

  .planner-estimate {
    grid-row: 1;
    grid-column: 1 / -1;
    text-align: left;
  }

  .restore-wide {
    display: none;
  }

  .restore-short {
    display: inline;
  }
}
</style>
