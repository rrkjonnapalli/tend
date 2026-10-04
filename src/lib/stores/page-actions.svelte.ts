// Actions the open page offers to the command registry (toggle read/edit, set status, …).
// A page registers them while it is mounted; commands run them when they exist.
import { untrack } from 'svelte';
import type { PageAction } from '#lib/types/ui.type.ts';

type Actions = Partial<Record<PageAction, () => void>>;

class PageActions {
  #actions = $state(new Map<PageAction, () => void>());

  /** Registers actions; returns the function that removes them (use as an $effect cleanup). */
  provide = (actions: Actions) => {
    // Called from a page's $effect: reading the map here must not make that effect depend on it.
    untrack(() => this.#add(actions));
    return () => untrack(() => this.#remove(actions));
  };

  has = (id: PageAction) => this.#actions.has(id);

  run = (id: PageAction) => this.#actions.get(id)?.();

  #add(actions: Actions) {
    const next = new Map(this.#actions);
    for (const [id, fn] of Object.entries(actions)) next.set(id as PageAction, fn);
    this.#actions = next;
  }

  #remove(actions: Actions) {
    const rest = new Map(this.#actions);
    for (const id of Object.keys(actions)) rest.delete(id as PageAction);
    this.#actions = rest;
  }
}

export const pageActions = new PageActions();
