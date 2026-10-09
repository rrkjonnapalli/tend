// Words shown for timers in the bottom bar, toasts and notifications.
import type { Timer } from '#lib/types/timer.type.ts';
import { plural } from './plural.ts';

const UNTITLED = 'Timer';

/** A timer's title, or "Timer" when it has none. */
export const timerName = (timer: Pick<Timer, 'title'>) => timer.title || UNTITLED;

/** "Timer started · 10m · Tea"; the title part is left out when there is none. */
export function timerStartedText({ label, title }: { label: string; title: string }): string {
  const text = `Timer started · ${label}`;
  if (!title) return text;
  return `${text} · ${title}`;
}

/** A duration in words for the quick-create line: "5 seconds", "3 minutes", "1 minute 30 seconds". */
export function durationWords(ms: number): string {
  const total = Math.round(ms / 1000);
  const min = Math.floor(total / 60);
  const sec = total % 60;
  const parts = [];
  if (min) parts.push(plural(min, 'minute'));
  if (sec) parts.push(plural(sec, 'second'));
  return parts.join(' ');
}
