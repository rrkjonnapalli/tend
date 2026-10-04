import type { PersonName } from '#lib/types/person.type.ts';

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
