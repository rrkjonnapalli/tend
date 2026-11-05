<script lang="ts">
  // After Cut (MD-Paste, MD-Snap-Move): what was cut and where from, Cancel, and Paste here into
  // the folder (or stack) on screen. Browsing goes on as usual underneath. Spec: tend-mobile-spec.md §8.
  import { page } from '$app/state';
  import Icon from '../Icon.svelte';
  import { clip } from '#lib/stores/clip.svelte.ts';
  import { settings } from '#lib/stores/settings.svelte.ts';
  import { snaps } from '#lib/stores/snaps.svelte.ts';
  import { settingDefaults, settingKeys } from '#lib/config/settings.config.ts';
  import { LIBRARY_AREA } from '#lib/enums/library.enum.ts';
  import type { ClipTree } from '#lib/types/clip.type.ts';
  import { NOTES_ROOT_NAME } from '#lib/util/note-paths.ts';
  import { plural } from '#lib/util/plural.ts';
  import { FOLDER_PARAM, routes, STACK_PARAM } from '#lib/util/routes.ts';
  import { whereLabel } from '#lib/util/snaps.ts';

  /** Each list's page, the query parameter naming where it is, what it holds, and how a place is
   *  named ('' is its root: the notes or tasks root, or the board). */
  const lists: Record<ClipTree, { path: string; param: string; noun: string; placeOf: (at: string) => string }> = {
    notes: { path: routes.notes, param: FOLDER_PARAM, noun: 'note', placeOf: (at) => (at ? `${NOTES_ROOT_NAME}/${at}` : NOTES_ROOT_NAME) },
    tasks: { path: routes.tasks, param: FOLDER_PARAM, noun: 'task', placeOf: tasksPlace },
    snaps: { path: routes.libraryArea(LIBRARY_AREA.snaps), param: STACK_PARAM, noun: 'snap', placeOf: snapsPlace },
  };

  const cut = $derived(clip.cut!);
  const list = $derived(lists[cut.tree]);
  /** Paste goes into the place shown by the matching list, never into a page. */
  const onList = $derived(page.url.pathname === list.path);
  const here = $derived(page.url.searchParams.get(list.param) ?? '');
  const what = $derived(plural(cut.ids.length, list.noun));
  const ready = $derived(onList && clip.canPasteIn(here));

  function tasksPlace(at: string) {
    const root = settings.get(settingKeys.tasksRootName, settingDefaults.tasksRootName);
    return at ? `${root}/${at}` : root;
  }

  function snapsPlace(at: string) {
    return at ? whereLabel(at, snaps.stacks) : 'the board';
  }
</script>

<div class="bar" role="region" aria-label="Cut">
  <span class="ic"><Icon name="cut" size={18} /></span>
  <span class="txt"><b>Cut</b><span class="m">{what} from {list.placeOf(cut.from)}</span></span>
  <button class="tb" onclick={clip.clear}>Cancel</button>
  <button class="tb act" disabled={!ready} onclick={() => clip.paste(here)}>Paste here</button>
</div>

<style>
  .bar {
    position: fixed;
    left: 0;
    right: 0;
    bottom: var(--kb, 0px);
    z-index: 30;
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 20px calc(12px + env(safe-area-inset-bottom));
    border-top: 1px solid var(--border);
    background: var(--c-selected);
  }

  .ic {
    display: grid;
    color: var(--c-current);
  }

  .txt {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }

  b {
    font: var(--text-ui-strong);
    color: var(--c-current-ink);
  }

  .m {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font: var(--text-mono-meta);
    color: var(--text-muted);
  }

  .tb {
    height: 44px;
    padding: 0;
    border: 0;
    background: transparent;
    font: var(--text-ui);
    font-size: 15px;
    color: var(--text-muted);
  }

  .tb.act {
    font-weight: 700;
    color: var(--primary);
  }

  .tb:disabled {
    color: var(--c-muted-2);
    font-weight: 400;
  }

  .tb:focus-visible {
    outline: none;
    text-decoration: underline;
  }
</style>
