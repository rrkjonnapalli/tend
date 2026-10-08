<script lang="ts">
  // Settings → General: the vault folder, and notes or folders skipped for breaking the name rule
  // (with the name each can be renamed to, and one button to rename them all).
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  import Icon from './Icon.svelte';
  import { notesAPI } from '#lib/api/notes.ts';
  import { vault } from '#lib/stores/vault.svelte.ts';
  import type { Skipped } from '#lib/types/note.type.ts';
  import { toast } from '#lib/stores/toast.svelte.ts';
  import { plural } from '#lib/util/plural.ts';
  import { routes } from '#lib/util/routes.ts';

  let skipped = $state<Skipped[]>([]);
  let showSkipped = $state(false);
  const fixable = $derived(skipped.filter((s) => s.fix).length);

  onMount(loadSkipped);

  async function loadSkipped() {
    skipped = await notesAPI.skipped();
  }

  function toggleSkipped() {
    showSkipped = !showSkipped;
  }

  async function fixNames() {
    const left = await notesAPI.fixNames();
    toast.show(`Renamed ${plural(skipped.length - left.length, 'item')}`);
    skipped = left;
  }
</script>

<section class="group">
  <h2 class="h2">Vault</h2>
  <div class="vault">
    <span class="vpath">{vault.path}</span>
    <button class="tlink" onclick={() => goto(routes.start)}>Change vault folder…</button>
  </div>
  {#if skipped.length}
    <button class="skipped" aria-expanded={showSkipped} onclick={toggleSkipped}>
      <Icon name={showSkipped ? 'chevronDown' : 'chevronRight'} size={10} stroke={3} />
      {plural(skipped.length, 'item')} skipped: names may use only letters, digits, spaces and . , _ -, and can’t start or end with a space or dot
    </button>
    {#if showSkipped}
      <ul class="skiplist">
        {#each skipped as item (item.path)}
          <li>notes/{item.path} <span class="to">{item.fix ? `→ ${item.fix}` : '· no valid name'}</span></li>
        {/each}
      </ul>
      {#if fixable}
        <div><button class="tlink" onclick={fixNames}>Rename {plural(fixable, 'item')} to fit</button></div>
      {/if}
    {/if}
  {/if}
</section>

<style>
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

  .to {
    color: var(--c-muted-2);
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
