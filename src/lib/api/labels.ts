import type { Label } from '#lib/types/label.type.ts';
import { call } from './core.ts';

export const labelsAPI = {
  list: () => call<Label[]>('labels_list'),
  /** `value` is the label's id in capitals (`DB_SCRIPT`), fixed once added. */
  add: (input: { name: string; value: string; color: string }) => call<Label>('label_add', { args: input }),
  rename: (input: { id: string; name: string }) => call<Label>('label_rename', { args: input }),
  recolor: (input: { id: string; color: string }) => call<Label>('label_recolor', { args: input }),
  reorder: (ids: string[]) => call<void>('label_reorder', { args: { ids } }),
  /** How many tasks carry the label. */
  usage: (id: string) => call<number>('label_usage', { args: { id } }),
  remove: (id: string) => call<void>('label_delete', { args: { id } }),
  /** Replaces a task's labels; returns them in the list's order. */
  setForTask: (input: { id: string; labels: string[] }) => call<string[]>('task_labels_set', { args: input }),
};
