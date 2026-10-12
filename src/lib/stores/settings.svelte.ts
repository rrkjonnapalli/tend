// App settings from the vault's `settings` table. Read once per vault, written through on change.
import { settingsAPI } from '#lib/api/settings.ts';
import { settingDefaults, settingKeys } from '#lib/config/settings.config.ts';
import type { DeleteKind } from '#lib/types/ui.type.ts';

class Settings {
  values = $state<Record<string, unknown>>({});

  load = async () => {
    this.values = await settingsAPI.all();
  };

  get<T>(key: string, fallback: T): T {
    return key in this.values ? (this.values[key] as T) : fallback;
  }

  /** Whether deleting this kind asks first (Settings → General → Confirmations). */
  asksDelete(kind: DeleteKind): boolean {
    return this.get(settingKeys.confirmDelete(kind), settingDefaults.confirmDelete);
  }

  set = async (key: string, value: unknown) => {
    this.values[key] = value;
    await settingsAPI.set({ key, value });
  };
}

export const settings = new Settings();
