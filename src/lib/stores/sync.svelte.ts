// Sync for the open vault: its settings and how the last run went. A run that brought changes
// reloads every store, which also lets open pages notice items deleted elsewhere (useDeleted).
import { syncAPI } from '#lib/api/sync.ts';
import type { SyncLive, SyncSetup, SyncState, SyncStatus } from '#lib/types/sync.type.ts';
import { notes } from './notes.svelte.ts';
import { people } from './people.svelte.ts';
import { settings } from './settings.svelte.ts';
import { tagPool } from './tags.svelte.ts';
import { tasks } from './tasks.svelte.ts';
import { toast } from './toast.svelte.ts';

const OK = 'ok';

class SyncStore {
  status = $state<SyncStatus | null>(null);
  #listening = false;

  state = $derived.by((): SyncState => {
    const s = this.status;
    if (!s?.config) return 'off';
    if (s.running) return 'running';
    const failed = s.error !== null || (s.lastResult !== null && s.lastResult !== OK);
    if (failed) return 'failed';
    return 'synced';
  });

  load = async () => {
    this.status = await syncAPI.status();
    await this.#listen();
  };

  connect = async (setup: SyncSetup) => {
    await syncAPI.connect(setup);
    await this.load();
  };

  now = async () => {
    await syncAPI.now();
  };

  setInterval = async (minutes: number) => {
    await syncAPI.setInterval(minutes);
    await this.load();
  };

  setToken = async (token: string) => {
    await syncAPI.setToken(token);
    await this.load();
  };

  disconnect = async () => {
    await syncAPI.disconnect();
    await this.load();
  };

  /** Subscribes once; the store lives as long as the app. */
  async #listen() {
    if (this.#listening) return;
    this.#listening = true;
    await syncAPI.onStatus(this.#onLive);
    await syncAPI.onApplied(reloadAll);
  }

  #onLive = async (live: SyncLive) => {
    const firstFailure = live.error !== null && !this.status?.error;
    if (firstFailure) toast.show(`Sync: ${live.error?.message}`);
    if (live.running) {
      if (this.status) this.status = { ...this.status, ...live };
      return;
    }
    // Finished: read the run's time and result back.
    this.status = await syncAPI.status();
  };
}

/** Every store that shows synced rows. */
async function reloadAll() {
  await Promise.all([notes.load(), tasks.load(), people.load(), settings.load(), tagPool.load()]);
}

export const sync = new SyncStore();
