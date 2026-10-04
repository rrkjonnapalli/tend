<script lang="ts" generics="T extends { id: string }">
  // A search field over a list, inside a popover: type to filter, ↑↓ to move, Enter to pick.
  import type { Snippet } from 'svelte';

  let {
    items,
    match,
    onpick,
    placeholder,
    label,
    empty = 'No matches',
    row,
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
    max?: number;
  } = $props();

  let query = $state('');
  let hi = $state(0);
  const shown = $derived(items.filter((i) => match(i, query.trim().toLowerCase())).slice(0, max));

  function onkeydown(e: KeyboardEvent) {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      hi = Math.min(hi + 1, shown.length - 1);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      hi = Math.max(hi - 1, 0);
    } else if (e.key === 'Enter' && shown[hi]) {
      e.preventDefault();
      onpick(shown[hi]);
    }
  }
</script>

<div class="picker">
  <!-- svelte-ignore a11y_autofocus -->
  <input {placeholder} aria-label={label} autofocus bind:value={query} oninput={() => (hi = 0)} {onkeydown} />
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
    font: 400 14px/20px var(--font-ui);
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
    font: 400 14px/20px var(--font-ui);
    color: var(--text);
    cursor: pointer;
  }

  .po.on {
    background: var(--c-hover);
  }

  .po :global(.sub) {
    margin-left: auto;
    font-size: 12px;
    color: var(--c-muted-2);
  }

  .none {
    padding: 8px 12px;
    font: 400 13px/18px var(--font-ui);
    color: var(--c-muted-2);
  }
</style>
