<script setup lang="ts">
import UiButton from '../ui/Button.vue';
import UiInlineNotice from '../ui/InlineNotice.vue';
import SettingsArchiveSection from './settings/SettingsArchiveSection.vue';
import SettingsDownloadSection from './settings/SettingsDownloadSection.vue';
import SettingsMaintenanceSection from './settings/SettingsMaintenanceSection.vue';
import SettingsMediaSection from './settings/SettingsMediaSection.vue';
import SettingsNamingSection from './settings/SettingsNamingSection.vue';
import SettingsProcessingSection from './settings/SettingsProcessingSection.vue';
import SettingsUpdateSection from './settings/SettingsUpdateSection.vue';
import SettingsSectionNav from './settings/SettingsSectionNav.vue';
import { useSettingsPage } from "./useSettingsPage";
const { settingsForm, supportsDesktopPaths, settings, settingsGlobalSpeedLimitError, settingsFormChanged, settingsEmbeddingFormatError, resetSettingsDraft, saveSettings, restoreAllDefaults, activeSettingsSection, availableSettingsSections, currentSettingsSection } = useSettingsPage();
</script>
<template>
  <section class="page-grid settings-grid">
    <section class="panel settings-panel">
      <div class="panel-heading settings-heading">
        <div class="settings-heading-copy">
          <h2>设置</h2>
        </div>
        <div class="settings-actions">
          <span v-if="settingsFormChanged" class="dirty-indicator">未保存</span>
          <UiButton variant="ghost" :disabled="settings.loading || settings.saving" @click="restoreAllDefaults">
            <UIcon name="i-tabler-refresh" class="size-4" aria-hidden="true" />
            恢复默认
          </UiButton>
          <UiButton
            variant="ghost"
            :disabled="settings.loading || settings.saving || !settingsFormChanged"
            @click="resetSettingsDraft"
          >
            <UIcon name="i-tabler-arrow-back-up" class="size-4" aria-hidden="true" />
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

      <div class="settings-workspace">
        <SettingsSectionNav v-model="activeSettingsSection" :sections="availableSettingsSections" />

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

          <SettingsDownloadSection
            v-if="activeSettingsSection === 'settings-download'"
            :form="settingsForm"
            :desktop-paths="supportsDesktopPaths"
          />

          <SettingsMediaSection v-else-if="activeSettingsSection === 'settings-media'" :form="settingsForm" />

          <SettingsNamingSection v-else-if="activeSettingsSection === 'settings-naming'" :form="settingsForm" />

          <SettingsProcessingSection
            v-else-if="activeSettingsSection === 'settings-media-advanced'"
            :form="settingsForm"
            :desktop-paths="supportsDesktopPaths"
          />

          <SettingsArchiveSection v-else-if="activeSettingsSection === 'settings-archive'" :form="settingsForm" />

          <SettingsUpdateSection v-else-if="activeSettingsSection === 'settings-update'" />

          <SettingsMaintenanceSection v-else :form="settingsForm" :desktop-paths="supportsDesktopPaths" />

          <p v-if="settings.error" class="settings-error">{{ settings.error }}</p>
        </div>
      </div>
    </section>
  </section>
</template>

<style src="./settings/SettingsForm.css"></style>
<style src="./SettingsPage.css"></style>
