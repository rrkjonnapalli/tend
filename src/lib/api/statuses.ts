import type { Status } from '#lib/types/status.type.ts';
import { call } from './core.ts';

export const statusesAPI = {
  list: () => call<Status[]>('statuses_list'),
  add: (input: { label: string; value: string; color: string }) => call<Status>('status_add', { args: input }),
  rename: (input: { value: string; label: string }) => call<Status>('status_rename', { args: input }),
  recolor: (input: { value: string; color: string }) => call<Status>('status_recolor', { args: input }),
  reorder: (values: string[]) => call<void>('status_reorder', { args: { values } }),
  remove: (value: string) => call<void>('status_delete', { args: { value } }),
};
