// The open vault. `ready` turns true once launch has either reopened a vault or given up.
import { vaultApi } from '#lib/api/vault.ts';
import { settings } from './settings.svelte.ts';
import { tagPool } from './tags.svelte.ts';

class VaultStore {
  path = $state<string | null>(null);
  ready = $state(false);

  /** Reopens the last vault. Returns false when the start page should show. */
  restore = async (): Promise<boolean> => {
    try {
      this.path = await vaultApi.restore();
      if (this.path) await this.#loadAll();
      return this.path !== null;
    } catch (e) {
      // A vault that can't be reopened sends the user to the start page to pick one.
      console.error('Could not reopen the last vault', e);
      this.path = null;
      return false;
    } finally {
      this.ready = true;
    }
  };

  /** Called once the start page has opened a vault. */
  opened = async (path: string) => {
    this.path = path;
    await this.#loadAll();
  };

  /** Everything read once per vault. */
  #loadAll = async () => {
    await Promise.all([settings.load(), tagPool.load()]);
  };
}

export const vault = new VaultStore();
