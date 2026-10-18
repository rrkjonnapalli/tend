import type { BOARD_KIND, BoardKind, DropKind, Paper } from '#lib/enums/snap.enum.ts';

export type Snap = {
  id: string;
  stackId: string | null;
  text: string;
  paper: Paper;
  /** A sticky's picked colour; `null` on other papers. */
  color: string | null;
  pinned: boolean;
  tags: string[];
  createdAt: number;
  updatedAt: number;
};

export type Stack = {
  id: string;
  parentId: string | null;
  /** Free text; empty shows as "Untitled stack". */
  name: string;
  updatedAt: number;
};

export type Board = { snaps: Snap[]; stacks: Stack[] };

/** A new untitled stack a saved snap starts, inside `parent` (null: on the board), in the same
 *  write as the snap. */
export type StartStack = { parent: string | null };

export type NewSnap = Pick<Snap, 'text' | 'stackId' | 'paper' | 'color' | 'pinned'> & { startStack?: StartStack };

/** Fields an edit can change; `stackId: null` puts it loose on the board. */
export type SnapPatch = Partial<NewSnap>;

/** What the snap editor hands back to save. `stackId` NEW_STACK (util/snaps.ts): a new stack
 *  where the board is. */
export type SnapDraft = {
  text: string;
  paper: Paper;
  color: string;
  stackId: string | null;
  pinned: boolean;
  tags: string[];
};

/** What the board shows, in order: a snap or a stack. */
export type BoardItem = { kind: typeof BOARD_KIND.snap; snap: Snap } | { kind: typeof BOARD_KIND.stack; stack: Stack };

/** One snap or stack on the board, by id: what is dragged. */
export type BoardRef = { kind: BoardKind; id: string };

/** Where a drag can end; `id: null` is the board itself (the path's first step). */
export type DropTarget = { kind: DropKind; id: string | null };

/** What the board shows: the open stack (none is the board) and the search needle. */
export type BoardView = { open: string | null; needle: string };

/** Asks whether `id` is `ancestor` or sits inside it. */
export type Nesting = { id: string | null; ancestor: string };

/** The board's columns: at least `minWidth` px wide, `gap` px apart. */
export type BoardColumn = { minWidth: number; gap: number };

/** Items to put in `count` columns, by each one's height. */
export type Packing<T> = { items: T[]; count: number; heightOf: (item: T) => number };

/** An item in its column, with its place in the full order. */
export type Placed<T> = { item: T; at: number };

/** The open snap editor: `snap` null writes a new one, with `filled` (text from Home) in. */
export type SnapEditing = { snap: Snap | null; filled?: string };

/** One line of the editor's stack picker. `stackId` is where the snap goes: a stack, null for the
 *  board, or NEW_STACK. */
export type StackChoice = { id: string; stackId: string | null; label: string };

/** What saving a draft gave: the stack it went in, and the tags' refusal if they were refused. */
export type SavedDraft = { stackId: string | null; tagsError: unknown };
