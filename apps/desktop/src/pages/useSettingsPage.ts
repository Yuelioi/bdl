import { ref } from 'vue';
import { computed } from 'vue';
import { settingsSections } from './settings/settingsCatalog';
import type { SettingsSectionId } from './settings/settingsSections';
import { useSettingsForm } from './settings/useSettingsForm';
import { useUpdateStore } from '../stores/update';
import { isMobilePlatform } from '../utils/platform';

export function useSettingsPage() {
  const settingsForm = useSettingsForm();
  const updater = useUpdateStore();
  const supportsDesktopPaths = !isMobilePlatform();
  const {
    settings,
    settingsGlobalSpeedLimitError,
    settingsFormChanged,
    settingsEmbeddingFormatError,
    resetSettingsDraft,
    restoreDefaultSettings,
    saveSettings,
  } = settingsForm;
  const restoreAllDefaults = () => {
    restoreDefaultSettings();
    updater.setAutoCheck(false);
  };
  const activeSettingsSection = ref<SettingsSectionId>('settings-download');
  const availableSettingsSections = computed(() =>
    updater.supported ? settingsSections : settingsSections.filter((section) => section.id !== 'settings-update'),
  );
  const currentSettingsSection = computed(
    () =>
      availableSettingsSections.value.find((section) => section.id === activeSettingsSection.value) ??
      availableSettingsSections.value[0],
  );
  return {
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
  };
}
