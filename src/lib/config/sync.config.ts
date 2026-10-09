// Sync choices offered in the UI.
import type { SyncSetup } from '#lib/types/sync.type.ts';

/** Minutes between automatic syncs; 0 is off (spec §15). */
export const syncIntervals = [
  { minutes: 0, label: 'Off' },
  { minutes: 15, label: 'Every 15 minutes' },
  { minutes: 60, label: 'Every hour' },
] as const;

export const emptySetup = (): SyncSetup => ({ url: '', token: '', branch: '', username: '' });

/** Shown wherever a remote is chosen or shown (spec §14). */
export const publicRepoWarning = '⚠ Beware of public repos: anything in your vault, private content included, is pushed there.';
