<script lang="ts">
  // The Library's top bar: the area switch, + (the open area's own add) and the area's search.
  import { goto } from '$app/navigation';
  import Icon from './Icon.svelte';
  import Segmented from './Segmented.svelte';
  import SearchLine from './SearchLine.svelte';
  import { libraryAreas } from '#lib/config/library.config.ts';
  import type { LibraryArea } from '#lib/enums/library.enum.ts';
  import { library } from '#lib/stores/library.svelte.ts';
  import { routes } from '#lib/util/routes.ts';

  let { area }: { area: LibraryArea } = $props();

  const options = libraryAreas.map((a) => ({ value: a.area, label: a.label }));
  /** Focus stays on the switch (a navigation would drop it), so the next ← → switches again. */
  const switchTo = (to: LibraryArea) => goto(routes.libraryArea(to), { reset: false });
  const search = $derived(libraryAreas.find((a) => a.area === area)?.search ?? '');
</script>

<div class="lbar" data-zone="bar">
  <Segmented {options} value={area} label="Library" onchange={switchTo} />
  <button class="icon-btn add" aria-label="Add to the library" title="Add" onclick={library.requestAdd}>
    <Icon name="plus" size={16} />
  </button>
  <div class="find">
    <SearchLine bind:value={library.query} label={search} placeholder={search} clearable />
  </div>
</div>

<style>
  .lbar {
    flex: none;
    display: flex;
    align-items: center;
    gap: 12px;
    height: 56px;
    padding: 0 20px;
    border-bottom: 1px solid var(--border);
  }

  .add {
    margin-left: auto;
  }

  .find {
    width: 300px;
  }
</style>
