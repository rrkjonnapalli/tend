<script lang="ts">
  // The top of a phone explorer (MC-Notes, MC-Tasks): the drawer button (the whole tree), search
  // and +, then ↑ and the path when inside a folder, then the collapsible Folders section with the
  // folders directly inside this one; tapping one opens it. Spec: tend-mobile-spec.md §5.
  import Icon from '../Icon.svelte';
  import PhoneSearch from './PhoneSearch.svelte';
  import { ui } from '#lib/stores/ui.svelte.ts';
  import { baseOf, parentOf } from '#lib/util/names.ts';

  let {
    rootName,
    folder,
    subfolders,
    hrefOf,
    placeholder,
    query = $bindable(''),
    open = $bindable(true),
    onadd,
  }: {
    rootName: string;
    folder: string;
    /** The folders directly inside `folder`. */
    subfolders: string[];
    hrefOf: (path: string) => string;
    placeholder: string;
    query?: string;
    open?: boolean;
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

{#if subfolders.length > 0}
  <button class="p-section" aria-expanded={open} onclick={() => (open = !open)}>
    <Icon name={open ? 'chevronDown' : 'chevronRight'} size={14} stroke={2.5} />Folders<span class="n">{subfolders.length}</span>
  </button>
  {#if open}
    {#each subfolders as path (path)}
      <a class="fr" href={hrefOf(path)}>
        <span class="fi"><Icon name="folder" size={18} /></span><span class="tt">{baseOf(path)}</span>
        <span class="ch"><Icon name="chevronRight" size={14} /></span>
      </a>
    {/each}
  {/if}
  <div class="gap"></div>
{/if}

<style>
  /* Sizes from docs/mobile/design/tend-mobile2.css (.qs, .loc, .fs, .fr). */

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

  .fr {
    display: flex;
    align-items: center;
    gap: 6px;
    width: calc(100% + 20px);
    height: 44px;
    margin: 0 -10px;
    padding: 0 10px;
    border: 0;
    border-radius: var(--radius-control);
    background: transparent;
    text-align: left;
    color: var(--text);
    text-decoration: none;
  }

  .fr {
    gap: 10px;
    padding-left: 34px;
    font: var(--text-ui);
    font-size: 15px;
  }

  .fi {
    display: grid;
    color: var(--c-muted-2);
  }

  .tt {
    flex: 1;
  }

  .ch {
    display: grid;
    color: var(--c-faint);
  }

  .gap {
    height: 12px;
  }

  .fr:active,
  .ib:active {
    background: var(--c-hover);
  }

  .fr:focus-visible,
  .ib:focus-visible {
    outline: none;
    background: var(--c-hover);
  }
</style>
