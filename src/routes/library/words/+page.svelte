<script lang="ts">
  // Words: the list by kind on the left, the open word (or a new one) on the right. The open word
  // is in the URL (`?w=<id>`), so Back and Forward move between words.
  import { untrack } from 'svelte';
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import LibraryEmpty from '#lib/components/LibraryEmpty.svelte';
  import WordEntry from '#lib/components/WordEntry.svelte';
  import WordList from '#lib/components/WordList.svelte';
  import { LIBRARY_AREA } from '#lib/enums/library.enum.ts';
  import { library } from '#lib/stores/library.svelte.ts';
  import { words } from '#lib/stores/words.svelte.ts';
  import { routes } from '#lib/util/routes.ts';
  import { searchNeedle } from '#lib/util/search.ts';
  import { groupWords } from '#lib/util/words.ts';

  words.load();

  /** A title handed over from Home: the page opens adding it. */
  const handed = library.takeHanded();
  let adding = $state(handed !== null);
  /** The new word's title as typed, mirrored in the list's "new …" row. */
  let draftTitle = $state(handed ?? '');

  const groups = $derived(groupWords(words.words, words.kinds, searchNeedle(library.query)));
  const requested = $derived(page.url.searchParams.get('w'));
  /** The open word: the one in the URL, else the first in the list. */
  const open = $derived(words.byId(requested) ?? groups[0]?.words[0] ?? null);

  // The bar's + adds a word; presses from before this page opened don't.
  let seenAdds = library.adds;
  $effect(() => {
    const adds = library.adds;
    if (adds === seenAdds) return;
    seenAdds = adds;
    untrack(startAdding);
  });

  function startAdding() {
    draftTitle = '';
    adding = true;
  }

  function show(id: string) {
    adding = false;
    draftTitle = '';
    goto(routes.libraryWord(id));
  }

  function afterDelete() {
    goto(routes.libraryArea(LIBRARY_AREA.words), { replaceState: true });
  }
</script>

{#if words.loaded && words.words.length === 0 && !adding}
  <LibraryEmpty title="Nothing here yet" hint="Press + to add a word, an idiom or a phrase." />
{:else}
  <div class="words">
    <WordList {groups} selected={open?.id ?? null} adding={adding ? draftTitle.trim() || 'new word' : null} onpick={show} />
    {#if adding}
      <WordEntry word={null} filled={draftTitle} onsaved={show} oncancel={() => (adding = false)} onopen={show} ondeleted={afterDelete} ontyped={(t) => (draftTitle = t)} />
    {:else if open}
      {#key open.id}
        <WordEntry word={open} onsaved={show} oncancel={() => {}} onopen={show} ondeleted={afterDelete} />
      {/key}
    {/if}
  </div>
{/if}

<style>
  .words {
    flex: 1 1 auto;
    display: flex;
    min-height: 0;
  }
</style>
