// Actions the open page offers to the command registry (toggle read/edit, set status, …).
// A page registers them while it is mounted; commands run them when they exist.
import { untrack } from 'svelte';

export type PageAction = 'note.mode' | 'note.tag' | 'page.width' | 'task.status' | 'task.due' | 'task.myday';

class PageActions {
  #actions = $state(new Map<PageAction, () => void>());

  /** Registers actions; returns the function that removes them (use as an $effect cleanup). */
  provide = (actions: Partial<Record<PageAction, () => void>>) => {
    // Called from a page's $effect: reading the map here must not make that effect depend on it.
    untrack(() => {
      const next = new Map(this.#actions);
      for (const [k, fn] of Object.entries(actions)) next.set(k as PageAction, fn);
      this.#actions = next;
    });
    return () =>
      untrack(() => {
        const rest = new Map(this.#actions);
        for (const k of Object.keys(actions)) rest.delete(k as PageAction);
        this.#actions = rest;
      });
  };

  has = (id: PageAction) => this.#actions.has(id);

  run = (id: PageAction) => this.#actions.get(id)?.();
}

export const pageActions = new PageActions();
