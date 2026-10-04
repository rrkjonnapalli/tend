// The open vault. `ready` turns true once launch has either reopened a vault or given up.
import { vaultApi } from '#lib/api/vault.ts';
import { settings } from './settings.svelte.ts';

class VaultStore {
  path = $state<string | null>(null);
  ready = $state(false);

  /** Reopens the last vault. Returns false when the start page should show. */
  restore = async (): Promise<boolean> => {
    try {
      this.path = await vaultApi.restore();
      if (this.path) await settings.load();
      return this.path !== null;
    } finally {
      this.ready = true;
    }
  };

  /** Called once the start page has opened a vault. */
  opened = async (path: string) => {
    this.path = path;
    await settings.load();
  };
}

export const vault = new VaultStore();
