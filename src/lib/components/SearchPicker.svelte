<script lang="ts" generics="T extends { id: string }">
  // A search field over a list, inside a popover: type to filter, ↑↓ to move, Enter to pick.
  import { plainText } from '#lib/actions/plain-text.action.ts';
  import { focusOnMount } from '#lib/actions/focus.action.ts';
  import type { Snippet } from 'svelte';
  import { nextIndex, prevIndex } from '#lib/util/list-nav.ts';

  let {
    items,
    match,
    onpick,
    placeholder,
    label,
    empty = 'No matches',
    row,
    lead,
    max = 8,
  }: {
    items: T[];
    /** True when the item matches the lower-cased query. */
    match: (item: T, query: string) => boolean;
    onpick: (item: T) => void;
    placeholder: string;
    label: string;
    empty?: string;
    row: Snippet<[T]>;
    /** Rows above the matches (e.g. "Unassign"). */
    lead?: Snippet;
    max?: number;
  } = $props();

  let query = $state('');
  let hi = $state(0);
  const needle = $derived(query.trim().toLowerCase());
  const shown = $derived(items.filter((item) => match(item, needle)).slice(0, max));

  function onkeydown(e: KeyboardEvent) {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      hi = nextIndex(hi, shown.length);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      hi = prevIndex(hi);
    } else if (e.key === 'Enter' && shown[hi]) {
      e.preventDefault();
      onpick(shown[hi]);
    }
  }
</script>

<div class="picker">
  <input use:plainText use:focusOnMount {placeholder} aria-label={label} bind:value={query} oninput={() => (hi = 0)} {onkeydown} />
  {@render lead?.()}
  <div role="listbox" aria-label={label}>
    {#each shown as item, i (item.id)}
      <button class="po" class:on={i === hi} role="option" aria-selected={i === hi} onclick={() => onpick(item)} onmouseenter={() => (hi = i)}>
        {@render row(item)}
      </button>
    {/each}
    {#if shown.length === 0}<div class="none">{empty}</div>{/if}
  </div>
</div>

<style>
  .picker {
    width: 280px;
    display: flex;
    flex-direction: column;
  }

  input {
    width: 100%;
    height: 34px;
    padding: 0 12px;
    border: 0;
    border-bottom: 1px solid var(--c-chip);
    outline: none;
    font: var(--text-ui);
  }

  .po {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    min-height: 32px;
    padding: 0 12px;
    border: 0;
    background: transparent;
    text-align: left;
    font: var(--text-ui);
    color: var(--text);
    cursor: pointer;
  }

  .po.on {
    background: var(--c-cursor);
  }

  .po :global(.sub) {
    margin-left: auto;
    font-size: 12px;
    color: var(--c-muted-2);
  }

  .none {
    padding: 8px 12px;
    font: var(--text-caption);
    color: var(--c-muted-2);
  }
</style>
