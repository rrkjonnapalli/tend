// The open vault. `ready` turns true once launch has reopened a vault, found none, or failed.
import { vaultAPI } from '#lib/api/vault.ts';
import { messageOf } from '#lib/api/core.ts';
import { settings } from './settings.svelte.ts';
import { tagPool } from './tags.svelte.ts';

class VaultStore {
  path = $state<string | null>(null);
  ready = $state(false);
  /** Why the remembered vault couldn't be reopened, and which one it was. */
  problem = $state<{ path: string | null; message: string } | null>(null);

  /** Reopens the last vault. Waits as long as macOS takes to grant folder access. */
  restore = async () => {
    this.ready = false;
    this.problem = null;
    try {
      const r = await vaultAPI.restore();
      if (r.error) this.problem = { path: r.path, message: r.error };
      else if (r.path) {
        this.path = r.path;
        await this.#loadAll();
      }
    } catch (e) {
      this.problem = { path: null, message: messageOf(e) };
    } finally {
      this.ready = true;
    }
  };

  /** "Choose another folder" from the error screen. */
  dismiss = () => {
    this.problem = null;
    this.path = null;
  };

  /** Called once the start page has opened a vault. */
  opened = async (path: string) => {
    this.path = path;
    this.problem = null;
    await this.#loadAll();
  };

  /** Everything read once per vault. */
  #loadAll = async () => {
    await Promise.all([settings.load(), tagPool.load()]);
  };
}

export const vault = new VaultStore();
