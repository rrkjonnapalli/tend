// Everyone in the vault, for assignees and the People list.
import { peopleAPI } from '#lib/api/people.ts';
import type { PersonRow } from '#lib/types/person.type.ts';

class PeopleStore {
  people = $state<PersonRow[]>([]);

  load = async () => {
    this.people = await peopleAPI.list();
  };

  byId = (id: string | null) => (id ? this.people.find((p) => p.id === id) : undefined);
}

export const people = new PeopleStore();
