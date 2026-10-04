import type { PersonName, PersonRow } from '#lib/types/person.type.ts';
import { textMatches } from './search.ts';

/** Nickname when set, otherwise first and last name. */
export function displayName(p: PersonName): string {
  const nickname = p.nickname?.trim();
  if (nickname) return nickname;
  return [p.firstName, p.lastName].filter(Boolean).join(' ');
}

/** "Priya Nair" → "PN"; a single word gives one letter. */
export function initials(p: PersonName): string {
  const words = displayName(p).split(/\s+/).filter(Boolean);
  if (words.length === 0) return '';
  const first = words[0][0];
  if (words.length === 1) return first.toUpperCase();
  const last = words[words.length - 1][0];
  return (first + last).toUpperCase();
}

/** "Ravi Kumar Rao" → first "Ravi", last "Kumar Rao". */
export function splitName(text: string): { first: string; last: string } {
  const [first = '', ...rest] = text.trim().split(/\s+/);
  return { first, last: rest.join(' ') };
}

/** True when the lower-case query is in the person's name, phone or email. An empty query matches everyone. */
export function personMatches(p: PersonRow, query: string): boolean {
  const fields = [displayName(p), p.firstName, p.lastName, p.phone ?? '', p.email ?? ''];
  return textMatches(fields, query);
}
