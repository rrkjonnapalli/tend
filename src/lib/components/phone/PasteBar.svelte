<script lang="ts">
  // After Cut (MD-Paste): what was cut and where from, Cancel, and Paste here into the folder of
  // the list on screen. Browsing goes on as usual underneath. Spec: tend-mobile-spec.md §8.
  import { page } from '$app/state';
  import Icon from '../Icon.svelte';
  import { clip } from '#lib/stores/clip.svelte.ts';
  import { settings } from '#lib/stores/settings.svelte.ts';
  import { settingDefaults, settingKeys } from '#lib/config/settings.config.ts';
  import { NOTES_ROOT_NAME } from '#lib/util/note-paths.ts';
  import { plural } from '#lib/util/plural.ts';
  import { routes } from '#lib/util/routes.ts';

  const cut = $derived(clip.cut!);
  const listPath = $derived(cut.tree === 'notes' ? routes.notes : routes.tasks);
  /** Paste goes into the folder of the matching list (Notes or Tasks), never a page. */
  const onList = $derived(page.url.pathname === listPath);
  const folder = $derived(page.url.searchParams.get('f') ?? '');
  const rootName = $derived(cut.tree === 'notes' ? NOTES_ROOT_NAME : settings.get(settingKeys.tasksRootName, settingDefaults.tasksRootName));
  const what = $derived(plural(cut.ids.length, cut.tree === 'notes' ? 'note' : 'task'));
  const from = $derived(cut.from ? `${rootName}/${cut.from}` : rootName);
  const ready = $derived(onList && clip.canPasteIn(folder));
</script>

<div class="bar" role="region" aria-label="Cut">
  <span class="ic"><Icon name="cut" size={18} /></span>
  <span class="txt"><b>Cut</b><span class="m">{what} from {from}</span></span>
  <button class="tb" onclick={clip.clear}>Cancel</button>
  <button class="tb act" disabled={!ready} onclick={() => clip.paste(folder)}>Paste here</button>
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
