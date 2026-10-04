export type StepStatus = 'found' | 'created' | 'error';
export type Step = { item: string; status: StepStatus; detail: string };
export type Check = { path: string; steps: Step[]; ok: boolean };

/** Launch result: no remembered vault (`path` null), opened, or why it couldn't be. */
export type Restore = { path: string | null; error: string | null };
