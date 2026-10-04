import { call } from './core.ts';

export const settingsApi = {
  all: () => call<Record<string, unknown>>('settings_all'),
  set: (key: string, value: unknown) => call<void>('settings_set', { key, value }),
};
