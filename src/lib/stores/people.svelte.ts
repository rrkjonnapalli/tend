// Everyone in the vault, for assignees and the People list.
import { peopleApi, type PersonRow } from '#lib/api/people.ts';

class PeopleStore {
  people = $state<PersonRow[]>([]);

  load = async () => {
    this.people = await peopleApi.list();
  };

  byId = (id: string | null) => (id ? this.people.find((p) => p.id === id) : undefined);
}

export const people = new PeopleStore();
