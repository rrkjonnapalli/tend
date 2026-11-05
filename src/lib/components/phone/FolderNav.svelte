<script lang="ts">
  // The top of a phone explorer (MC-Notes, MC-Tasks): the drawer button (the folder tree), search
  // and +, then ↑ and the path when inside a folder. Spec: tend-mobile-spec.md §5.
  import Icon from '../Icon.svelte';
  import PhoneSearch from './PhoneSearch.svelte';
  import { parentOf } from '#lib/util/names.ts';
  import { ui } from '#lib/stores/ui.svelte.ts';

  let {
    rootName,
    folder,
    hrefOf,
    placeholder,
    query = $bindable(''),
    onadd,
  }: {
    rootName: string;
    folder: string;
    hrefOf: (path: string) => string;
    placeholder: string;
    query?: string;
    onadd: () => void;
  } = $props();

  const parts = $derived(folder ? folder.split('/') : []);
</script>

<PhoneSearch {placeholder} bind:query {onadd} onmenu={() => (ui.drawer = true)} />

{#if folder}
  <div class="loc">
    <a class="ib" href={hrefOf(parentOf(folder))} aria-label="Up one folder"><Icon name="up" size={20} /></a>
    <span class="crumb">
      {rootName}
      {#each parts as part, i (i)}
        <span class="sl">/</span>{#if i === parts.length - 1}<b>{part}</b>{:else}{part}{/if}
      {/each}
    </span>
  </div>
{/if}

<style>
  /* Sizes from docs/mobile/design/tend-mobile2.css (.loc). */
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

  .loc {
    display: flex;
    align-items: center;
    gap: 4px;
    height: 44px;
    margin: 0 -12px 2px;
  }

  .crumb {
    min-width: 0;
    display: flex;
    gap: 6px;
    font: var(--text-caption);
    font-size: 14px;
    color: var(--c-muted-2);
    white-space: nowrap;
    overflow: hidden;
  }

  .crumb b {
    color: var(--text);
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .sl {
    color: var(--c-faint);
  }

  .ib:active,
  .ib:focus-visible {
    outline: none;
    background: var(--c-hover);
  }
</style>
