<script lang="ts">
  // "Move to…": a path field with folder suggestions. Leaves out the item's own folder and,
  // for folders, the folder itself and everything inside it. A new path is created.
  import Panel from './Panel.svelte';
  import { isUnder, isValidPath } from '#lib/util/names.ts';
  import { label as keyLabel } from '#lib/util/keys.ts';

  let {
    what,
    folders,
    current,
    self,
    rootName,
    onmove,
    onclose,
  }: {
    /** Shown in the header, e.g. "work/meetings/". */
    what: string;
    /** Every folder path, '' (the root) included. */
    folders: string[];
    /** The folder the item is in now. */
    current: string;
    /** When moving a folder: its own path. */
    self?: string;
    rootName: string;
    onmove: (target: string) => void;
    onclose: () => void;
  } = $props();

  let query = $state('');
  let hi = $state(0);
  /** True once ↑↓ picked a suggestion; Enter then takes it instead of the typed path. */
  let browsing = $state(false);
  const MAX = 6;

  const allowed = $derived(folders.filter((f) => f !== current && !(self !== undefined && isUnder(f, self))));
  const target = $derived(query.trim().replace(/\/+$/, ''));
  const suggestions = $derived(allowed.filter((f) => !target || f.includes(target)).slice(0, MAX));

  const error = $derived.by(() => {
    if (target && !isValidPath(target)) return 'Folder names: a–z, 0–9 and _ only';
    if (self !== undefined && isUnder(target, self)) return 'Can’t move a folder into itself';
    if (target === current) return 'Already there';
    return '';
  });

  const info = $derived(folders.includes(target) ? `into ${target || rootName}/` : `will create ${target}/`);

  function go(path: string) {
    if (error && path === target) return;
    onmove(path);
  }

  function onkeydown(e: KeyboardEvent) {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      if (browsing) hi = Math.min(hi + 1, suggestions.length - 1);
      browsing = suggestions.length > 0;
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      hi = Math.max(hi - 1, 0);
    } else if (e.key === 'Tab' && suggestions[hi] !== undefined) {
      e.preventDefault();
      query = suggestions[hi];
    } else if (e.key === 'Enter') {
      e.preventDefault();
      go(browsing && suggestions[hi] !== undefined ? suggestions[hi] : target);
    }
  }
</script>

<Panel label="Move" {onclose}>
  {#snippet head()}
    <span>Move <span class="what">{what}</span> to</span><span class="k">{keyLabel('mod+shift+m')}</span>
  {/snippet}
  <div class="row">
    <span class="l">to</span>
    <!-- svelte-ignore a11y_autofocus -->
    <input
      autofocus
      aria-label="Destination folder"
      placeholder="folder path, empty = top level"
      bind:value={query}
      oninput={() => ((hi = 0), (browsing = false))}
      {onkeydown} />
  </div>
  {#each suggestions as s, i (s)}
    <button class="sg" class:on={browsing && i === hi} onclick={() => go(s)}>{s || rootName}</button>
  {/each}
  {#snippet foot()}
    {#if error}<span class="err">{error}</span>{:else}<span>{info}</span>{/if}
    <span class="r">↑↓ · ↵ move · esc</span>
  {/snippet}
</Panel>

<style>
  .what {
    font-family: var(--font-mono);
    color: var(--text);
  }

  .row {
    display: flex;
    align-items: center;
    gap: 10px;
    height: 42px;
    padding: 0 14px;
  }

  .l {
    flex: 0 0 28px;
    font: 400 12px/16px var(--font-ui);
    color: var(--c-faint);
  }

  input {
    flex: 1;
    border: 0;
    outline: none;
    background: transparent;
    font: 400 13px/20px var(--font-mono);
    color: var(--text-muted);
  }

  .sg {
    display: block;
    width: 100%;
    height: 28px;
    padding: 0 14px 0 52px;
    border: 0;
    background: transparent;
    text-align: left;
    font: 400 13px/18px var(--font-mono);
    color: var(--text-muted);
    cursor: pointer;
  }

  .sg.on,
  .sg:hover {
    background: var(--c-hover);
    color: var(--text);
  }
</style>
