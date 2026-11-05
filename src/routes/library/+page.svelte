<script lang="ts">
  // /library: open the area used last.
  import { nav } from '#lib/stores/history.svelte.ts';
  import { settings } from '#lib/stores/settings.svelte.ts';
  import { settingDefaults, settingKeys } from '#lib/config/settings.config.ts';
  import { LIBRARY_AREA, type LibraryArea } from '#lib/enums/library.enum.ts';
  import { ui } from '#lib/stores/ui.svelte.ts';
  import { routes } from '#lib/util/routes.ts';

  $effect(() => {
    const last = settings.get<LibraryArea>(settingKeys.libraryArea, settingDefaults.libraryArea);
    // A phone has no Home: it opens Words instead.
    const area = ui.phone && last === LIBRARY_AREA.home ? LIBRARY_AREA.words : last;
    nav.replace(routes.libraryArea(area));
  });
</script>
