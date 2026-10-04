// Birthday text for the People list and the person page.
import type { PersonRow } from '#lib/api/people.ts';
import { nextBirthday, parseIso, shortDate, type Dob } from './dates.ts';

/** Within this many days the birthday is highlighted. */
export const SOON_DAYS = 14;

export function dobOf(p: Pick<PersonRow, 'dobYear' | 'dobMonth' | 'dobDay'>): Dob | null {
  return p.dobMonth && p.dobDay ? { year: p.dobYear, month: p.dobMonth, day: p.dobDay } : null;
}

/** '9 Oct' with no year: a birthday is always the next one. */
const dayMonth = (day: string) => shortDate(day, parseIso(day));

const inDays = (n: number) => (n === 0 ? 'today' : n === 1 ? 'tomorrow' : `in ${n} days`);

/** "9 Oct · in 6 days · turns 31" (list column). */
export function birthdayLabel(dob: Dob, now = new Date()): { text: string; soon: boolean; days: number } {
  const next = nextBirthday(dob, now);
  const parts = [dayMonth(next.date), inDays(next.days)];
  if (next.age !== null) parts.push(`turns ${next.age}`);
  return { text: parts.join(' · '), soon: next.days <= SOON_DAYS, days: next.days };
}

/** "next birthday 9 Oct, turns 31 · in 6 days" (person page hint). */
export function birthdayHint(dob: Dob, now = new Date()): string {
  const next = nextBirthday(dob, now);
  const age = next.age !== null ? `, turns ${next.age}` : '';
  return `next birthday ${dayMonth(next.date)}${age} · ${inDays(next.days)}`;
}
