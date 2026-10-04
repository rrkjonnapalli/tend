import { listen } from '@tauri-apps/api/event';
import { call } from './core.ts';

export type NoteRow = { id: string; folder: string; name: string; title: string; modifiedAt: number };
export type NoteTree = { folders: string[]; notes: NoteRow[] };
export type NoteDoc = NoteRow & { text: string; tags: string[] };
export type Item = { kind: 'note'; id: string } | { kind: 'folder'; path: string };

export const notesApi = {
  tree: () => call<NoteTree>('notes_tree'),
  rescan: () => call<NoteTree>('notes_rescan'),
  read: (id: string) => call<NoteDoc>('note_read', { id }),
  write: (id: string, text: string) => call<NoteRow>('note_write', { id, text }),
  create: (folder: string, name: string) => call<NoteRow>('note_create', { folder, name }),
  createFolder: (path: string) => call<void>('folder_create', { path }),
  /** Returns the new id (notes) or path (folders). */
  rename: (item: Item, name: string) => call<string>('item_rename', { item, name }),
  move: (item: Item, target: string) => call<string>('item_move', { item, target }),
  remove: (item: Item) => call<void>('item_delete', { item }),
  skipped: () => call<string[]>('skipped_items'),
  /** Fires after the watcher re-indexes outside changes. */
  onChanged: (fn: () => void) => listen('notes:changed', fn),
};
