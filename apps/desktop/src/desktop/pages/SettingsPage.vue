<script setup lang="ts">
import { computed, onActivated, ref, watch, watchPostEffect } from 'vue';
import SettingsLeaveDialog from '../../shared/features/settings/SettingsLeaveDialog.vue';
import UiButton from '../../shared/ui/Button.vue';
import UiDialog from '../../shared/ui/Dialog.vue';
import UiInlineNotice from '../../shared/ui/InlineNotice.vue';
import SettingsDownloadSection from '../../shared/features/settings/SettingsDownloadSection.vue';
import SettingsMaintenanceSection from '../../shared/features/settings/SettingsMaintenanceSection.vue';
import SettingsMediaSection from '../../shared/features/settings/SettingsMediaSection.vue';
import PresetManager from '../../shared/features/settings/PresetManager.vue';
import SettingsUpdateSection from '../../shared/features/settings/SettingsUpdateSection.vue';
import SettingsSectionNav from '../components/settings/SettingsSectionNav.vue';
import SettingsMcpSection from '../components/settings/SettingsMcpSection.vue';
import SettingsCard from '../../shared/features/settings/SettingsCard.vue';
import type { SettingsSection } from '../../shared/features/settings/settingsSections';
import ClipboardPreference from '../../shared/ui/ClipboardPreference.vue';
import { useUiStore } from '../../shared/stores/ui';
import { useSettingsPage } from '../../shared/features/settings/useSettingsPage';
const ui = useUiStore();
const {
  settingsLeaveOpen,
  saveAndLeave,
  discardAndLeave,
  settingsForm,
  supportsDesktopPaths,
  settings,
  settingsGlobalSpeedLimitError,
  taskStartIntervalError,
  settingsFormChanged,
  settingsEmbeddingFormatError,
  resetSettingsDraft,
  saveSettings,
  restoreAllDefaults,
  activeSettingsSection,
  availableSettingsSections,
  currentSettingsSection,
} = useSettingsPage();

const aiSection = { id: 'settings-ai', label: 'AI 接入', group: '应用', icon: 'i-tabler-plug-connected' } as const satisfies Omit<SettingsSection, 'mobilePage'>;
const desktopSections = computed(() => [...availableSettingsSections.value, aiSection]);
const sectionLabel = computed(() => activeSettingsSection.value === aiSection.id ? aiSection.label : currentSettingsSection.value.label);

const hiddenFieldError = computed(() => {
  if (taskStartIntervalError.value && activeSettingsSection.value !== 'settings-tasks') {
    return { section: 'settings-tasks' as const, label: '调整间隔', message: taskStartIntervalError.value };
  }
  if (settingsGlobalSpeedLimitError.value && activeSettingsSection.value !== 'settings-tasks') {
    return { section: 'settings-tasks' as const, label: '调整限速', message: settingsGlobalSpeedLimitError.value };
  }
  if (settings.proxySettingsError && activeSettingsSection.value !== 'settings-network') {
    return { section: 'settings-network' as const, label: '修改代理', message: settings.proxySettingsError };
  }
  return null;
});
const content = ref<HTMLElement>();
const sectionHeading = ref<HTMLElement>();
const restoreOpen = ref(false);
const confirmRestore = () => {
  restoreOpen.value = false;
  restoreAllDefaults();
};
const hasDraftActions = computed(
  () => settingsFormChanged.value || !['settings-presets', 'settings-update', aiSection.id].includes(activeSettingsSection.value),
);
watch(activeSettingsSection, () => {
  content.value?.scrollTo({ top: 0 });
  if (ui.activeTab === 'settings') sectionHeading.value?.focus({ preventScroll: true });
}, { flush: 'post' });
const focusRequestedFfmpeg = () => {
  if (ui.activeTab !== 'settings' || !ui.settingsFocusFfmpeg) return;
  const ffmpeg = content.value?.querySelector<HTMLElement>('#settings-ffmpeg');
  if (!ffmpeg?.isConnected) return;
  ui.settingsFocusFfmpeg = false;
  ffmpeg.scrollIntoView({ block: 'start' });
  ffmpeg.focus({ preventScroll: true });
};
watchPostEffect(focusRequestedFfmpeg);
onActivated(focusRequestedFfmpeg);
</script>
<template>
  <section class="page-grid settings-grid">
    <SettingsLeaveDialog
      v-model="settingsLeaveOpen"
      :saving="settings.saving"
      :invalid="Boolean(settings.namingTemplateError || settings.proxySettingsError || settingsGlobalSpeedLimitError || taskStartIntervalError || settingsEmbeddingFormatError)"
      :error="
        settings.error || settings.proxySettingsError || settingsGlobalSpeedLimitError || taskStartIntervalError || settingsEmbeddingFormatError || settings.namingTemplateError
      "
      @save="saveAndLeave"
      @discard="discardAndLeave"
    />
    <section class="panel settings-panel">
      <UiDialog v-model="restoreOpen" title="恢复默认设置" size="compact">
        <p>这会替换未保存的修改。保存后，自定义下载和命名预设也会被默认预设替换。更新来源和自动检测会立即恢复默认。</p>
        <template #footer>
          <UiButton variant="secondary" @click="restoreOpen = false">取消</UiButton>
          <UiButton @click="confirmRestore">恢复默认</UiButton>
        </template>
      </UiDialog>
      <header class="settings-heading"><h2>设置</h2></header>

      <div class="settings-workspace">
        <SettingsSectionNav v-model="activeSettingsSection" :sections="desktopSections" />

        <div class="settings-main">
          <header class="settings-section-header">
            <h3 ref="sectionHeading" class="settings-section-heading" tabindex="-1">{{ sectionLabel }}</h3>
          </header>
          <div ref="content" class="settings-content">
            <div class="settings-section-body">
              <SettingsDownloadSection
                v-if="activeSettingsSection === 'settings-download'"
                :form="settingsForm"
                :desktop-paths="supportsDesktopPaths"
                group="saving"
              />

              <SettingsDownloadSection
                v-else-if="activeSettingsSection === 'settings-tasks'"
                :form="settingsForm"
                group="tasks"
              />

              <SettingsMediaSection v-else-if="activeSettingsSection === 'settings-media'" :form="settingsForm" />

              <PresetManager
                v-else-if="activeSettingsSection === 'settings-presets'"
                v-model:kind="ui.settingsPresetKind"
              />

              <SettingsUpdateSection v-else-if="activeSettingsSection === 'settings-update'" />

              <SettingsMcpSection v-else-if="activeSettingsSection === aiSection.id" />

              <div v-else-if="activeSettingsSection === 'settings-links'" class="settings-block">
                <SettingsCard title="剪贴板识别"><ClipboardPreference show-save-status /></SettingsCard>
                <SettingsDownloadSection :form="settingsForm" group="links" />
              </div>

              <SettingsMaintenanceSection
                v-else
                :form="settingsForm"
                :desktop-paths="supportsDesktopPaths"
                :group="activeSettingsSection === 'settings-network' ? 'network' : 'storage'"
              />
            </div>
          </div>
          <div v-if="settings.notice || hiddenFieldError || settings.error" class="settings-feedback">
            <UiInlineNotice v-if="hiddenFieldError" tone="danger" :action-label="hiddenFieldError.label" @action="activeSettingsSection = hiddenFieldError.section">
              {{ hiddenFieldError.message }}
            </UiInlineNotice>
            <UiInlineNotice v-if="settings.error" tone="danger">{{ settings.error }}</UiInlineNotice>
            <UiInlineNotice v-else-if="settings.notice" :tone="settings.notice.tone">{{ settings.notice.message }}</UiInlineNotice>
          </div>
          <footer v-if="hasDraftActions" class="settings-footer">
            <UiButton variant="ghost" :disabled="settings.loading || settings.saving" @click="restoreOpen = true">
              <UIcon name="i-tabler-refresh" class="size-4" aria-hidden="true" />
              恢复默认
            </UiButton>
            <div class="settings-actions">
              <span v-if="settingsFormChanged" class="dirty-indicator" role="status">有未保存的修改</span>
              <UiButton
                v-if="settingsFormChanged"
                variant="ghost"
                :disabled="settings.loading || settings.saving"
                @click="resetSettingsDraft"
              >撤销</UiButton>
              <UiButton
                :variant="settingsFormChanged ? 'primary' : 'secondary'"
                :disabled="settings.loading || settings.saving || !settingsFormChanged || Boolean(settings.namingTemplateError || settings.proxySettingsError || settingsGlobalSpeedLimitError || taskStartIntervalError || settingsEmbeddingFormatError)"
                @click="saveSettings"
              >
                <UIcon
                  :name="settings.saving ? 'i-tabler-loader-2' : 'i-tabler-device-floppy'"
                  class="size-4"
                  :class="{ 'animate-spin': settings.saving }"
                  aria-hidden="true"
                />
                {{ settings.saving ? '保存中' : '保存' }}
              </UiButton>
            </div>
          </footer>
        </div>
      </div>
    </section>
  </section>
</template>

<style src="../../shared/features/settings/SettingsForm.css"></style>
<style src="./SettingsPage.css"></style>
