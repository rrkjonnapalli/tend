import { call } from './core.ts';

export type StepStatus = 'found' | 'created' | 'error';
export type Step = { item: string; status: StepStatus; detail: string };
export type Check = { path: string; steps: Step[]; ok: boolean };

export const vaultApi = {
  open: (path: string) => call<Check>('vault_open', { path }),
  restore: () => call<string | null>('vault_restore'),
};
