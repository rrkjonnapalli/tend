<script lang="ts">
  // /library: open the area used last.
  import { nav } from '#lib/stores/history.svelte.ts';
  import { settings } from '#lib/stores/settings.svelte.ts';
  import { settingDefaults, settingKeys } from '#lib/config/settings.config.ts';
  import type { LibraryArea } from '#lib/enums/library.enum.ts';
  import { routes } from '#lib/util/routes.ts';

  $effect(() => {
    const last = settings.get<LibraryArea>(settingKeys.libraryArea, settingDefaults.libraryArea);
    nav.replace(routes.libraryArea(last));
  });
</script>
