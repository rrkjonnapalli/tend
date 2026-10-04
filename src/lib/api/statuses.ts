import { call } from './core.ts';

export type Status = { value: string; label: string; color: string; position: number; isDefault: boolean };

export const FIRST_STATUS = 'NOT_STARTED';
export const DONE_STATUS = 'DONE';

export const statusesApi = {
  list: () => call<Status[]>('statuses_list'),
};
