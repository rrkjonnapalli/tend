// Notices when the item an open page shows drops out of its store after a reload (deleted
// elsewhere, by sync or outside Tend) and keeps its last copy, so the page can offer to keep it.
// Nothing polls: the check runs whenever the store reloads.

type Source<T> = {
  /** The page's id, from the route. */
  id: () => string;
  /** The item as the store has it now; undefined once it is gone. */
  live: () => T | undefined;
};

export function useDeleted<T extends { id: string }>({ id, live }: Source<T>) {
  let last = $state<T | null>(null);
  let gone = $state(false);
  /** The id this page is deleting itself, so that isn't taken for a deletion elsewhere. */
  let leavingId: string | null = null;

  $effect(() => {
    const item = live();
    if (item) {
      last = item;
      gone = false;
      leavingId = null;
      return;
    }
    const wasShown = last !== null && last.id === id();
    const deletedHere = leavingId === id();
    if (wasShown && !deletedHere) gone = true;
  });

  return {
    get gone() {
      return gone;
    },
    /** The item as last seen; kept while it is gone. */
    get last() {
      return last;
    },
    /** Runs a delete made on this page. */
    async leave(remove: () => Promise<void>) {
      leavingId = id();
      try {
        await remove();
      } catch (e) {
        leavingId = null;
        throw e;
      }
    },
    /** The user kept or discarded it. */
    settle() {
      gone = false;
    },
  };
}
