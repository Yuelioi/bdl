export type SettingsSectionId =
  | 'settings-download'
  | 'settings-media'
  | 'settings-presets'
  | 'settings-update'
  | 'settings-maintenance';

export interface SettingsSection {
  id: SettingsSectionId;
  label: string;
  compactLabel: string;
  description: string;
  icon: string;
}
