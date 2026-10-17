import { listen } from '@tauri-apps/api/event';
import type { AvailableUpdate } from '#lib/types/update.type.ts';
import { call } from './core.ts';

/** Rust's quit signal (src-tauri/src/lib.rs `QUIT_EVENT`). */
const QUIT_EVENT = 'app:quitting';

export const appAPI = {
  /** Fires when the app is about to quit; answer with `flushed()` once saved. */
  onQuitting: (fn: () => void) => listen(QUIT_EVENT, fn),
  flushed: () => call<void>('app_flushed'),
  /** A newer release, or null. Desktop only. */
  updateCheck: () => call<AvailableUpdate | null>('app_update_check'),
  /** Downloads and installs the newest release, then the app restarts; false when there was none.
   *  A download can take a while, so no deadline. */
  updateInstall: () => call<boolean>('app_update_install', { timeoutMs: null }),
};
