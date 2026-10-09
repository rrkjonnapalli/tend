<script lang="ts">
  import { plainText } from '#lib/actions/plain-text.action.ts';
  // A list page's search line: one row under the page bar, the field at the right end.
  // While it's on screen, ⌘L (the "page.search" action) focuses it.
  import Icon from './Icon.svelte';
  import { pageActions } from '#lib/stores/page-actions.svelte.ts';

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

  let input = $state<HTMLInputElement>();

  function focusSearch() {
    input?.focus();
    input?.select();
  }

  $effect(() => pageActions.provide({ 'page.search': focusSearch }));
</script>

<div class="line">
  <label class="search">
    <span class="ic"><Icon name="search" size={14} /></span>
    <input use:plainText bind:this={input} aria-label={label} {placeholder} bind:value />
  </label>
</div>

<style>
  .line {
    flex: 0 0 auto;
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
    border-radius: var(--radius-control);
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
    font: var(--text-ui);
  }
</style>
