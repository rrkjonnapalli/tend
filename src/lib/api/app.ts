import { listen } from '@tauri-apps/api/event';
import { call } from './core.ts';

/** Rust's quit signal (src-tauri/src/lib.rs `QUIT_EVENT`). */
const QUIT_EVENT = 'app:quitting';

export const appAPI = {
  /** Fires when the app is about to quit; answer with `flushed()` once saved. */
  onQuitting: (fn: () => void) => listen(QUIT_EVENT, fn),
  flushed: () => call<void>('app_flushed'),
};
