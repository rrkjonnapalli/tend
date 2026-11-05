<script lang="ts">
  // The top row of a phone list (MC-Notes, MD-People): the drawer button when the page has one, a
  // soft search field and +.
  import Icon from '../Icon.svelte';

  let {
    placeholder,
    query = $bindable(''),
    onadd,
    onmenu,
  }: { placeholder: string; query?: string; onadd: () => void; onmenu?: () => void } = $props();
</script>

<div class="top" class:menu={onmenu}>
  {#if onmenu}<button class="ib" aria-label="Folders" onclick={onmenu}><Icon name="list" size={20} /></button>{/if}
  <label class="fld">
    <Icon name="search" size={18} />
    <input type="search" {placeholder} aria-label={placeholder} autocapitalize="off" bind:value={query} />
  </label>
  <button class="ib" aria-label="New" onclick={onadd}><Icon name="plus" size={22} /></button>
</div>

<style>
  /* Sizes from docs/mobile/design/tend-mobile2.css (.qs). */
  .top {
    display: flex;
    align-items: center;
    gap: 6px;
    height: 56px;
    margin-right: -12px;
  }

  .top.menu {
    margin-left: -12px;
  }

  .fld {
    flex: 1;
    display: flex;
    align-items: center;
    gap: 8px;
    height: 36px;
    padding: 0 12px;
    border-radius: var(--radius-md);
    background: var(--c-chip);
    color: var(--c-muted-2);
  }

  .fld:focus-within {
    background: var(--c-hover);
  }

  .fld input {
    flex: 1;
    min-width: 0;
    border: 0;
    outline: 0;
    background: transparent;
    font: var(--text-ui);
    font-size: 15px;
    color: var(--text);
  }

  .ib {
    flex: none;
    display: grid;
    place-items: center;
    width: 44px;
    height: 44px;
    border: 0;
    border-radius: var(--radius-control);
    background: transparent;
    color: var(--text-muted);
  }

  .ib:active,
  .ib:focus-visible {
    outline: none;
    background: var(--c-hover);
  }
</style>
