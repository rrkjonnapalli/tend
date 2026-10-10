// The notes index as the UI sees it. Reloads when the watcher reports outside changes.
import { notesAPI } from '#lib/api/notes.ts';
import { tagsAPI } from '#lib/api/tags.ts';
import type { Item, NoteRestore, NoteRow, NoteTree } from '#lib/types/note.type.ts';
import { isUnder } from '#lib/util/names.ts';

class NotesStore {
  folders = $state<string[]>([]);
  notes = $state<NoteRow[]>([]);
  /** Bumped on every reload so open pages can refresh their own data. */
  version = $state(0);
  #listening = false;

  load = async () => {
    this.#apply(await notesAPI.tree());
    await this.#listen();
  };

  rescan = async () => {
    this.#apply(await notesAPI.rescan());
  };

  #apply(tree: NoteTree) {
    this.folders = tree.folders;
    this.notes = tree.notes;
    this.version += 1;
  }

  /** Subscribes once; the store lives as long as the app, so it never unsubscribes. */
  async #listen() {
    if (this.#listening) return;
    this.#listening = true;
    await notesAPI.onChanged(() => this.load());
  }

  byId = (id: string) => this.notes.find((n) => n.id === id);

  /** Puts a row the API returned back in the list: a save moves its modified time, and the
   *  watcher doesn't reload for Tend's own saves. */
  patch = (row: NoteRow) => {
    const i = this.notes.findIndex((n) => n.id === row.id);
    if (i >= 0) this.notes[i] = row;
  };

  /** Every note in `folder` or below it, newest edit first. */
  under = (folder: string) => {
    const inside = this.notes.filter((n) => isUnder(n.folder, folder));
    return inside.sort((a, b) => b.modifiedAt - a.modifiedAt);
  };

  foldersUnder = (folder: string) => this.folders.filter((f) => f !== folder && isUnder(f, folder));

  create = async ({ folder, title }: { folder: string; title: string }) => {
    const row = await notesAPI.create({ folder, title });
    await this.load();
    return row;
  };

  /** Recreates a note deleted elsewhere as a new one (new id): same folder and file name, the
   *  text and tags the page held. */
  restore = async ({ folder, name, text, tags }: NoteRestore) => {
    const made = await notesAPI.create({ folder, title: name });
    await notesAPI.write({ id: made.id, text });
    if (tags.length > 0) await tagsAPI.setForNote({ id: made.id, tags });
    await this.load();
    return made;
  };

  createFolder = async (path: string) => {
    const created = await notesAPI.createFolder(path);
    await this.load();
    return created;
  };

  rename = async (item: Item, name: string) => {
    const result = await notesAPI.rename({ item, name });
    await this.load();
    return result;
  };

  move = async (item: Item, target: string) => {
    const result = await notesAPI.move({ item, target });
    await this.load();
    return result;
  };

  remove = async (item: Item) => {
    await notesAPI.remove(item);
    await this.load();
  };
}

export const notes = new NotesStore();
