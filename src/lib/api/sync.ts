import { listen } from '@tauri-apps/api/event';
import type { Check } from '#lib/types/vault.type.ts';
import type { SyncLive, SyncSetup, SyncStatus } from '#lib/types/sync.type.ts';
import { call } from './core.ts';

/** Events the sync runner sends (src-tauri/src/sync/runner.rs). */
const EVENTS = { status: 'sync:status', applied: 'sync:applied' };

// Connect and restore push or clone a whole vault, so they have no deadline.
export const syncAPI = {
  status: () => call<SyncStatus>('sync_status'),
  connect: (setup: SyncSetup) => call<void>('sync_connect', { args: { setup }, timeoutMs: null }),
  /** Restores into an empty folder and opens it as the vault. */
  restore: (input: { path: string; setup: SyncSetup }) => call<Check>('sync_restore', { args: input, timeoutMs: null }),
  now: () => call<void>('sync_now'),
  setInterval: (minutes: number) => call<void>('sync_set_interval', { args: { minutes } }),
  setToken: (token: string) => call<void>('sync_token_set', { args: { token } }),
  /** Waits for a run in progress to end, which can be a whole push, so no deadline either. */
  disconnect: () => call<void>('sync_disconnect', { timeoutMs: null }),
  onStatus: (fn: (live: SyncLive) => void) => listen<SyncLive>(EVENTS.status, (e) => fn(e.payload)),
  /** Fires after a run changed rows or files, so every store reloads. */
  onApplied: (fn: () => void) => listen(EVENTS.applied, fn),
};
