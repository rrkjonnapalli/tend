// The board: every snap (pinned first, then the latest edit) and every stack.
import { snapsAPI } from '#lib/api/snaps.ts';
import type { Snap, Stack } from '#lib/types/snap.type.ts';

class SnapsStore {
  snaps = $state<Snap[]>([]);
  stacks = $state<Stack[]>([]);
  /** False until the first load, so an empty board isn't shown as empty too soon. */
  loaded = $state(false);

  load = async () => {
    const board = await snapsAPI.board();
    this.snaps = board.snaps;
    this.stacks = board.stacks;
    this.loaded = true;
  };
}

export const snaps = new SnapsStore();
