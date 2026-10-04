import { call } from './core.ts';

export type PersonRow = {
  id: string;
  firstName: string;
  lastName: string;
  nickname: string | null;
  dobYear: number | null;
  dobMonth: number | null;
  dobDay: number | null;
  phone: string | null;
  email: string | null;
};

export const peopleApi = {
  list: () => call<PersonRow[]>('people_list'),
};

/** Nickname when set, otherwise first and last name. */
export const displayName = (p: Pick<PersonRow, 'firstName' | 'lastName' | 'nickname'>) =>
  p.nickname?.trim() || [p.firstName, p.lastName].filter(Boolean).join(' ');

/** "Priya Nair" → "PN". */
export function initials(p: Pick<PersonRow, 'firstName' | 'lastName' | 'nickname'>): string {
  const parts = displayName(p).split(/\s+/).filter(Boolean);
  return ((parts[0]?.[0] ?? '') + (parts.length > 1 ? parts[parts.length - 1][0] : '')).toUpperCase();
}
