// The open vault. `ready` turns true once launch has reopened a vault, found none, or failed.
import { vaultAPI } from '#lib/api/vault.ts';
import { messageOf } from '#lib/errors/app-error.ts';
import type { Restore, VaultProblem } from '#lib/types/vault.type.ts';
import { settings } from './settings.svelte.ts';
import { tagPool } from './tags.svelte.ts';
import { sync } from './sync.svelte.ts';

class VaultStore {
  path = $state<string | null>(null);
  ready = $state(false);
  problem = $state<VaultProblem | null>(null);

  /** Reopens the last vault. Waits as long as macOS takes to grant folder access. */
  restore = async () => {
    this.ready = false;
    this.problem = null;
    try {
      await this.#apply(await vaultAPI.restore());
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

  /** Records why launch failed, or loads the reopened vault. No path means no vault was remembered. */
  #apply = async (r: Restore) => {
    if (r.error) {
      this.problem = { path: r.path, message: r.error };
      return;
    }
    if (!r.path) return;
    this.path = r.path;
    await this.#loadAll();
  };

  /** Everything read once per vault. */
  #loadAll = async () => {
    await Promise.all([settings.load(), tagPool.load(), sync.load()]);
  };
}

export const vault = new VaultStore();
