// The board: every snap (pinned first, then the latest edit) and every stack.
import { snapsAPI } from '#lib/api/snaps.ts';
import type { Snap, SnapEditing, Stack } from '#lib/types/snap.type.ts';
import { latestOnly } from '#lib/util/latest.ts';
import { settings } from './settings.svelte.ts';
import { toast } from './toast.svelte.ts';

class SnapsStore {
  snaps = $state<Snap[]>([]);
  stacks = $state<Stack[]>([]);
  /** False until the first load, so an empty board isn't shown as empty too soon. */
  loaded = $state(false);
  /** The open snap editor (SnapDialogs shows it, on any page): `snap` null writes a new one. */
  editing = $state<SnapEditing | null>(null);
  /** The snap whose delete is being confirmed. */
  deleting = $state<Snap | null>(null);
  /** The stack being named on the board: one just made, or one being renamed. */
  naming = $state<string | null>(null);

  load = latestOnly(
    () => snapsAPI.board(),
    (board) => {
      this.snaps = board.snaps;
      this.stacks = board.stacks;
      this.loaded = true;
    },
  );

  /** The stack `id` when it exists; none for the board, or for a stack that is gone. */
  stackOr = (id: string | null) => (this.stacks.some((s) => s.id === id) ? id : null);

  /** Opens a blank paper, unless an editor is open already (its draft stays). */
  write = () => {
    this.editing ??= { snap: null };
  };

  /** Asks first unless Settings → General turned that off for snaps. */
  askDelete = (snap: Snap) => {
    if (settings.asksDelete('snap')) this.deleting = snap;
    else this.remove(snap);
  };

  remove = async (snap: Snap) => {
    this.deleting = null;
    this.editing = null;
    await snapsAPI.remove(snap.id);
    await this.load();
    toast.show('Snap deleted');
  };
}

export const snaps = new SnapsStore();
