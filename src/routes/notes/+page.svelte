<script lang="ts">
  // Folder list: every note under the folder (recursive), newest edit first.
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import Breadcrumbs from '#lib/components/Breadcrumbs.svelte';
  import PageBar from '#lib/components/PageBar.svelte';
  import SearchLine from '#lib/components/SearchLine.svelte';
  import { searchNeedle, textMatches } from '#lib/util/search.ts';
  import { focusOnMount } from '#lib/actions/focus.action.ts';
  import { notes } from '#lib/stores/notes.svelte.ts';
  import { messageOf } from '#lib/errors/app-error.ts';
  import { nameError, relativeTo } from '#lib/util/names.ts';
  import { siblingNames } from '#lib/util/explorer-rows.ts';
  import { folderCrumbs, folderSummary } from '#lib/util/note-paths.ts';
  import { ago } from '#lib/util/dates.ts';
  import { routes } from '#lib/util/routes.ts';

  const folder = $derived(page.url.searchParams.get('f') ?? '');
  let query = $state('');
  const needle = $derived(searchNeedle(query));
  const inFolder = $derived(notes.under(folder));
  const rows = $derived(inFolder.filter((n) => textMatches([n.name, n.title, n.folder], needle)));
  const summary = $derived(folderSummary({ notes: inFolder.length, folders: notes.foldersUnder(folder).length }));
  const crumbs = $derived(folderCrumbs(folder));

  let draft = $state('');
  let error = $state('');
  const siblings = $derived(siblingNames({ tree: { folders: notes.folders, leaves: notes.notes }, folder }));
  const draftError = $derived(draft.trim() ? nameError(draft.trim(), siblings) : '');

  async function create() {
    const name = draft.trim();
    if (!name || draftError) return;
    try {
      const row = await notes.create({ folder, name });
      draft = '';
      goto(routes.note(row.id, { edit: true }));
    } catch (e) {
      error = messageOf(e);
    }
  }

  function onkeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') create();
  }
</script>

<main class="main">
  <PageBar meta={summary}><Breadcrumbs {crumbs} /></PageBar>
  <article class="list">
    <SearchLine bind:value={query} label="Search notes" placeholder="Search notes" />
    <div class="rows">
      <label class="add">
        <span>+</span>
        <input
          use:focusOnMount
          placeholder="new note"
          aria-label="New note"
          bind:value={draft}
          oninput={() => (error = '')}
          {onkeydown} />
      </label>
      {#if draftError || error}<span class="err">{draftError || error}</span>{/if}
      {#each rows as n (n.id)}
        <a class="row" href={routes.note(n.id)}>
          <span class="name">{n.name}</span>
          <span class="meta">{relativeTo(n.folder, folder)}</span>
          <span class="meta right">{ago(n.modifiedAt)}</span>
        </a>
      {/each}
      {#if rows.length === 0}<span class="caption muted empty">{needle ? 'No notes match' : 'No notes here yet'}</span>{/if}
    </div>
  </article>
</main>

<style>
  .main {
    flex: 1 1 auto;
    min-width: 0;
    padding: 0 24px 48px;
    overflow: auto;
  }

  .list {
    display: flex;
    flex-direction: column;
  }


  .rows {
    display: flex;
    flex-direction: column;
  }

  .add {
    display: flex;
    align-items: center;
    gap: 12px;
    min-height: 36px;
    font: 400 14px/20px var(--font-ui);
    color: var(--c-muted-2);
  }

  .add input {
    flex: 1;
    border: 0;
    outline: none;
    background: transparent;
    font: 400 14px/20px var(--font-ui);
    color: var(--text);
  }

  .add input::placeholder {
    color: var(--c-muted-2);
  }

  .err {
    padding: 0 0 8px 22px;
    font: 400 12px/16px var(--font-ui);
    color: var(--danger);
  }

  .row {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto 64px;
    align-items: center;
    gap: 12px;
    min-height: 36px;
    padding: 0 16px;
    margin: 0 -16px;
    font: 400 14px/20px var(--font-ui);
    color: var(--text);
    text-decoration: none;
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
    font: 400 13px/18px var(--font-ui);
    color: var(--c-muted-2);
  }

  .right {
    text-align: right;
  }

  .empty {
    padding-top: 8px;
  }
</style>
