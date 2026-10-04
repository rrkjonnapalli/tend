import { call } from './core.ts';

export type Timer = { id: string; title: string; endsAt: number };

export const timersApi = {
  list: () => call<Timer[]>('timers_list'),
  start: (title: string, ms: number) => call<Timer>('timer_start', { title, ms }),
  addMinute: (id: string) => call<void>('timer_add_minute', { id }),
  cancel: (id: string) => call<void>('timer_cancel', { id }),
  ring: (id: string) => call<void>('timer_ring', { id }),
  finishedWhileClosed: () => call<Timer[]>('timers_finished_while_closed'),
};
