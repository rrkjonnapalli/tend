<script lang="ts">
  // Library Home: the recent words, the latest snaps on a small board and the newest references.
  // While the Library search has text it shows every match instead, grouped by area. + asks what
  // the thing is and hands it to its area.
  import { untrack } from 'svelte';
  import { goto } from '$app/navigation';
  import LibraryAdd from '#lib/components/LibraryAdd.svelte';
  import LibraryEmpty from '#lib/components/LibraryEmpty.svelte';
  import LibraryItem from '#lib/components/LibraryItem.svelte';
  import MiniBoard from '#lib/components/MiniBoard.svelte';
  import { homeCounts } from '#lib/config/library.config.ts';
  import { LIBRARY_AREA, type SaveArea } from '#lib/enums/library.enum.ts';
  import { library } from '#lib/stores/library.svelte.ts';
  import { refs } from '#lib/stores/refs.svelte.ts';
  import { snaps } from '#lib/stores/snaps.svelte.ts';
  import { words } from '#lib/stores/words.svelte.ts';
  import type { LibraryGroup } from '#lib/types/library.type.ts';
  import { newest, refHit, snapHit, wordHit } from '#lib/util/library.ts';
  import { plural } from '#lib/util/plural.ts';
  import { refTitle } from '#lib/util/refs.ts';
  import { routes } from '#lib/util/routes.ts';
  import { searchNeedle, textMatches } from '#lib/util/search.ts';

  words.load();
  snaps.load();
  refs.load();

  let adding = $state(false);

  const needle = $derived(searchNeedle(library.query));
  const counts = $derived(
    [plural(words.words.length, 'word'), plural(snaps.snaps.length, 'snap'), plural(refs.refs.length, 'reference')].join(' · '),
  );

  const recentWords = $derived(newest(words.words, homeCounts.words).map(wordHit));
  const latestSnaps = $derived(newest(snaps.snaps, homeCounts.snaps));
  const newRefs = $derived(newest(refs.refs, homeCounts.refs).map(refHit));

  /** Every match of the search, by area; areas without one are left out. */
  const groups = $derived.by((): LibraryGroup[] => {
    const all: LibraryGroup[] = [
      { area: LIBRARY_AREA.words, label: 'Words', hits: words.words.filter((w) => textMatches([w.title, w.meaning], needle)).map(wordHit) },
      { area: LIBRARY_AREA.snaps, label: 'Snaps', hits: snaps.snaps.filter((s) => textMatches([s.text], needle)).map((s) => snapHit(s)) },
      { area: LIBRARY_AREA.refs, label: 'References', hits: refs.refs.filter((r) => textMatches([refTitle(r), r.value, r.why], needle)).map(refHit) },
    ];
    return all.filter((g) => g.hits.length > 0);
  });
  const loaded = $derived(words.loaded && snaps.loaded && refs.loaded);

  // The bar's + opens the add box; presses from before this page opened don't.
  let seenAdds = library.adds;
  $effect(() => {
    const adds = library.adds;
    if (adds === seenAdds) return;
    seenAdds = adds;
    untrack(() => (adding = true));
  });

  const open = (area: SaveArea) => goto(routes.libraryArea(area));
</script>

{#if needle}
  <div class="scroll">
    {#if loaded && groups.length === 0}
      <LibraryEmpty title="Nothing matches" hint="Nothing in the library matches “{library.query.trim()}”." />
    {:else}
      <div class="results">
        {#each groups as group (group.area)}
          <section class="group">
            <h2>{group.label}<span class="count">{group.hits.length}</span></h2>
            {#each group.hits as hit (hit.id)}
              <LibraryItem {hit} />
            {/each}
          </section>
        {/each}
      </div>
    {/if}
  </div>
{:else}
  <div class="scroll">
    <div class="home">
      <div>
        <h1>Library</h1>
        <span class="meta">{counts}</span>
      </div>
      <div class="cols">
        <section class="col">
          <div class="head">
            <h2>Words</h2>
            <span class="meta">recent</span>
            <button class="more" onclick={() => open(LIBRARY_AREA.words)}>All →</button>
          </div>
          {#each recentWords as hit (hit.id)}
            <LibraryItem {hit} />
          {/each}
          {#if words.loaded && recentWords.length === 0}
            <p class="none">No words yet. Press + and type one.</p>
          {/if}
        </section>
        <section class="col">
          <div class="head">
            <h2>Snaps</h2>
            <span class="meta">latest</span>
            <button class="more" onclick={() => open(LIBRARY_AREA.snaps)}>Board →</button>
          </div>
          <MiniBoard snaps={latestSnaps} loaded={snaps.loaded} />
        </section>
        <section class="col">
          <div class="head">
            <h2>References</h2>
            <span class="meta">new</span>
            <button class="more" onclick={() => open(LIBRARY_AREA.refs)}>All →</button>
          </div>
          {#each newRefs as hit (hit.id)}
            <LibraryItem {hit} />
          {/each}
          {#if refs.loaded && newRefs.length === 0}
            <p class="none">No references yet. Press + and paste a link.</p>
          {/if}
        </section>
      </div>
    </div>
  </div>
{/if}

{#if adding}
  <LibraryAdd onclose={() => (adding = false)} />
{/if}

<style>
  .scroll {
    flex: 1 1 auto;
    display: flex;
    flex-direction: column;
    min-height: 0;
    overflow: auto;
  }

  .home {
    box-sizing: border-box;
    width: 100%;
    max-width: 1140px;
    margin: 0 auto;
    padding: 26px 32px 24px;
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  h1 {
    margin: 0;
    font: var(--text-heading);
  }

  h2 {
    margin: 0;
    font: var(--text-section);
  }

  .meta {
    font: var(--text-caption);
    color: var(--text-muted);
  }

  .cols {
    display: grid;
    grid-template-columns: minmax(0, 1fr) minmax(0, 1.25fr) minmax(0, 1fr);
    gap: 28px;
  }

  .col {
    display: flex;
    flex-direction: column;
    gap: 8px;
    min-width: 0;
  }

  .head {
    display: flex;
    align-items: baseline;
    gap: 8px;
    margin-bottom: 2px;
  }

  .more {
    margin-left: auto;
    padding: 0;
    border: 0;
    background: transparent;
    font: var(--text-caption);
    color: var(--primary);
    cursor: pointer;
  }

  .more:hover,
  .more:focus-visible {
    color: var(--primary-hover);
  }

  .none {
    margin: 0;
    padding: 8px 0;
    font: var(--text-caption);
    color: var(--c-muted-2);
  }

  .results {
    box-sizing: border-box;
    width: 100%;
    max-width: 820px;
    margin: 0 auto;
    padding: 24px 32px;
    display: flex;
    flex-direction: column;
    gap: 22px;
  }

  .group {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .group h2 {
    margin-bottom: 4px;
  }

  .count {
    margin-left: 6px;
    font: var(--text-meta);
    color: var(--text-muted);
  }
</style>
