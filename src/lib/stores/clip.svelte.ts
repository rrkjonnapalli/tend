// The phone's selection and cut (tend-mobile-spec.md §8): long-press starts selecting rows of
// one list (or snaps on the board); Cut keeps them until Paste here or Cancel. Only one of the two bars shows at a time.
import { notes } from './notes.svelte.ts';
import { tasks } from './tasks.svelte.ts';
import { canPaste, selectedRoots } from '#lib/util/clip.ts';
import { notesAPI } from '#lib/api/notes.ts';
import { tasksAPI } from '#lib/api/tasks.ts';
import type { ClipTree, Cut } from '#lib/types/clip.type.ts';
import { snaps } from './snaps.svelte.ts';
import { snapsAPI } from '#lib/api/snaps.ts';

/** Moves the cut rows into `target`, one at a time. A subtask can't move on its own
 *  (SUBTASK_MOVE): it moves with its parent, or stays. */
async function moveCut({ cut, target }: { cut: Cut; target: string }) {
  if (cut.tree === 'notes') {
    for (const id of cut.ids) await notesAPI.move({ item: { kind: 'note', id }, target });
  } else if (cut.tree === 'snaps') {
    // The board itself is '': no stack.
    const stackId = target || null;
    for (const id of cut.ids) await snapsAPI.update({ id, patch: { stackId } });
  } else {
    const topLevel = cut.ids.filter((id) => tasks.byId(id)?.parentId === null);
    for (const id of topLevel) await tasksAPI.move({ id, target });
  }
}

const reloads: Record<ClipTree, () => Promise<unknown>> = {
  notes: () => notes.load(),
  tasks: () => tasks.load(),
  snaps: () => snaps.load(),
};

/** Reloads the list a paste changed. */
const reloadTree = (tree: ClipTree) => reloads[tree]();

class Clip {
  /** The rows picked so far, in the list `tree`. */
  selecting = $state<{ tree: ClipTree; ids: string[] } | null>(null);
  /** What was cut, and the folder it came from (for the paste bar). */
  cut = $state<Cut | null>(null);

  /** A bar owns the bottom of the screen: the menu button hides. */
  get busy() {
    return this.selecting !== null || this.cut !== null;
  }

  start = (tree: ClipTree, id: string) => {
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

  /** Cuts one item without a selection: a snap's editor on a phone. */
  cutOne = (tree: ClipTree, id: string, from: string) => {
    this.selecting = null;
    this.cut = { tree, ids: [id], from };
  };

  clear = () => {
    this.cut = null;
  };

  /** Where each cut item is now: its folder. */
  /** Where each item is now: its folder, or its stack ('' for loose on the board). */
  #foldersOf(tree: ClipTree, ids: string[]): string[] {
    if (tree === 'notes') return ids.map((id) => notes.byId(id)?.folder ?? '');
    if (tree === 'snaps') return ids.map((id) => snaps.snaps.find((s) => s.id === id)?.stackId ?? '');
    return ids.map((id) => tasks.byId(id)?.path ?? '');
  }

  canPasteIn = (target: string) => {
    if (!this.cut) return false;
    return canPaste({ from: this.#foldersOf(this.cut.tree, this.cut.ids), target });
  };

  /** What a selection acts on: every note or snap; tasks as trees (a subtask whose parent is selected goes
   *  with it). */
  roots = (tree: ClipTree, ids: string[]): string[] => {
    if (tree !== 'tasks') return ids;
    const items = ids.flatMap((id) => {
      const task = tasks.byId(id);
      return task ? [{ id, parentId: task.parentId }] : [];
    });
    return selectedRoots(items);
  };

  /** Moves what was cut into `target`, then reloads once. */
  paste = async (target: string) => {
    const cut = this.cut;
    if (!cut) return;
    // A move that fails partway still reloads: the items moved before it show where they went.
    try {
      await moveCut({ cut, target });
    } finally {
      await reloadTree(cut.tree);
    }
    this.cut = null;
  };
}

export const clip = new Clip();
