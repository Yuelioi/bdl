<script setup lang="ts">
import UiTabs from '../../shared/ui/Tabs.vue';
import type { SettingsSectionId } from '../../shared/features/settings/settingsSections';
const emit = defineEmits<{ back: [] }>();
import SettingsLeaveDialog from '../../shared/features/settings/SettingsLeaveDialog.vue';
import UiButton from '../../shared/ui/Button.vue';
import UiInlineNotice from '../../shared/ui/InlineNotice.vue';
import SettingsDownloadSection from '../../shared/features/settings/SettingsDownloadSection.vue';
import SettingsMaintenanceSection from '../../shared/features/settings/SettingsMaintenanceSection.vue';
import SettingsMediaSection from '../../shared/features/settings/SettingsMediaSection.vue';
import PresetManager from '../../shared/features/settings/PresetManager.vue';
import SettingsUpdateSection from '../../shared/features/settings/SettingsUpdateSection.vue';

import { useUiStore } from '../../shared/stores/ui';
const ui = useUiStore();
import { useSettingsPage } from '../../shared/features/settings/useSettingsPage';
const {
  settingsLeaveOpen,
  requestSettingsLeave,
  saveAndLeave,
  discardAndLeave,
  settingsForm,
  supportsDesktopPaths,
  settings,
  settingsGlobalSpeedLimitError,
  settingsFormChanged,
  settingsEmbeddingFormatError,
  resetSettingsDraft,
  saveSettings,
  restoreAllDefaults,
  activeSettingsSection,
  availableSettingsSections,
} = useSettingsPage();
</script>
<template>
  <section class="mobile-page settings-grid">
    <SettingsLeaveDialog
      v-model="settingsLeaveOpen"
      :saving="settings.saving"
      :invalid="Boolean(settings.namingTemplateError || settings.proxySettingsError || settingsGlobalSpeedLimitError || settingsEmbeddingFormatError)"
      :error="
        settings.error || settings.proxySettingsError || settingsGlobalSpeedLimitError || settingsEmbeddingFormatError || settings.namingTemplateError
      "
      @save="saveAndLeave"
      @discard="discardAndLeave"
    />
    <section class="settings-panel">
      <div class="panel-heading settings-heading">
        <div class="settings-heading-copy">
          <button
            class="mobile-icon-button"
            type="button"
            aria-label="返回我的"
            @click="requestSettingsLeave(() => emit('back'))"
          >
            <UIcon name="i-tabler-arrow-left" />
          </button>
          <h2>设置</h2>
        </div>
        <div v-if="activeSettingsSection !== 'settings-presets' || settingsFormChanged" class="settings-actions">
          <span v-if="settingsFormChanged" class="dirty-indicator">未保存</span>
          <UDropdownMenu
            :items="[
              {
                label: '撤销修改',
                icon: 'i-tabler-arrow-back-up',
                disabled: settings.loading || settings.saving || !settingsFormChanged,
                onSelect: resetSettingsDraft,
              },
              {
                label: '恢复默认',
                icon: 'i-tabler-refresh',
                disabled: settings.loading || settings.saving,
                onSelect: restoreAllDefaults,
              },
            ]"
          >
            <UiButton variant="ghost" aria-label="更多设置操作"><UIcon name="i-tabler-dots" class="size-4" /></UiButton>
          </UDropdownMenu>
          <UiButton
            :disabled="
              settings.loading ||
              settings.saving ||
              !settingsFormChanged ||
              Boolean(settings.namingTemplateError) ||
              Boolean(settings.proxySettingsError) ||
              Boolean(settingsGlobalSpeedLimitError) ||
              Boolean(settingsEmbeddingFormatError)
            "
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
      </div>

      <UiInlineNotice v-if="settings.notice" :tone="settings.notice.tone">
        {{ settings.notice.message }}
      </UiInlineNotice>

      <UiTabs
        :model-value="activeSettingsSection"
        :tabs="
          availableSettingsSections.map((section) => ({
            label: section.compactLabel,
            value: section.id,
          }))
        "
        stretch
        @update:model-value="activeSettingsSection = $event as SettingsSectionId"
      />
      <div class="settings-workspace">
        <div class="settings-content">
          <SettingsDownloadSection
            v-if="activeSettingsSection === 'settings-download'"
            :form="settingsForm"
            :desktop-paths="supportsDesktopPaths"
          />

          <SettingsMediaSection v-else-if="activeSettingsSection === 'settings-media'" :form="settingsForm" />

          <PresetManager
            v-else-if="activeSettingsSection === 'settings-presets'"
            v-model:kind="ui.settingsPresetKind"
          />

          <SettingsUpdateSection v-else-if="activeSettingsSection === 'settings-update'" />

          <SettingsMaintenanceSection v-else :form="settingsForm" :desktop-paths="supportsDesktopPaths" />

          <p v-if="settings.error" class="settings-error">{{ settings.error }}</p>
        </div>
      </div>
    </section>
  </section>
</template>

<style src="../../shared/features/settings/SettingsForm.css"></style>
<style scoped src="./SettingsPage.css"></style>
