<script lang="ts">
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import { vault } from '#lib/stores/vault.svelte.ts';

  const sections = [
    { id: 'general', label: 'General', sub: 'App-wide preferences.' },
    { id: 'notes', label: 'Notes', sub: 'Notes folder and editor.' },
    { id: 'tasks', label: 'Tasks', sub: 'Statuses and board columns.' },
    { id: 'people', label: 'People', sub: 'Contacts and birthdays.' },
    { id: 'palette', label: 'Command palette', sub: 'Every command, with its shortcut. Press ⌘K anywhere to open the palette.' },
  ] as const;

  const current = $derived(sections.find((s) => s.id === page.url.searchParams.get('s')) ?? sections[0]);

  function show(id: string) {
    goto(`/settings?s=${id}`, { replace: true });
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
      <section class="group">
        <h2 class="h2">Vault</h2>
        <div class="vault">
          <span class="vpath">{vault.path}</span>
          <button class="tlink" onclick={() => goto('/start')}>Change vault folder…</button>
        </div>
      </section>
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
    padding: 40px 24px 64px;
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

  .group {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }

  .h2 {
    font: 500 15px/20px var(--font-heading);
    margin: 0;
  }

  .vault {
    display: flex;
    align-items: center;
    gap: 16px;
    min-height: 32px;
  }

  .vpath {
    flex: 1;
    min-width: 0;
    font: 400 13px/18px var(--font-mono);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .tlink {
    padding: 0;
    border: 0;
    background: transparent;
    color: var(--link);
    font: 400 14px/20px var(--font-ui);
    white-space: nowrap;
    cursor: pointer;
  }

  .tlink:hover {
    color: var(--primary-hover);
    text-decoration: underline;
  }
</style>
