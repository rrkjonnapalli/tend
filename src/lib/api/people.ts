import type { ContactFields, ContactKind, Person, PersonPatch, PersonRow } from '#lib/types/person.type.ts';
import { call } from './core.ts';

export const peopleAPI = {
  list: () => call<PersonRow[]>('people_list'),
  /** Returns the new person's id. */
  create: (input: { first: string; last: string }) => call<string>('person_create', { args: input }),
  get: (id: string) => call<Person>('person_get', { args: { id } }),
  update: (input: { id: string; patch: PersonPatch }) => call<Person>('person_update', { args: input }),
  remove: (id: string) => call<void>('person_delete', { args: { id } }),
  addContact: (input: ContactFields & { person: string; kind: ContactKind }) => call<void>('contact_add', { args: input }),
  updateContact: (input: ContactFields & { id: string }) => call<void>('contact_update', { args: input }),
  removeContact: (id: string) => call<void>('contact_delete', { args: { id } }),
};
