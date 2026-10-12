// Every reference, newest first, for the References list.
import { refsAPI } from '#lib/api/refs.ts';
import type { Ref } from '#lib/types/ref.type.ts';

class RefsStore {
  refs = $state<Ref[]>([]);
  /** False until the first load, so an empty list isn't shown as "Nothing saved yet". */
  loaded = $state(false);

  load = async () => {
    this.refs = await refsAPI.list();
    this.loaded = true;
  };

  byId = (id: string) => this.refs.find((r) => r.id === id);

  /** Another reference with the same link key (`except`: the one being edited). */
  withKey = (key: string, except?: string) => this.refs.find((r) => r.urlKey === key && r.id !== except);
}

export const refs = new RefsStore();
