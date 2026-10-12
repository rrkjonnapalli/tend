<script lang="ts">
  // The Library's top bar: the area switch, + (the open area's own add) and the area's search.
  import { goto } from '$app/navigation';
  import Icon from './Icon.svelte';
  import ViewTabs from './ViewTabs.svelte';
  import SearchLine from './SearchLine.svelte';
  import { libraryAreas } from '#lib/config/library.config.ts';
  import type { LibraryArea } from '#lib/enums/library.enum.ts';
  import { library } from '#lib/stores/library.svelte.ts';
  import { routes } from '#lib/util/routes.ts';

  let { area }: { area: LibraryArea } = $props();

  const options = libraryAreas.map((a) => ({ value: a.area, label: a.label }));
  /** Focus stays on the tab (a navigation would drop it), so the next ← → switches again. */
  const switchTo = (to: LibraryArea) => goto(routes.libraryArea(to), { reset: false });
  const search = $derived(libraryAreas.find((a) => a.area === area)?.search ?? '');
</script>

<div class="lbar" data-zone="bar">
  <nav class="areas" aria-label="Library">
    <ViewTabs {options} value={area} onchange={switchTo} />
  </nav>
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
    gap: 8px;
    height: 48px;
    padding: 0 24px;
  }

  .areas {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .add {
    margin-left: auto;
  }

  .find {
    width: 240px;
  }
</style>
