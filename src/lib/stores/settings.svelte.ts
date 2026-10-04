// App settings from the vault's `settings` table. Read once per vault, written through on change.
import { settingsApi } from '#lib/api/settings.ts';

class Settings {
  values = $state<Record<string, unknown>>({});

  load = async () => {
    this.values = await settingsApi.all();
  };

  get<T>(key: string, fallback: T): T {
    return key in this.values ? (this.values[key] as T) : fallback;
  }

  set = async (key: string, value: unknown) => {
    this.values[key] = value;
    await settingsApi.set(key, value);
  };
}

export const settings = new Settings();
