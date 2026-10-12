<script lang="ts">
  // References: the list, newest first, filtered by the Library search; + or a row opens the popup.
  import { untrack } from 'svelte';
  import ColumnHeader from '#lib/components/ColumnHeader.svelte';
  import LibraryEmpty from '#lib/components/LibraryEmpty.svelte';
  import RefForm from '#lib/components/RefForm.svelte';
  import RefRow from '#lib/components/RefRow.svelte';
  import { library } from '#lib/stores/library.svelte.ts';
  import { refs } from '#lib/stores/refs.svelte.ts';
  import type { Ref } from '#lib/types/ref.type.ts';
  import { refTitle } from '#lib/util/refs.ts';
  import { searchNeedle, textMatches } from '#lib/util/search.ts';

  const COLUMNS = ['Title', 'Reference', 'Why I kept it', 'Tags', 'Added'];

  refs.load();

  /** The open popup: `editing` null adds a new reference. */
  let form = $state<{ editing: Ref | null } | null>(null);

  const needle = $derived(searchNeedle(library.query));
  const shown = $derived(refs.refs.filter((r) => textMatches([refTitle(r), r.value, r.why], needle)));

  // The bar's + opens the popup; presses from before this page opened don't.
  let seenAdds = library.adds;
  $effect(() => {
    const adds = library.adds;
    if (adds === seenAdds) return;
    seenAdds = adds;
    untrack(() => (form = { editing: null }));
  });
</script>

{#if refs.loaded && refs.refs.length === 0}
  <LibraryEmpty title="Nothing saved yet" hint="Press + and paste a link." />
{:else}
  <div class="list-page refs">
    <ColumnHeader template="var(--grid-ref)" gap="14px" columns={COLUMNS} />
    <div class="list-scroll" data-zone="rows">
      {#each shown as ref (ref.id)}
        <RefRow {ref} onopen={() => (form = { editing: ref })} />
      {/each}
    </div>
    {#if refs.loaded && shown.length === 0}
      <LibraryEmpty title="Nothing matches" hint="Try other words." />
    {/if}
  </div>
{/if}

{#if form}
  {#key form}
    <RefForm editing={form.editing} onclose={() => (form = null)} onopen={(ref) => (form = { editing: ref })} />
  {/key}
{/if}

<style>
  .refs {
    padding-top: 16px;
  }
</style>
