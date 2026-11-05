<script lang="ts">
  // Notes on a phone (MC-Notes, MC-Notes-Open): the notes directly in the folder, newest edit first;
  // the folders are in the drawer. A search looks through this folder and everything inside it.
  import FolderNav from './FolderNav.svelte';
  import QuickSheet from './QuickSheet.svelte';
  import { notes } from '#lib/stores/notes.svelte.ts';
  import { NOTES_ROOT_NAME } from '#lib/util/note-paths.ts';
  import { searchNeedle, textMatches } from '#lib/util/search.ts';
  import { ago } from '#lib/util/dates.ts';
  import { routes } from '#lib/util/routes.ts';

  let { folder }: { folder: string } = $props();

  let query = $state('');
  let adding = $state(false);

  const needle = $derived(searchNeedle(query));
  const shown = $derived(
    needle ? notes.under(folder).filter((n) => textMatches([n.name, n.folder], needle)) : notes.notes.filter((n) => n.folder === folder),
  );
  const rows = $derived([...shown].sort((a, b) => b.modifiedAt - a.modifiedAt));
</script>

<main class="phone-list">
  <FolderNav rootName={NOTES_ROOT_NAME} {folder} hrefOf={routes.notesFolder} placeholder="Search notes" bind:query onadd={() => (adding = true)} />
  {#each rows as n (n.id)}
    <a class="qr" href={routes.note(n.id)}>
      <span class="tt">{n.name}</span>
      {#if needle && n.folder}<span class="path">{n.folder}</span>{/if}
      <span class="meta">{ago(n.modifiedAt)}</span>
    </a>
  {:else}
    <p class="empty">{needle ? 'No notes match' : 'No notes here yet'}</p>
  {/each}
</main>

{#if adding}<QuickSheet tree="notes" {folder} rootName={NOTES_ROOT_NAME} onclose={() => (adding = false)} />{/if}
