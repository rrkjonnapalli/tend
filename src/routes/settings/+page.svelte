<script lang="ts">
  import { page } from '$app/state';
  import StatusesEditor from '#lib/components/StatusesEditor.svelte';
  import LabelsEditor from '#lib/components/LabelsEditor.svelte';
  import WordKindsEditor from '#lib/components/WordKindsEditor.svelte';
  import SettingsGeneral from '#lib/components/SettingsGeneral.svelte';
  import SettingsPalette from '#lib/components/SettingsPalette.svelte';
  import SettingsSync from '#lib/components/SettingsSync.svelte';
  import SettingsTags from '#lib/components/SettingsTags.svelte';
  import PageBar from '#lib/components/PageBar.svelte';
  import { hasLayers } from '#lib/util/layers.ts';
  import { isTypingTarget } from '#lib/util/keys.ts';
  import { nextIndex, prevIndex } from '#lib/util/list-nav.ts';
  import Breadcrumbs from '#lib/components/Breadcrumbs.svelte';
  import PageTitle from '#lib/components/PageTitle.svelte';
  import { nav } from '#lib/stores/history.svelte.ts';
  import { routes } from '#lib/util/routes.ts';
  import { SETTINGS_SECTION, type SettingsSection } from '#lib/enums/settings.enum.ts';

  /** The Settings root in breadcrumbs, lowercase like `notes`, `tasks` and `people`. */
  const SETTINGS_ROOT_NAME = 'settings';

  /** The query parameter holding the open section (see routes.settingsSection). */
  const SECTION_PARAM = 's';

  const sections: { id: SettingsSection; label: string; sub: string }[] = [
    { id: SETTINGS_SECTION.general, label: 'General', sub: 'App-wide preferences.' },
    { id: SETTINGS_SECTION.tasks, label: 'Tasks', sub: 'Statuses, board columns and labels.' },
    { id: SETTINGS_SECTION.library, label: 'Library', sub: 'Word kinds.' },
    { id: SETTINGS_SECTION.tags, label: 'Tags', sub: 'Every tag, and the ones you blocked.' },
    { id: SETTINGS_SECTION.sync, label: 'Sync', sub: 'Keep this vault in a Git repo, and in step with your other devices.' },
    { id: SETTINGS_SECTION.palette, label: 'Command palette', sub: 'Every command, with its shortcut. Press ⌘K anywhere to open the palette.' },
  ];

  const requested = $derived(page.url.searchParams.get(SECTION_PARAM));
  const current = $derived(sections.find((s) => s.id === requested) ?? sections[0]);

  let navEl = $state<HTMLElement>();

  function show(id: SettingsSection) {
    return nav.replace(routes.settingsSection(id), { keepFocus: true });
  }

  /** ↑↓ switch sections, like a sidebar, unless focus is in a field or the page's content. */
  async function onkeydown(e: KeyboardEvent) {
    const isArrow = e.key === 'ArrowDown' || e.key === 'ArrowUp';
    const hasModifier = e.metaKey || e.ctrlKey || e.altKey || e.shiftKey;
    if (!isArrow || hasModifier || hasLayers() || isTypingTarget(e.target)) return;
    const onNav = e.target === document.body || navEl?.contains(e.target as Node);
    if (!onNav) return;
    e.preventDefault();
    const at = sections.findIndex((s) => s.id === current.id);
    const next = e.key === 'ArrowDown' ? nextIndex(at, sections.length) : prevIndex(at);
    navEl?.querySelectorAll<HTMLElement>('.sec')[next]?.focus();
    await show(sections[next].id);
  }
</script>

<svelte:window {onkeydown} />

<aside class="side" aria-label="Settings sections" data-pane="side">
  <div class="top"><span class="caption muted">Settings</span></div>
  <nav class="secs" bind:this={navEl}>
    {#each sections as s (s.id)}
      <button class="sec" class:on={s.id === current.id} aria-current={s.id === current.id ? 'page' : undefined} onclick={() => show(s.id)}>
        {s.label}
      </button>
    {/each}
  </nav>
</aside>

<main class="detail-page detail-stack">
  <PageBar><Breadcrumbs crumbs={[{ label: SETTINGS_ROOT_NAME }, { label: current.label }]} /></PageBar>
  <article class="detail-body">
    <PageTitle title={current.label} meta={current.sub} />

    {#if current.id === SETTINGS_SECTION.general}
      <SettingsGeneral />
    {:else if current.id === SETTINGS_SECTION.tasks}
      <StatusesEditor />
      <LabelsEditor />
    {:else if current.id === SETTINGS_SECTION.library}
      <WordKindsEditor />
    {:else if current.id === SETTINGS_SECTION.tags}
      <SettingsTags />
    {:else if current.id === SETTINGS_SECTION.sync}
      <SettingsSync />
    {:else}
      <SettingsPalette />
    {/if}
  </article>
</main>

<style>
  .side {
    flex: 0 0 var(--explorer-w);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
  }

  .top {
    flex: 0 0 40px;
    display: flex;
    align-items: center;
    padding: 0 16px;
  }

  .secs {
    display: flex;
    flex-direction: column;
  }

  .sec {
    display: flex;
    align-items: center;
    min-height: 32px;
    padding: 0 16px;
    border: 0;
    background: transparent;
    text-align: left;
    font: var(--text-ui);
    color: var(--text-muted);
    cursor: pointer;
  }

  .sec:hover {
    background: var(--c-hover);
    color: var(--text);
  }

  .sec.on {
    background: var(--c-selected);
    color: var(--c-current-ink);
  }

</style>
