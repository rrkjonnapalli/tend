<script lang="ts">
  // Folder list: every note under the folder (recursive), newest edit first.
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import Breadcrumbs from '#lib/components/Breadcrumbs.svelte';
  import PageBar from '#lib/components/PageBar.svelte';
  import ListTitle from '#lib/components/ListTitle.svelte';
  import ColumnHeader from '#lib/components/ColumnHeader.svelte';
  import EditButton from '#lib/components/EditButton.svelte';
  import InlineTitle from '#lib/components/InlineTitle.svelte';
  import { overflowTitle } from '#lib/actions/overflow-title.action.ts';
  import type { NoteRow } from '#lib/types/note.type.ts';
  import SearchLine from '#lib/components/SearchLine.svelte';
  import AddLine from '#lib/components/AddLine.svelte';
  import { rowNav } from '#lib/actions/row-nav.action.ts';
  import { searchNeedle, textMatches } from '#lib/util/search.ts';
  import { notes } from '#lib/stores/notes.svelte.ts';
  import { messageOf } from '#lib/errors/app-error.ts';
  import { nameError, relativeTo } from '#lib/util/names.ts';
  import { folderCrumbs, folderSummary } from '#lib/util/note-paths.ts';
  import { ago } from '#lib/util/dates.ts';
  import { routes } from '#lib/util/routes.ts';

  const folder = $derived(page.url.searchParams.get('f') ?? '');
  let query = $state('');
  const needle = $derived(searchNeedle(query));
  const inFolder = $derived(notes.under(folder));
  const rows = $derived(inFolder.filter((n) => textMatches([n.name, n.folder], needle)));
  const summary = $derived(folderSummary({ notes: inFolder.length, folders: notes.foldersUnder(folder).length }));
  const crumbs = $derived(folderCrumbs(folder));

  let draft = $state('');
  let error = $state('');

  async function create() {
    const name = draft.trim();
    if (!name) return;
    try {
      const row = await notes.create({ folder, title: name });
      draft = '';
      goto(routes.note(row.id, { edit: true }));
    } catch (e) {
      error = messageOf(e);
    }
  }

  // ---- rename in place

  /** Titles over the list columns: name | edit · modified · folder. */
  const COLUMNS = ['Name', '', 'Modified', 'Folder'];

  let editingId = $state<string | null>(null);

  function stopEditing() {
    editingId = null;
  }

  function rename(note: NoteRow, name: string) {
    stopEditing();
    notes.rename({ kind: 'note', id: note.id }, name);
  }
</script>

<main class="list-page" use:rowNav>
  <PageBar><Breadcrumbs {crumbs} /></PageBar>
  <ListTitle title={crumbs[crumbs.length - 1].label} meta={summary} />
  <SearchLine bind:value={query} label="Search notes" placeholder="Search notes" />
  <AddLine bind:value={draft} label="New note" placeholder="new note" {error} onadd={create} oninput={() => (error = '')} />
  <ColumnHeader template="var(--grid-note)" columns={COLUMNS} />
  <div class="list-scroll">
    <div class="rows">
      {#each rows as n (n.id)}
        {#if editingId === n.id}
          <div class="row editing">
            <InlineTitle value={n.name} label="Note name" check={nameError} onsave={(name) => rename(n, name)} oncancel={stopEditing} />
            <span></span>
            <span class="meta">{ago(n.modifiedAt)}</span>
            <span class="meta path" use:overflowTitle>{relativeTo(n.folder, folder)}</span>
          </div>
        {:else}
          <a class="row reveal-on-hover" data-row href={routes.note(n.id)}>
            <span class="name">{n.name}</span>
            <EditButton label="Rename note" onedit={() => (editingId = n.id)} />
            <span class="meta">{ago(n.modifiedAt)}</span>
            <span class="meta path" use:overflowTitle>{relativeTo(n.folder, folder)}</span>
          </a>
        {/if}
      {/each}
      {#if rows.length === 0}<span class="list-empty">{needle ? 'No notes match' : 'No notes here yet'}</span>{/if}
    </div>
  </div>
</main>

<style>

  .rows {
    display: flex;
    flex-direction: column;
  }

  .row {
    display: grid;
    /* name | edit · modified · path */
    grid-template-columns: var(--grid-note);
    align-items: center;
    column-gap: var(--col-gap);
    min-height: 36px;
    padding: 0 16px;
    margin: 0 -16px;
    font: var(--text-ui);
    color: var(--text);
    text-decoration: none;
  }

  .row.editing:hover {
    background: transparent;
  }

  .row:hover {
    background: var(--c-hover);
    color: var(--text);
  }

  .name {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .meta {
    font: var(--text-caption);
    color: var(--c-muted-2);
  }

  .path {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
