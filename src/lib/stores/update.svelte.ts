// App updates: checked once at launch and from Settings → General; installing restarts the app.
import { getVersion } from '@tauri-apps/api/app';
import { appAPI } from '#lib/api/app.ts';
import type { AvailableUpdate } from '#lib/types/update.type.ts';
import { toast } from './toast.svelte.ts';

class UpdateStore {
  /** The running version, once read. */
  current = $state('');
  available = $state<AvailableUpdate | null>(null);
  checking = $state(false);
  installing = $state(false);
  /** A check finished in this session, so "up to date" can be said. */
  checked = $state(false);

  /** At launch: an update is announced; a failed check (often just offline) waits for the next launch or a manual check. */
  start = async () => {
    this.current = await getVersion();
    try {
      await this.check();
    } catch {
      return;
    }
    if (this.available) toast.show(`Tend ${this.available.version} is ready to install in Settings → General`);
  };

  check = async () => {
    this.checking = true;
    try {
      this.available = await appAPI.updateCheck();
      this.checked = true;
    } finally {
      this.checking = false;
    }
  };

  /** On success the app quits as usual (saving and syncing first) and opens again as the new version. */
  install = async () => {
    this.installing = true;
    try {
      const installed = await appAPI.updateInstall();
      if (!installed) this.#upToDate();
    } finally {
      this.installing = false;
    }
  };

  #upToDate() {
    this.available = null;
    this.checked = true;
    toast.show('Tend is up to date');
  }
}

export const update = new UpdateStore();
