import type { Timer } from '#lib/types/timer.type.ts';
import { call } from './core.ts';

export const timersAPI = {
  list: () => call<Timer[]>('timers_list'),
  start: (input: { title: string; ms: number }) => call<Timer>('timer_start', { args: input }),
  addMinute: (id: string) => call<void>('timer_add_minute', { args: { id } }),
  cancel: (id: string) => call<void>('timer_cancel', { args: { id } }),
  ring: (id: string) => call<void>('timer_ring', { args: { id } }),
  finishedWhileClosed: () => call<Timer[]>('timers_finished_while_closed'),
};
