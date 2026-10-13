// The Settings sections; each is also the `?s=` value of its URL (see routes.settingsSection).
export const SETTINGS_SECTION = {
  general: 'general',
  tasks: 'tasks',
  library: 'library',
  tags: 'tags',
  sync: 'sync',
  palette: 'palette',
} as const;

export type SettingsSection = (typeof SETTINGS_SECTION)[keyof typeof SETTINGS_SECTION];
