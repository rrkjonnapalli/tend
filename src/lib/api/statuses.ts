import { call } from './core.ts';

export type Status = { value: string; label: string; color: string; position: number; isDefault: boolean };

export const FIRST_STATUS = 'NOT_STARTED';
export const DONE_STATUS = 'DONE';

export const statusesApi = {
  list: () => call<Status[]>('statuses_list'),
  add: (label: string, value: string, color: string) => call<Status>('status_add', { label, value, color }),
  rename: (value: string, label: string) => call<Status>('status_rename', { value, label }),
  recolor: (value: string, color: string) => call<Status>('status_recolor', { value, color }),
  reorder: (values: string[]) => call<void>('status_reorder', { values }),
  remove: (value: string) => call<void>('status_delete', { value }),
};

/** The ten swatches offered for a status colour (tokens.css --sw-*). */
export const SWATCHES: [string, string][] = [
  ['Grey', '#7d8995'],
  ['Blue', '#1259d3'],
  ['Sky', '#429ff5'],
  ['Violet', '#6f5ee4'],
  ['Lilac', '#b1b2ff'],
  ['Green', '#1e7a55'],
  ['Mint', '#62d2a2'],
  ['Orange', '#d57149'],
  ['Red', '#da0037'],
  ['Pink', '#f67280'],
];

/** "In review" → "IN_REVIEW". */
export const statusValue = (label: string) =>
  label
    .toUpperCase()
    .replace(/[^A-Z0-9]+/g, '_')
    .replace(/^[_0-9]+|_+$/g, '')
    .slice(0, 32);
