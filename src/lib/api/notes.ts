import { listen } from '@tauri-apps/api/event';
import { config } from '#lib/config/app.config.ts';
import type { Item, NoteDoc, NoteRow, NoteTree, Skipped } from '#lib/types/note.type.ts';
import { call } from './core.ts';

export const notesAPI = {
  tree: () => call<NoteTree>('notes_tree'),
  /** Reads every note on disk, so a big vault gets the long deadline. */
  rescan: () => call<NoteTree>('notes_rescan', { timeoutMs: config.slowCallTimeoutMs }),
  read: (id: string) => call<NoteDoc>('note_read', { args: { id } }),
  write: (input: { id: string; text: string }) => call<NoteRow>('note_write', { args: input }),
  create: (input: { folder: string; title: string }) => call<NoteRow>('note_create', { args: input }),
  /** Resolves to the created path (a taken name gets a suffix). */
  createFolder: (path: string) => call<string>('folder_create', { args: { path } }),
  /** `name` is a note's new title or a folder's new name. Returns the id (notes) or new path (folders). */
  rename: (input: { item: Item; name: string }) => call<string>('item_rename', { args: input }),
  move: (input: { item: Item; target: string }) => call<string>('item_move', { args: input }),
  remove: (item: Item) => call<void>('item_delete', { args: { item } }),
  skipped: () => call<Skipped[]>('skipped_items'),
  /** Lets scans rename names that break the rule, rescans, and returns what is still skipped. */
  fixNames: () => call<Skipped[]>('notes_fix_names'),
  /** Fires after the watcher re-indexes outside changes. */
  onChanged: (fn: () => void) => listen('notes:changed', fn),
};
