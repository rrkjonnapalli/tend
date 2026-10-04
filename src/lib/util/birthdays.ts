// Birthday text for the People list and the person page.
import type { PersonRow } from '#lib/types/person.type.ts';
import type { BirthdayLabel } from '#lib/types/birthday.type.ts';
import type { DOB } from '#lib/types/date.type.ts';
import { nextBirthday, parseISO, shortDate } from './dates.ts';

/** Within this many days the birthday is highlighted. */
export const SOON_DAYS = 14;

type DOBFields = Pick<PersonRow, 'dobYear' | 'dobMonth' | 'dobDay'>;

export function dobOf(p: DOBFields): DOB | null {
  if (!p.dobMonth || !p.dobDay) return null;
  return { year: p.dobYear, month: p.dobMonth, day: p.dobDay };
}

/** '9 Oct' with no year: a birthday is always the next one. */
const dayMonth = (day: string) => shortDate(day, parseISO(day));

function inDays(n: number): string {
  if (n === 0) return 'today';
  if (n === 1) return 'tomorrow';
  return `in ${n} days`;
}

/** "9 Oct · in 6 days · turns 31" (list column). */
export function birthdayLabel(dob: DOB, now = new Date()): BirthdayLabel {
  const next = nextBirthday(dob, now);
  const parts = [dayMonth(next.date), inDays(next.days)];
  if (next.age !== null) parts.push(`turns ${next.age}`);
  return { text: parts.join(' · '), soon: next.days <= SOON_DAYS, days: next.days };
}

/** The list label for a person, or null when no birthday is set. */
export function personBirthday(p: DOBFields, now = new Date()): BirthdayLabel | null {
  const dob = dobOf(p);
  if (!dob) return null;
  return birthdayLabel(dob, now);
}

/** "next birthday 9 Oct, turns 31 · in 6 days" (person page hint). */
export function birthdayHint(dob: DOB, now = new Date()): string {
  const next = nextBirthday(dob, now);
  const age = next.age !== null ? `, turns ${next.age}` : '';
  return `next birthday ${dayMonth(next.date)}${age} · ${inDays(next.days)}`;
}
