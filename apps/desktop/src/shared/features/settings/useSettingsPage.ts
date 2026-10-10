import { ref, onUnmounted } from 'vue';
import { computed } from 'vue';
import { settingsSections } from './settingsCatalog';
import type { SettingsSectionId } from './settingsSections';
import { useSettingsForm } from './useSettingsForm';
import { useUpdateStore } from '../../stores/update';
import { isMobilePlatform } from '../../platform/environment';
import { useUiStore } from '../../stores/ui';

export function useSettingsPage() {
  const settingsForm = useSettingsForm();
  const ui = useUiStore();
  const pendingLeave = ref<(() => void) | null>(null);
  const requestSettingsLeave = (leave: () => void) => {
    if (settingsForm.settingsFormChanged.value || settingsForm.settings.saving) pendingLeave.value = leave;
    else leave();
  };
  ui.setSettingsNavigationGuard(requestSettingsLeave);
  onUnmounted(() => ui.setSettingsNavigationGuard(null));
  const settingsLeaveOpen = computed({
    get: () => pendingLeave.value !== null,
    set: (open: boolean) => {
      if (!open) pendingLeave.value = null;
    },
  });
  const finishLeave = () => {
    const leave = pendingLeave.value;
    pendingLeave.value = null;
    leave?.();
  };
  const saveAndLeave = async () => {
    await settingsForm.saveSettings();
    if (!settingsForm.settingsFormChanged.value && !settingsForm.settings.error) finishLeave();
  };
  const discardAndLeave = () => {
    settingsForm.resetSettingsDraft();
    finishLeave();
  };
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
    currentSettingsSection,
  };
}
