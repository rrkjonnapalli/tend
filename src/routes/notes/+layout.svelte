<script lang="ts">
  import type { ExplorerActions, ExplorerItem } from '#lib/types/explorer.type.ts';
  import type { Item } from '#lib/types/note.type.ts';
  import type { Snippet } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import Explorer from '#lib/components/Explorer.svelte';
  import { notes } from '#lib/stores/notes.svelte.ts';
  import { nav } from '#lib/stores/history.svelte.ts';
  import { isUnder, join, parentOf } from '#lib/util/names.ts';
  import { NOTES_ROOT_NAME, folderContents } from '#lib/util/note-paths.ts';
  import { routes } from '#lib/util/routes.ts';

  let { children }: { children: Snippet } = $props();

  notes.load();

  const noteId = $derived(page.params.id ?? null);
  const folder = $derived(page.url.searchParams.get('f') ?? '');
  const selected = $derived<ExplorerItem>(noteId ? { kind: 'leaf', id: noteId } : { kind: 'folder', path: folder });
  const leaves = $derived(notes.notes.map((n) => ({ id: n.id, folder: n.folder, name: n.name })));

  const toApi = (item: ExplorerItem): Item => (item.kind === 'folder' ? { kind: 'folder', path: item.path } : { kind: 'note', id: item.id });

  function hrefOf(item: ExplorerItem): string {
    if (item.kind === 'folder') return routes.notesFolder(item.path);
    return routes.note(item.id);
  }

  /** After a folder moves, a list open inside it follows it to the new path. */
  function follow({ from, to }: { from: string; to: string }) {
    if (noteId || !isUnder(folder, from)) return;
    const moved = to + folder.slice(from.length);
    nav.replace(routes.notesFolder(moved));
  }

  async function createLeaf({ folder, name }: { folder: string; name: string }) {
    const row = await notes.create({ folder, title: name });
    goto(routes.note(row.id, { edit: true }));
  }

  async function rename(item: ExplorerItem, name: string) {
    const result = await notes.rename(toApi(item), name);
    if (item.kind === 'folder') follow({ from: item.path, to: result });
  }

  async function move(item: ExplorerItem, target: string) {
    const result = await notes.move(toApi(item), target);
    if (item.kind === 'folder') follow({ from: item.path, to: result });
  }

  /** Deleting the open note goes back to its folder. */
  async function removeNote(id: string) {
    const noteFolder = notes.byId(id)?.folder ?? '';
    await notes.remove({ kind: 'note', id });
    if (id === noteId) nav.back(routes.notesFolder(noteFolder));
  }

  /** Deleting the folder that holds the open list or note goes to its parent. */
  async function removeFolder(path: string) {
    await notes.remove({ kind: 'folder', path });
    const viewing = noteId ? (notes.byId(noteId)?.folder ?? '') : folder;
    if (isUnder(viewing, path)) nav.replace(routes.notesFolder(parentOf(path)));
  }

  async function remove(item: ExplorerItem) {
    if (item.kind === 'leaf') await removeNote(item.id);
    else await removeFolder(item.path);
  }

  function contents(path: string): string {
    return folderContents({ notes: notes.under(path).length, folders: notes.foldersUnder(path).length });
  }

  const actions: ExplorerActions = {
    open: (item) => goto(hrefOf(item)),
    createFolder: ({ parent, name }) => notes.createFolder(join(parent, name)),
    createLeaf,
    rename,
    move,
    remove,
    reload: () => notes.rescan(),
    contents,
  };
</script>

<Explorer id="notes" rootName={NOTES_ROOT_NAME} folders={notes.folders} {leaves} {selected} {actions} />
<div class="pane">{@render children()}</div>

<style>
  .pane {
    flex: 1 1 auto;
    min-width: 0;
    display: flex;
  }
</style>
