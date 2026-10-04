// Timer durations: 'mm:ss' (minutes any length, seconds 0–59) or a bare number of minutes.
import type { Duration } from '#lib/types/timer.type.ts';

const ERRORS = {
  zero: 'must be longer than 0',
  format: 'use mm:ss, e.g. 05:00',
  seconds: 'seconds must be 0–59',
};

const pad2 = (n: number) => String(n).padStart(2, '0');
const fail = (error: string): Duration => ({ ok: false, error });

export function parseDuration(text: string): Duration {
  const value = text.trim();
  const minutesOnly = value.match(/^(\d+)$/);
  if (minutesOnly) {
    const min = Number(minutesOnly[1]);
    if (min === 0) return fail(ERRORS.zero);
    return durationOf({ min, sec: 0 });
  }

  const clock = value.match(/^(\d*):(\d{1,2})$/);
  if (!clock) return fail(ERRORS.format);
  const min = Number(clock[1] || 0);
  const sec = Number(clock[2]);
  if (sec > 59) return fail(ERRORS.seconds);
  if (min === 0 && sec === 0) return fail(ERRORS.zero);
  return durationOf({ min, sec });
}

function durationOf({ min, sec }: { min: number; sec: number }): Duration {
  return { ok: true, ms: (min * 60 + sec) * 1000, label: `${min}:${pad2(sec)}` };
}

/** Milliseconds left → 'm:ss' (minutes grow past 59). */
export function countdown(ms: number): string {
  const sec = Math.max(0, Math.ceil(ms / 1000));
  return `${Math.floor(sec / 60)}:${pad2(sec % 60)}`;
}
