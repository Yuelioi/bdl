<script setup lang="ts">
import { ref } from 'vue';
import type { SettingsSectionId } from '../settings/settingsSections';
const props = defineProps<{ initialSection?: SettingsSectionId }>();
const emit = defineEmits<{ back: [] }>();
import UiButton from '../../ui/Button.vue';
import UiInlineNotice from '../../ui/InlineNotice.vue';
import SettingsArchiveSection from '../settings/SettingsArchiveSection.vue';
import SettingsDownloadSection from '../settings/SettingsDownloadSection.vue';
import SettingsMaintenanceSection from '../settings/SettingsMaintenanceSection.vue';
import SettingsMediaSection from '../settings/SettingsMediaSection.vue';
import SettingsNamingSection from '../settings/SettingsNamingSection.vue';
import SettingsProcessingSection from '../settings/SettingsProcessingSection.vue';
import SettingsUpdateSection from '../settings/SettingsUpdateSection.vue';

import { useSettingsPage } from '../useSettingsPage';
const {
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
  currentSettingsSection,
} = useSettingsPage();
const sectionOpen = ref(Boolean(props.initialSection));
if (props.initialSection) activeSettingsSection.value = props.initialSection;
</script>
<template>
  <section class="mobile-page settings-grid">
    <section class="settings-panel">
      <div class="panel-heading settings-heading">
        <div class="settings-heading-copy">
          <button
            class="mobile-icon-button"
            type="button"
            :aria-label="sectionOpen ? '返回设置' : '返回我的'"
            @click="sectionOpen ? (sectionOpen = false) : emit('back')"
          >
            <UIcon name="i-tabler-arrow-left" />
          </button>
          <h2>{{ sectionOpen ? currentSettingsSection.label : '设置' }}</h2>
        </div>
        <div class="settings-actions">
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
        <p class="settings-heading-description">
          {{ sectionOpen ? currentSettingsSection.description : '按你的习惯下载和保存。' }}
        </p>
      </div>

      <UiInlineNotice v-if="settings.notice" :tone="settings.notice.tone">
        {{ settings.notice.message }}
      </UiInlineNotice>

      <div v-if="!sectionOpen" class="settings-index">
        <button
          v-for="section in availableSettingsSections"
          :key="section.id"
          type="button"
          class="settings-category"
          @click="
            activeSettingsSection = section.id;
            sectionOpen = true;
          "
        >
          <span class="settings-category-icon"><UIcon :name="section.icon" /></span
          ><span class="settings-category-copy"
            ><strong>{{ section.label }}</strong
            ><span>{{ section.description }}</span></span
          ><UIcon name="i-tabler-chevron-right" class="settings-chevron" />
        </button>
      </div>
      <div v-else class="settings-workspace">
        <div class="settings-content">
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

<style src="../settings/SettingsForm.css"></style>
<style scoped src="./SettingsPage.css"></style>
