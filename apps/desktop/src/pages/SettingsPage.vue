<script setup lang="ts">
import { computed, ref } from 'vue';

import UiButton from '../ui/Button.vue';
import SettingsArchiveSection from './settings/SettingsArchiveSection.vue';
import SettingsDownloadSection from './settings/SettingsDownloadSection.vue';
import SettingsMaintenanceSection from './settings/SettingsMaintenanceSection.vue';
import SettingsMediaSection from './settings/SettingsMediaSection.vue';
import SettingsNamingSection from './settings/SettingsNamingSection.vue';
import SettingsProcessingSection from './settings/SettingsProcessingSection.vue';
import SettingsSectionNav from './settings/SettingsSectionNav.vue';
import { settingsSections } from './settings/settingsCatalog';
import type { SettingsSectionId } from './settings/settingsSections';
import { useSettingsForm } from './settings/useSettingsForm';

const settingsForm = useSettingsForm();
const {
  settings,
  settingsGlobalSpeedLimitError,
  settingsFormChanged,
  settingsEmbeddingFormatError,
  resetSettingsDraft,
  restoreDefaultSettings,
  saveSettings,
} = settingsForm;

const activeSettingsSection = ref<SettingsSectionId>('settings-download');
const currentSettingsSection = computed(
  () => settingsSections.find((section) => section.id === activeSettingsSection.value) ?? settingsSections[0],
);
</script>

<template>
  <section class="page-grid settings-grid">
    <section class="panel settings-panel">
      <div class="panel-heading settings-heading">
        <div class="settings-heading-copy">
          <span>偏好与维护</span>
          <h2>设置</h2>
          <p>调整新任务的默认行为，当前下载只会立即应用全局限速。</p>
        </div>
        <div class="settings-actions">
          <span v-if="settingsFormChanged" class="dirty-indicator">未保存</span>
          <UiButton variant="ghost" :disabled="settings.loading || settings.saving" @click="restoreDefaultSettings">
            恢复默认
          </UiButton>
          <UiButton
            variant="ghost"
            :disabled="settings.loading || settings.saving || !settingsFormChanged"
            @click="resetSettingsDraft"
          >
            撤销
          </UiButton>
          <UiButton
            :disabled="
              settings.loading ||
              settings.saving ||
              !settingsFormChanged ||
              Boolean(settings.namingTemplateError) ||
              Boolean(settingsGlobalSpeedLimitError) ||
              Boolean(settingsEmbeddingFormatError)
            "
            @click="saveSettings"
          >
            {{ settings.saving ? '保存中' : '保存' }}
          </UiButton>
        </div>
      </div>

      <UiInlineNotice v-if="settings.notice" :tone="settings.notice.tone">
        {{ settings.notice.message }}
      </UiInlineNotice>

      <div class="settings-workspace">
        <SettingsSectionNav v-model="activeSettingsSection" :sections="settingsSections" />

        <div class="settings-content">
          <header class="settings-section-heading">
            <span class="settings-section-icon" aria-hidden="true">
              <UIcon :name="currentSettingsSection.icon" />
            </span>
            <div>
              <h3>{{ currentSettingsSection.label }}</h3>
              <p>{{ currentSettingsSection.description }}</p>
            </div>
          </header>

          <SettingsDownloadSection v-if="activeSettingsSection === 'settings-download'" :form="settingsForm" />

          <SettingsMediaSection v-else-if="activeSettingsSection === 'settings-media'" :form="settingsForm" />

          <SettingsNamingSection v-else-if="activeSettingsSection === 'settings-naming'" :form="settingsForm" />

          <SettingsProcessingSection
            v-else-if="activeSettingsSection === 'settings-media-advanced'"
            :form="settingsForm"
          />

          <SettingsArchiveSection v-else-if="activeSettingsSection === 'settings-archive'" :form="settingsForm" />

          <SettingsMaintenanceSection v-else :form="settingsForm" />

          <p v-if="settings.error" class="settings-error">{{ settings.error }}</p>
        </div>
      </div>
    </section>
  </section>
</template>

<style src="./SettingsPage.css"></style>
