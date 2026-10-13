<script lang="ts">
  // The Library: the bar on top, the open area below. Switching area clears the search and is
  // remembered, so the Library reopens on it.
  import { untrack, type Snippet } from 'svelte';
  import { page } from '$app/state';
  import LibraryBar from '#lib/components/LibraryBar.svelte';
  import { zoneNav } from '#lib/actions/zone-nav.action.ts';
  import { library } from '#lib/stores/library.svelte.ts';
  import { settings } from '#lib/stores/settings.svelte.ts';
  import { settingKeys } from '#lib/config/settings.config.ts';
  import { libraryAreaOf } from '#lib/util/routes.ts';

  let { children }: { children: Snippet } = $props();

  /** The open area; none on `/library` itself. */
  const area = $derived(libraryAreaOf(page.route.id));

  $effect(() => {
    const now = area;
    if (!now) return;
    untrack(() => {
      library.clearQuery();
      settings.set(settingKeys.libraryArea, now);
    });
  });
</script>

<main class="library" use:zoneNav>
  {#if area}<LibraryBar {area} />{/if}
  {@render children()}
</main>

<style>
  .library {
    flex: 1 1 auto;
    display: flex;
    flex-direction: column;
    min-height: 0;
  }
</style>
