export type SettingsSectionId =
  | 'settings-download'
  | 'settings-media'
  | 'settings-naming'
  | 'settings-media-advanced'
  | 'settings-archive'
  | 'settings-update'
  | 'settings-maintenance';

export interface SettingsSection {
  id: SettingsSectionId;
  label: string;
  description: string;
  icon: string;
}
