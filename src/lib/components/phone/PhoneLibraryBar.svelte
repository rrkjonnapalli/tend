<script lang="ts">
  // The Library's top on a phone (ML-Words): search and +, then the tabs Words · Snaps ·
  // References with their counts. No Home and no tag filter (tend-mobile-spec.md §7).
  import PhoneSearch from './PhoneSearch.svelte';
  import { LIBRARY_AREA, type LibraryArea } from '#lib/enums/library.enum.ts';
  import { library } from '#lib/stores/library.svelte.ts';
  import { words } from '#lib/stores/words.svelte.ts';
  import { snaps } from '#lib/stores/snaps.svelte.ts';
  import { refs } from '#lib/stores/refs.svelte.ts';
  import { routes } from '#lib/util/routes.ts';

  let { area }: { area: LibraryArea } = $props();

  words.load();
  snaps.load();
  refs.load();

  const tabs = $derived([
    { area: LIBRARY_AREA.words, label: 'Words', count: words.words.length },
    { area: LIBRARY_AREA.snaps, label: 'Snaps', count: snaps.snaps.length },
    { area: LIBRARY_AREA.refs, label: 'References', count: refs.refs.length },
  ]);
</script>

<div class="bar">
  <PhoneSearch placeholder="Search library" bind:query={library.query} onadd={library.requestAdd} />
  <nav class="tabs" aria-label="Library">
    {#each tabs as t (t.area)}
      <a class="vt" class:on={area === t.area} href={routes.libraryArea(t.area)} aria-current={area === t.area ? 'page' : undefined}>
        {t.label}<span class="n">{t.count}</span>
      </a>
    {/each}
  </nav>
</div>

<style>
  .bar {
    flex: none;
    padding: 4px 20px 0;
  }

  .tabs {
    display: flex;
    gap: 16px;
    margin: 2px 0 8px;
  }

  .vt {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    height: 36px;
    padding-top: 2px;
    border-bottom: 2px solid transparent;
    font: var(--text-ui);
    font-size: 15px;
    color: var(--c-muted-2);
    text-decoration: none;
  }

  .vt.on {
    color: var(--c-current-ink);
    border-bottom-color: var(--c-current);
  }

  .vt:focus-visible {
    outline: none;
    background: var(--c-hover);
  }

  .n {
    font: var(--text-meta);
    color: var(--c-muted-2);
  }
</style>
