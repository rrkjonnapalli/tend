<script lang="ts">
  import { focusOnMount } from '#lib/actions/focus.action.ts';
  import { plainText } from '#lib/actions/plain-text.action.ts';
  // "Move to…": a path field with folder suggestions. Leaves out the item's own folder and,
  // for folders, the folder itself and everything inside it. A new path is created.
  import Panel from './Panel.svelte';
  import { cleanPath, isUnder, pathError } from '#lib/util/names.ts';
  import { label as keyLabel } from '#lib/util/keys.ts';
  import { nextIndex, prevIndex } from '#lib/util/list-nav.ts';

  let {
    what,
    kind,
    folders,
    current,
    self,
    rootName,
    onmove,
    onclose,
  }: {
    /** The item's name, shown in the header: "Move work/meetings/ to". */
    what?: string;
    /** Without `what`, the header names the kind instead: "Move task to". */
    kind?: string;
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

  const MAX_SUGGESTIONS = 6;

  let query = $state('');
  let hi = $state(0);
  /** True once ↑↓ picked a suggestion; Enter then takes it instead of the typed path. */
  let browsing = $state(false);

  /** When moving a folder, `path` is that folder or inside it. */
  const insideSelf = (path: string) => self !== undefined && isUnder(path, self);

  const allowed = $derived(folders.filter((f) => f !== current && !insideSelf(f)));
  const target = $derived(cleanPath(query));
  const suggestions = $derived(allowed.filter((f) => !target || f.includes(target)).slice(0, MAX_SUGGESTIONS));

  const error = $derived.by(() => {
    if (pathError(query)) return pathError(query);
    if (insideSelf(target)) return 'Can’t move a folder into itself';
    if (target === current) return 'Already there';
    return '';
  });

  const info = $derived(folders.includes(target) ? `into ${target || rootName}/` : `will create ${target}/`);

  function go(path: string) {
    if (error && path === target) return;
    onmove(path);
  }

  function resetHighlight() {
    hi = 0;
    browsing = false;
  }

  function onkeydown(e: KeyboardEvent) {
    const highlighted = suggestions[hi];
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      if (browsing) hi = nextIndex(hi, suggestions.length);
      browsing = suggestions.length > 0;
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      hi = prevIndex(hi);
    } else if (e.key === 'Tab' && highlighted !== undefined) {
      e.preventDefault();
      query = highlighted;
    } else if (e.key === 'Enter') {
      e.preventDefault();
      const picked = browsing && highlighted !== undefined;
      go(picked ? highlighted : target);
    }
  }
</script>

<Panel label="Move" {onclose}>
  {#snippet head()}
    <span>Move {#if what}<span class="what">{what}</span>{:else}{kind}{/if} to</span><span class="k">{keyLabel('mod+shift+m')}</span>
  {/snippet}
  <div class="row">
    <span class="l">to</span>
    <input
      use:plainText
      use:focusOnMount
      aria-label="Destination folder"
      placeholder="folder path, empty = top level"
      bind:value={query}
      oninput={resetHighlight}
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
    font: var(--text-meta);
    color: var(--c-muted-2);
  }

  input {
    flex: 1;
    border: 0;
    outline: none;
    background: transparent;
    font: var(--text-code);
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
    font: var(--text-mono);
    color: var(--text-muted);
    cursor: pointer;
  }

  .sg.on,
  .sg:hover {
    background: var(--c-hover);
    color: var(--text);
  }
</style>
