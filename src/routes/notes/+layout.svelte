<script lang="ts">
  import type { Snippet } from 'svelte';
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import Explorer, { type ExplorerItem } from '#lib/components/Explorer.svelte';
  import { notes } from '#lib/stores/notes.svelte.ts';
  import type { Item } from '#lib/api/notes.ts';
  import { baseOf, isUnder, join, parentOf } from '#lib/util/names.ts';
  import { plural } from '#lib/util/plural.ts';
  import { routes } from '#lib/util/routes.ts';

  let { children }: { children: Snippet } = $props();

  notes.load();

  const noteId = $derived(page.params.id ?? null);
  const folder = $derived(page.url.searchParams.get('f') ?? '');
  const selected = $derived<ExplorerItem>(noteId ? { kind: 'leaf', id: noteId } : { kind: 'folder', path: folder });
  const leaves = $derived(notes.notes.map((n) => ({ id: n.id, folder: n.folder, name: n.name })));

  const toApi = (item: ExplorerItem): Item => (item.kind === 'folder' ? { kind: 'folder', path: item.path } : { kind: 'note', id: item.id });

  /** After a folder moves, a list open inside it follows it to the new path. */
  function follow(oldPath: string, newPath: string) {
    if (!noteId && isUnder(folder, oldPath)) goto(routes.notesFolder(newPath + folder.slice(oldPath.length)), { replace: true });
  }

  const actions = {
    open: (item: ExplorerItem) => goto(item.kind === 'folder' ? routes.notesFolder(item.path) : routes.note(item.id)),
    createFolder: (parent: string, name: string) => notes.createFolder(join(parent, name)),
    createLeaf: async (folder: string, name: string) => {
      const row = await notes.create(folder, name);
      goto(routes.note(row.id, true));
    },
    rename: async (item: ExplorerItem, name: string) => {
      const result = await notes.rename(toApi(item), name);
      if (item.kind === 'folder') follow(item.path, result);
    },
    move: async (item: ExplorerItem, target: string) => {
      const result = await notes.move(toApi(item), target);
      if (item.kind === 'folder') follow(item.path, result);
    },
    remove: async (item: ExplorerItem) => {
      const gone = item.kind === 'folder' ? item.path : null;
      const noteFolder = item.kind === 'leaf' ? notes.byId(item.id)?.folder ?? '' : '';
      await notes.remove(toApi(item));
      if (item.kind === 'leaf' && item.id === noteId) goto(routes.notesFolder(noteFolder), { replace: true });
      if (gone !== null && !noteId && isUnder(folder, gone)) goto(routes.notesFolder(parentOf(gone)), { replace: true });
      if (gone !== null && noteId && isUnder(notes.byId(noteId)?.folder ?? '', gone)) goto(routes.notesFolder(parentOf(gone)), { replace: true });
    },
    reload: () => notes.rescan(),
    contents: (path: string) => {
      const n = notes.under(path).length;
      const f = notes.foldersUnder(path).length;
      if (!n && !f) return 'The folder is empty.';
      return `It contains ${plural(n, 'note')}${f ? ` and ${plural(f, 'folder')}` : ''}. Everything inside is deleted from disk.`;
    },
  };
</script>

<Explorer id="notes" rootName="notes" folders={notes.folders} {leaves} {selected} {actions} />
<div class="pane">{@render children()}</div>

<style>
  .pane {
    flex: 1 1 auto;
    min-width: 0;
    display: flex;
  }
</style>
