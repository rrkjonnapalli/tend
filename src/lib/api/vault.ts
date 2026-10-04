import type { Check, Restore } from '#lib/types/vault.type.ts';
import { call } from './core.ts';

// Both may wait on macOS asking the user for folder access, so neither has a deadline.
export const vaultAPI = {
  open: (path: string) => call<Check>('vault_open', { args: { path }, timeoutMs: null }),
  restore: () => call<Restore>('vault_restore', { timeoutMs: null }),
};
