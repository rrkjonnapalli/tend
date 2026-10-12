import type { Paper } from '#lib/enums/snap.enum.ts';

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

export type NewSnap = Pick<Snap, 'text' | 'stackId' | 'paper' | 'color' | 'pinned'>;

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
export type BoardItem = { kind: 'snap'; snap: Snap } | { kind: 'stack'; stack: Stack };
