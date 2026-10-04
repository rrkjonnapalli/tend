import { call } from './core.ts';

export const settingsAPI = {
  all: () => call<Record<string, unknown>>('settings_all'),
  set: (input: { key: string; value: unknown }) => call<void>('settings_set', { args: input }),
};
