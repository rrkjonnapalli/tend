<script lang="ts">
  // Notes on a phone (MC-Notes, MC-Notes-Open): the notes directly in the folder, newest edit first;
  // the folders are in the drawer. A search looks through this folder and everything inside it.
  import FolderNav from './FolderNav.svelte';
  import QuickSheet from './QuickSheet.svelte';
  import SelectBar from './SelectBar.svelte';
  import Selection from './Selection.svelte';
  import Icon from '../Icon.svelte';
  import { longPress } from '#lib/actions/long-press.action.ts';
  import { clip } from '#lib/stores/clip.svelte.ts';
  import { settings } from '#lib/stores/settings.svelte.ts';
  import { notes } from '#lib/stores/notes.svelte.ts';
  import { NOTES_ROOT_NAME } from '#lib/util/note-paths.ts';
  import { searchNeedle, textMatches } from '#lib/util/search.ts';
  import { ago } from '#lib/util/dates.ts';
  import { routes } from '#lib/util/routes.ts';

  let { folder }: { folder: string } = $props();

  let query = $state('');
  let adding = $state(false);

  const selecting = $derived(clip.selecting?.tree === 'notes');

  const needle = $derived(searchNeedle(query));
  const shown = $derived(
    needle ? notes.under(folder).filter((n) => textMatches([n.name, n.folder], needle)) : notes.notes.filter((n) => n.folder === folder),
  );
  const rows = $derived([...shown].sort((a, b) => b.modifiedAt - a.modifiedAt));
</script>

<main class="phone-list">
  {#if selecting}
    <SelectBar all={rows.map((n) => n.id)} />
  {:else}
    <FolderNav rootName={NOTES_ROOT_NAME} {folder} hrefOf={routes.notesFolder} placeholder="Search notes" bind:query onadd={() => (adding = true)} />
  {/if}
  {#each rows as n (n.id)}
    {#snippet body()}
      <span class="tt">{n.name}</span>
      {#if needle && n.folder}<span class="path">{n.folder}</span>{/if}
      <span class="meta">{ago(n.modifiedAt)}</span>
    {/snippet}
    {#if selecting}
      {@const on = clip.selecting?.ids.includes(n.id)}
      <button class="qr" class:sel={on} aria-pressed={on} onclick={() => clip.toggle(n.id)}>
        <span class="check"><Icon name="check" size={14} stroke={3} /></span>{@render body()}
      </button>
    {:else}
      <a class="qr" href={routes.note(n.id)} use:longPress={() => clip.start('notes', n.id)}>{@render body()}</a>
    {/if}
  {:else}
    <p class="empty">{needle ? 'No notes match' : 'No notes here yet'}</p>
  {/each}
</main>

{#if selecting}
  <Selection
    {folder}
    noun="note"
    asksDelete={settings.asksDelete('note')}
    nameOf={(id) => notes.byId(id)?.name ?? ''}
    onrename={(id, name) => notes.rename({ kind: 'note', id }, name)}
    onremove={(id) => notes.remove({ kind: 'note', id })} />
{/if}

{#if adding}<QuickSheet tree="notes" {folder} rootName={NOTES_ROOT_NAME} onclose={() => (adding = false)} />{/if}
