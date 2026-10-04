// Words shown for timers in the bottom bar, toasts and notifications.
import type { Timer } from '#lib/types/timer.type.ts';

const UNTITLED = 'Timer';

/** A timer's title, or "Timer" when it has none. */
export const timerName = (timer: Pick<Timer, 'title'>) => timer.title || UNTITLED;

/** "Timer started · 10m · Tea"; the title part is left out when there is none. */
export function timerStartedText({ label, title }: { label: string; title: string }): string {
  const text = `Timer started · ${label}`;
  if (!title) return text;
  return `${text} · ${title}`;
}
