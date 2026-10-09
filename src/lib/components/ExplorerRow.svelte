<script lang="ts">
  import { plainText } from '#lib/actions/plain-text.action.ts';
  // One line of the explorer tree, with its inline name editor and the error under it.
  import type { DropState, ExplorerRow } from '#lib/types/explorer.type.ts';
  import Icon from './Icon.svelte';
  import { focusAndSelect } from '#lib/actions/focus.action.ts';
  import { dropPath, indentOf } from '#lib/util/explorer-rows.ts';

  let {
    row,
    selected,
    drop,
    ghosted,
    expanded,
    editing,
    draft = $bindable(''),
    draftError,
    hasMenu,
    menuOpen,
    onpress,
    onmenu,
    onopen,
    ontoggle,
    oneditkey,
    oneditblur,
  }: {
    row: ExplorerRow;
    selected: boolean;
    /** Whether a dragged row may land here; null when nothing is over this row. */
    drop: DropState;
    /** This row is the one being dragged. */
    ghosted: boolean;
    expanded: boolean;
    editing: boolean;
    draft: string;
    draftError: string;
    hasMenu: boolean;
    menuOpen: boolean;
    onpress: (e: PointerEvent) => void;
    onmenu: (e: MouseEvent) => void;
    onopen: () => void;
    ontoggle: (e: Event) => void;
    oneditkey: (e: KeyboardEvent) => void;
    oneditblur: () => void;
  } = $props();

  const indent = $derived(indentOf(row));
  const showsError = $derived(editing && !!draftError && !!draft.trim());
  const isOpen = $derived(expanded && !row.isNew);

  function onclick() {
    if (!row.isFolder) onopen();
  }

  function onChevron(e: Event) {
    if (!row.isNew) ontoggle(e);
  }

  const stop = (e: Event) => e.stopPropagation();
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<div
  class="row"
  class:leaf={!row.isFolder}
  class:sel={selected}
  class:drop={drop === 'allowed'}
  class:bad={drop === 'blocked'}
  class:ghosted
  class:editing
  style:padding-left="{indent}px"
  data-drop={dropPath(row)}
  role="treeitem"
  tabindex="-1"
  aria-selected={selected}
  aria-expanded={row.isFolder ? expanded : undefined}
  onpointerdown={onpress}
  oncontextmenu={onmenu}
  {onclick}>
  {#if row.isFolder}
    <button class="chev" aria-label="Expand or collapse folder" onclick={onChevron}>
      <Icon name={isOpen ? 'chevronDown' : 'chevronRight'} size={10} stroke={3} />
    </button>
  {/if}
  {#if editing}
    <input
      use:plainText
      class="inline-field ein"
      class:bad={showsError}
      aria-label="Name"
      bind:value={draft}
      onkeydown={oneditkey}
      onblur={oneditblur}
      onpointerdown={stop}
      use:focusAndSelect />
  {:else if row.isFolder}
    <button class="nm" onclick={onopen}>{row.name}</button>
  {:else}
    <span class="nm">{row.name}</span>
  {/if}
  {#if hasMenu && !editing}
    <button class="more" class:on={menuOpen} aria-label="More actions" title="More" onpointerdown={stop} onclick={onmenu}>
      <Icon name="more" size={14} />
    </button>
  {/if}
</div>
{#if showsError}
  <div class="name-err" style:padding-left="{indent}px">{draftError}</div>
{/if}

<style>
  .row {
    position: relative;
    display: flex;
    align-items: center;
    gap: 6px;
    flex: 0 0 28px;
    padding-right: 6px;
    font: var(--text-ui);
    color: var(--text);
    cursor: pointer;
    touch-action: none;
  }

  .row.leaf {
    color: var(--text-muted);
  }

  .row:hover {
    background: var(--c-hover);
  }

  .row.sel {
    background: var(--c-selected);
    color: var(--text);
  }

  /* The name field carries the fill while renaming. */
  .row.editing,
  .row.editing:hover {
    background: transparent;
  }

  .row.drop,
  :global(.dragging) .row.drop {
    background: var(--primary-soft);
    box-shadow: inset 0 0 0 1px var(--primary);
  }

  .row.bad {
    background: var(--danger-soft);
  }

  .row.ghosted {
    opacity: 0.4;
  }

  :global(.dragging) .row:hover:not(.drop):not(.bad) {
    background: transparent;
  }

  .chev {
    flex: 0 0 10px;
    width: 10px;
    height: 28px;
    padding: 0;
    border: 0;
    background: transparent;
    color: var(--c-muted-2);
    display: grid;
    place-items: center;
    cursor: pointer;
  }

  .nm {
    flex: 1;
    min-width: 0;
    height: 28px;
    padding: 0;
    border: 0;
    background: transparent;
    text-align: left;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font: inherit;
    color: inherit;
    cursor: pointer;
    line-height: 28px;
  }

  .more {
    width: 22px;
    height: 22px;
    padding: 0;
    border: 0;
    border-radius: var(--radius-sm);
    background: transparent;
    color: var(--c-muted-2);
    display: grid;
    place-items: center;
    cursor: pointer;
    visibility: hidden;
  }

  .row:hover .more,
  .more.on,
  .more:focus-visible {
    visibility: visible;
  }

  .more:hover,
  .more.on {
    background: var(--border);
    color: var(--text);
  }

  :global(.dragging) .more {
    visibility: hidden;
  }

  .ein {
    flex: 1;
    height: 24px;
    margin-left: -8px;
  }

  .name-err {
    padding: 2px 8px 6px;
    font: var(--text-meta);
    color: var(--danger);
  }
</style>
