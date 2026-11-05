// The phone's selection and cut (tend-mobile-spec.md §8): long-press starts selecting rows of
// one list; Cut keeps them until Paste here or Cancel. Only one of the two bars shows at a time.
import { notes } from './notes.svelte.ts';
import { tasks } from './tasks.svelte.ts';
import { canPaste } from '#lib/util/clip.ts';
import type { Tree } from '#lib/types/ui.type.ts';

class Clip {
  /** The rows picked so far, in the list `tree`. */
  selecting = $state<{ tree: Tree; ids: string[] } | null>(null);
  /** What was cut, and the folder it came from (for the paste bar). */
  cut = $state<{ tree: Tree; ids: string[]; from: string } | null>(null);

  /** A bar owns the bottom of the screen: the menu button hides. */
  get busy() {
    return this.selecting !== null || this.cut !== null;
  }

  start = (tree: Tree, id: string) => {
    this.cut = null;
    this.selecting = { tree, ids: [id] };
  };

  toggle = (id: string) => {
    if (!this.selecting) return;
    const { ids } = this.selecting;
    const next = ids.includes(id) ? ids.filter((x) => x !== id) : [...ids, id];
    this.selecting = next.length > 0 ? { ...this.selecting, ids: next } : null;
  };

  selectAll = (ids: string[]) => {
    if (this.selecting) this.selecting = { ...this.selecting, ids };
  };

  stop = () => {
    this.selecting = null;
  };

  cutSelected = (from: string) => {
    if (!this.selecting) return;
    this.cut = { ...this.selecting, from };
    this.selecting = null;
  };

  clear = () => {
    this.cut = null;
  };

  /** Where each cut item is now: its folder. */
  #foldersOf(tree: Tree, ids: string[]): string[] {
    if (tree === 'notes') return ids.map((id) => notes.byId(id)?.folder ?? '');
    return ids.map((id) => tasks.byId(id)?.path ?? '');
  }

  canPasteIn = (target: string) => {
    if (!this.cut) return false;
    return canPaste({ from: this.#foldersOf(this.cut.tree, this.cut.ids), target });
  };

  paste = async (target: string) => {
    const cut = this.cut;
    if (!cut) return;
    for (const id of cut.ids) {
      if (cut.tree === 'notes') await notes.move({ kind: 'note', id }, target);
      else await tasks.move({ id, target });
    }
    this.cut = null;
  };
}

export const clip = new Clip();
