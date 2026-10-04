<script lang="ts">
  import { goto } from '$app/navigation';
  import { page } from '$app/state';
  import Icon from '#lib/components/Icon.svelte';
  import StatusesEditor from '#lib/components/StatusesEditor.svelte';
  import { notesApi } from '#lib/api/notes.ts';
  import { vault } from '#lib/stores/vault.svelte.ts';
  import { commands, type CommandGroup } from '#lib/commands.ts';
  import { keyParts } from '#lib/util/keys.ts';
  import { plural } from '#lib/util/plural.ts';
  import { nav } from '#lib/stores/history.svelte.ts';

  const sections = [
    { id: 'general', label: 'General', sub: 'App-wide preferences.' },
    { id: 'notes', label: 'Notes', sub: 'Notes folder and editor.' },
    { id: 'tasks', label: 'Tasks', sub: 'Statuses and board columns.' },
    { id: 'people', label: 'People', sub: 'Contacts and birthdays.' },
    { id: 'palette', label: 'Command palette', sub: 'Every command, with its shortcut. Press ⌘K anywhere to open the palette.' },
  ] as const;

  const current = $derived(sections.find((s) => s.id === page.url.searchParams.get('s')) ?? sections[0]);

  let skipped = $state<string[]>([]);
  let showSkipped = $state(false);
  $effect(() => {
    if (current.id === 'general') notesApi.skipped().then((s) => (skipped = s));
  });

  const groups = $derived.by(() => {
    const out: { name: CommandGroup; items: typeof commands }[] = [];
    for (const c of commands) {
      let g = out.find((x) => x.name === c.group);
      if (!g) out.push((g = { name: c.group, items: [] }));
      g.items.push(c);
    }
    return out;
  });

  function show(id: string) {
    nav.replace(`/settings?s=${id}`);
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
        {#if skipped.length}
          <button class="skipped" aria-expanded={showSkipped} onclick={() => (showSkipped = !showSkipped)}>
            <Icon name={showSkipped ? 'chevronDown' : 'chevronRight'} size={10} stroke={3} />
            {plural(skipped.length, 'item')} skipped: names must use a–z, 0–9 and _ only
          </button>
          {#if showSkipped}
            <ul class="skiplist">
              {#each skipped as item (item)}<li>notes/{item}</li>{/each}
            </ul>
          {/if}
        {/if}
      </section>
    {:else if current.id === 'tasks'}
      <StatusesEditor />
    {:else if current.id === 'palette'}
      <section class="palette">
        {#each groups as g (g.name)}
          <div class="group">
            <h2 class="h2 gh">{g.name}</h2>
            {#each g.items as c (c.id)}
              <div class="crow">
                <span>{c.label}</span>
                <span class="cex">{c.example ?? ''}</span>
                <span class="ckeys">{#if c.keys}{#each keyParts(c.keys) as k, i (i)}<kbd>{k}</kbd>{/each}{/if}</span>
              </div>
            {/each}
          </div>
        {/each}
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

  .skipped {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 0;
    border: 0;
    background: transparent;
    font: 400 13px/18px var(--font-ui);
    color: var(--c-muted-2);
    cursor: pointer;
    text-align: left;
  }

  .skipped:hover {
    color: var(--text);
  }

  .skiplist {
    margin: 0;
    padding: 0 0 0 16px;
    list-style: none;
    font: 400 12px/20px var(--font-mono);
    color: var(--text-muted);
  }

  .palette {
    display: flex;
    flex-direction: column;
    gap: 24px;
  }

  .group {
    display: flex;
    flex-direction: column;
  }

  .gh {
    margin-bottom: 6px;
  }

  .crow {
    display: grid;
    grid-template-columns: minmax(0, 220px) minmax(0, 1fr) auto;
    align-items: center;
    gap: 16px;
    min-height: 34px;
    padding: 0 16px;
    margin: 0 -16px;
    border-top: 1px solid var(--c-chip);
    font: 400 14px/20px var(--font-ui);
  }

  .crow:hover {
    background: var(--c-hover);
  }

  .cex {
    font: 400 12px/16px var(--font-mono);
    color: var(--c-muted-2);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .ckeys {
    display: flex;
    gap: 4px;
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
