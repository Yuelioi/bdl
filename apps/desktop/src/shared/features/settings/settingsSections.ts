export type SettingsSectionId =
  | 'settings-download'
  | 'settings-tasks'
  | 'settings-media'
  | 'settings-presets'
  | 'settings-update'
  | 'settings-ai'
  | 'settings-links'
  | 'settings-network'
  | 'settings-maintenance';

export type SettingsDetailPage = 'saving' | 'tasks' | 'media' | 'presets' | 'links' | 'network' | 'storage' | 'update';

export const settingsGroups = ['下载', '应用'] as const;

export interface SettingsSection {
  id: SettingsSectionId;
  mobilePage: SettingsDetailPage;
  label: string;
  group: (typeof settingsGroups)[number];
  icon: string;
}
