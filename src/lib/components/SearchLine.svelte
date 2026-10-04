<script lang="ts">
  // A list page's search line: one row under the page bar, the field at the right end.
  import Icon from './Icon.svelte';

  let {
    value = $bindable(''),
    label,
    placeholder,
  }: {
    value?: string;
    /** Accessible name, e.g. "Search tasks". */
    label: string;
    placeholder: string;
  } = $props();
</script>

<div class="line">
  <label class="search">
    <span class="ic"><Icon name="search" size={14} /></span>
    <input aria-label={label} {placeholder} bind:value />
  </label>
</div>

<style>
  .line {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    min-height: 40px;
  }

  .search {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    max-width: 320px;
    height: 32px;
    padding: 0 10px;
    border: 1px solid var(--border);
    border-radius: 6px;
  }

  .search:focus-within {
    border-color: var(--primary);
  }

  .ic {
    display: grid;
    color: var(--c-muted-2);
  }

  .search input {
    flex: 1;
    min-width: 0;
    border: 0;
    outline: none;
    background: transparent;
    font: 400 14px/20px var(--font-ui);
  }
</style>
