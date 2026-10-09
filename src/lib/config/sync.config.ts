// Sync choices offered in the UI.
import type { SyncSetup } from '#lib/types/sync.type.ts';

/** Minutes between automatic syncs: 0 is off, at most a day (matches the backend). */
export const intervalLimits = { min: 0, max: 24 * 60 } as const;

export const emptySetup = (): SyncSetup => ({ url: '', token: '', branch: '', username: '' });

/** Shown wherever a remote is chosen or shown (spec §14). */
export const publicRepoWarning = '⚠ Beware of public repos: anything in your vault, private content included, is pushed there.';
