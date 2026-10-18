import type { Board, NewSnap, Snap, SnapPatch, Stack } from '#lib/types/snap.type.ts';
import { call } from './core.ts';

export const snapsAPI = {
  board: () => call<Board>('snaps_board'),
  create: (snap: NewSnap) => call<Snap>('snap_create', { args: { snap } }),
  update: (input: { id: string; patch: SnapPatch }) => call<Snap>('snap_update', { args: input }),
  remove: (id: string) => call<void>('snap_delete', { args: { id } }),
  setTags: (input: { id: string; tags: string[] }) => call<string[]>('snap_tags_set', { args: input }),
  /** A new untitled stack holding both snaps, all or nothing. */
  pairStack: (input: { parent: string | null; snaps: [string, string] }) => call<Stack>('stack_pair', { args: input }),
  renameStack: (input: { id: string; name: string }) => call<Stack>('stack_rename', { args: input }),
  moveStack: (input: { id: string; parent: string | null }) => call<Stack>('stack_move', { args: input }),
  /** Its snaps and sub-stacks move up a level. */
  removeStack: (id: string) => call<void>('stack_remove', { args: { id } }),
};
