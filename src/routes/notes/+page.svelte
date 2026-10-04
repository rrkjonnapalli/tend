<script lang="ts">
  // Folder list: every note under the folder (recursive), newest edit first.
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import Breadcrumbs, { type Crumb } from '#lib/components/Breadcrumbs.svelte';
  import { notes } from '#lib/stores/notes.svelte.ts';
  import { messageOf } from '#lib/api/core.ts';
  import { baseOf, nameError, relativeTo } from '#lib/util/names.ts';
  import { ago } from '#lib/util/dates.ts';
  import { plural } from '#lib/util/plural.ts';
  import { routes } from '#lib/util/routes.ts';

  const folder = $derived(page.url.searchParams.get('f') ?? '');
  const rows = $derived(notes.under(folder));
  const subfolders = $derived(notes.foldersUnder(folder).length);

  const crumbs = $derived.by((): Crumb[] => {
    const out: Crumb[] = [{ label: 'notes', href: routes.notesFolder('') }];
    const segs = folder ? folder.split('/') : [];
    segs.forEach((s, i) => out.push({ label: s, href: routes.notesFolder(segs.slice(0, i + 1).join('/')) }));
    return out;
  });

  let draft = $state('');
  let error = $state('');
  const siblings = $derived([
    ...notes.notes.filter((n) => n.folder === folder).map((n) => n.name),
    ...notes.folders.filter((f) => f.startsWith(folder ? `${folder}/` : '') && !f.slice(folder ? folder.length + 1 : 0).includes('/')).map(baseOf),
  ]);
  const draftError = $derived(draft.trim() ? nameError(draft.trim(), siblings) : '');

  async function create() {
    const name = draft.trim();
    if (!name || draftError) return;
    try {
      const row = await notes.create(folder, name);
      draft = '';
      goto(routes.note(row.id, true));
    } catch (e) {
      error = messageOf(e);
    }
  }
</script>

<main class="main">
  <article class="list">
    <div class="head">
      <Breadcrumbs {crumbs} />
      <h1 class="heading">{folder ? baseOf(folder) : 'notes'}</h1>
      <span class="caption muted">{plural(rows.length, 'note')}{subfolders ? ` · ${plural(subfolders, 'folder')}` : ''}</span>
    </div>
    <div class="rows">
      <label class="add">
        <span>+</span>
        <input
          placeholder="new note"
          aria-label="New note"
          bind:value={draft}
          oninput={() => (error = '')}
          onkeydown={(e) => e.key === 'Enter' && create()} />
      </label>
      {#if draftError || error}<span class="err">{draftError || error}</span>{/if}
      {#each rows as n (n.id)}
        <a class="row" href={routes.note(n.id)}>
          <span class="name">{n.name}</span>
          <span class="meta">{relativeTo(n.folder, folder)}</span>
          <span class="meta right">{ago(n.modifiedAt)}</span>
        </a>
      {/each}
      {#if rows.length === 0}<span class="caption muted empty">No notes here yet</span>{/if}
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
    padding-top: 48px;
    display: flex;
    flex-direction: column;
    gap: 24px;
  }

  .head {
    display: flex;
    flex-direction: column;
    gap: 6px;
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
