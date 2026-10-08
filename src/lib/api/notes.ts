import { listen } from '@tauri-apps/api/event';
import { config } from '#lib/config/app.config.ts';
import type { Item, NoteDoc, NoteRow, NoteTree } from '#lib/types/note.type.ts';
import { call } from './core.ts';

export const notesAPI = {
  tree: () => call<NoteTree>('notes_tree'),
  /** Reads every note on disk, so a big vault gets the long deadline. */
  rescan: () => call<NoteTree>('notes_rescan', { timeoutMs: config.slowCallTimeoutMs }),
  read: (id: string) => call<NoteDoc>('note_read', { args: { id } }),
  write: (input: { id: string; text: string }) => call<NoteRow>('note_write', { args: input }),
  create: (input: { folder: string; name: string }) => call<NoteRow>('note_create', { args: input }),
  /** Resolves to the created path (a taken name gets a suffix). */
  createFolder: (path: string) => call<string>('folder_create', { args: { path } }),
  /** Returns the new id (notes) or path (folders). */
  rename: (input: { item: Item; name: string }) => call<string>('item_rename', { args: input }),
  move: (input: { item: Item; target: string }) => call<string>('item_move', { args: input }),
  remove: (item: Item) => call<void>('item_delete', { args: { item } }),
  skipped: () => call<string[]>('skipped_items'),
  /** Fires after the watcher re-indexes outside changes. */
  onChanged: (fn: () => void) => listen('notes:changed', fn),
};
