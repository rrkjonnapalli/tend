import type { ErrorDef } from './error.type.ts';

/** A vault's sync settings, kept in the app config folder. The token is never part of them. */
export type SyncConfig = {
  url: string;
  branch: string;
  username: string | null;
  authorName: string;
  authorEmail: string;
  /** Minutes between automatic syncs; 0 is off. */
  intervalMinutes: number;
};

/** What the runner reports while it works. */
export type SyncLive = {
  running: boolean;
  /** Why the last run in this session failed; null after a success. */
  error: ErrorDef | null;
};

export type SyncStatus = SyncLive & {
  /** Null when the vault isn't connected. */
  config: SyncConfig | null;
  hasToken: boolean;
  lastRunAt: number | null;
  /** `ok`, or the error key of the last run (kept across restarts). */
  lastResult: string | null;
  /** The last run that worked. */
  lastOkAt: number | null;
};

/** What the user types to connect or restore. Empty branch or username takes the default. */
export type SyncSetup = { url: string; token: string; branch: string; username: string };

/** How the dock icon shows sync. */
export type SyncState = 'off' | 'running' | 'failed' | 'synced';
