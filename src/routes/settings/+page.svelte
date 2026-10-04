<script lang="ts">
  import { page } from '$app/state';
  import StatusesEditor from '#lib/components/StatusesEditor.svelte';
  import SettingsGeneral from '#lib/components/SettingsGeneral.svelte';
  import SettingsPalette from '#lib/components/SettingsPalette.svelte';
  import { nav } from '#lib/stores/history.svelte.ts';
  import { routes } from '#lib/util/routes.ts';

  /** The query parameter holding the open section. */
  const SECTION_PARAM = 's';

  const sections = [
    { id: 'general', label: 'General', sub: 'App-wide preferences.' },
    { id: 'notes', label: 'Notes', sub: 'Notes folder and editor.' },
    { id: 'tasks', label: 'Tasks', sub: 'Statuses and board columns.' },
    { id: 'people', label: 'People', sub: 'Contacts and birthdays.' },
    { id: 'palette', label: 'Command palette', sub: 'Every command, with its shortcut. Press ⌘K anywhere to open the palette.' },
  ] as const;

  const requested = $derived(page.url.searchParams.get(SECTION_PARAM));
  const current = $derived(sections.find((s) => s.id === requested) ?? sections[0]);

  function show(id: string) {
    nav.replace(`${routes.settings}?${SECTION_PARAM}=${id}`);
  }
</script>

<aside class="side" aria-label="Settings sections">
  <div class="top"><span class="caption muted">Settings</span></div>
  <nav class="secs">
    {#each sections as s (s.id)}
      <button class="sec" class:on={s.id === current.id} aria-current={s.id === current.id ? 'page' : undefined} onclick={() => show(s.id)}>
        {s.label}
      </button>
    {/each}
  </nav>
</aside>

<main class="main">
  <article class="body">
    <div class="head">
      <h1 class="heading">{current.label}</h1>
      <span class="caption muted">{current.sub}</span>
    </div>

    {#if current.id === 'general'}
      <SettingsGeneral />
    {:else if current.id === 'tasks'}
      <StatusesEditor />
    {:else if current.id === 'palette'}
      <SettingsPalette />
    {:else}
      <span class="caption muted">Nothing to configure here yet.</span>
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
    font: 400 14px/20px var(--font-ui);
    color: var(--text-muted);
    cursor: pointer;
  }

  .sec:hover {
    background: var(--c-hover);
    color: var(--text);
  }

  .sec.on {
    background: var(--c-chip);
    color: var(--text);
  }

  .main {
    flex: 1 1 auto;
    min-width: 0;
    padding: 12px 24px 64px;
    overflow: auto;
  }

  .body {
    max-width: 620px;
    margin: 0 auto;
    display: flex;
    flex-direction: column;
    gap: 28px;
  }

  .head {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
</style>
