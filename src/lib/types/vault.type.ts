export type StepStatus = 'found' | 'created' | 'error';
export type Step = { item: string; status: StepStatus; detail: string };
/** A question asked before Tend writes into the folder. */
export type Confirm = { kind: 'notAVault'; items: number; notes: number };
/** With `confirm`, nothing was done yet: open again with `confirmed` once the user agrees. */
export type Check = { path: string; steps: Step[]; ok: boolean; confirm: Confirm | null };

/** How a vault check step shows on the start page while the steps are revealed one by one. */
export type StepState = 'ok' | 'bad' | 'run' | 'wait';

/** Launch result: no remembered vault (`path` null), opened, or why it couldn't be. */
export type Restore = { path: string | null; error: string | null };

/** Why the remembered vault couldn't be reopened, and which one it was. */
export type VaultProblem = { path: string | null; message: string };
