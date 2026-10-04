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

export type ContactKind = 'phone' | 'email' | 'address';
export type Contact = { id: string; kind: ContactKind; value: string; label: string };
export type Person = PersonRow & { tob: string | null; notes: string; contacts: Contact[] };
export type PersonPatch = Partial<{
  firstName: string;
  lastName: string;
  nickname: string | null;
  dob: { year: number | null; month: number; day: number } | null;
  tob: string | null;
  notes: string;
}>;

export const peopleApi = {
  list: () => call<PersonRow[]>('people_list'),
  create: (first: string, last: string) => call<string>('person_create', { first, last }),
  get: (id: string) => call<Person>('person_get', { id }),
  update: (id: string, patch: PersonPatch) => call<Person>('person_update', { id, patch }),
  remove: (id: string) => call<void>('person_delete', { id }),
  addContact: (person: string, kind: ContactKind, value: string, label: string) =>
    call<void>('contact_add', { person, kind, value, label }),
  updateContact: (id: string, value: string, label: string) => call<void>('contact_update', { id, value, label }),
  removeContact: (id: string) => call<void>('contact_delete', { id }),
};

/** Nickname when set, otherwise first and last name. */
export const displayName = (p: Pick<PersonRow, 'firstName' | 'lastName' | 'nickname'>) =>
  p.nickname?.trim() || [p.firstName, p.lastName].filter(Boolean).join(' ');

/** "Priya Nair" → "PN". */
export function initials(p: Pick<PersonRow, 'firstName' | 'lastName' | 'nickname'>): string {
  const parts = displayName(p).split(/\s+/).filter(Boolean);
  return ((parts[0]?.[0] ?? '') + (parts.length > 1 ? parts[parts.length - 1][0] : '')).toUpperCase();
}
