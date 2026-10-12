<script lang="ts">
  import { plainText } from '#lib/actions/plain-text.action.ts';
  // One row of a colored list in Settings (statuses, labels): drag handle (unless pinned), color
  // swatch, name or inline rename, an optional note and fixed value, and edit / delete actions.
  import { focusAndSelect } from '#lib/actions/focus.action.ts';
  import Icon from './Icon.svelte';
  import Grip from './Grip.svelte';

  let {
    id,
    name,
    color,
    ring = false,
    pinned = false,
    note = '',
    meta = '',
    removable = true,
    nameLabel,
    editing,
    lifted,
    error,
    ongrab,
    onswatch,
    onedit,
    onrename,
    oncancel,
    onremove,
  }: {
    /** The row's key for drag reordering. */
    id: string;
    name: string;
    /** No swatch when left out (word kinds). */
    color?: string;
    /** Draw the swatch as a ring (the first status). */
    ring?: boolean;
    /** No drag handle: the row keeps its place. */
    pinned?: boolean;
    /** A small note after the name, e.g. "always first". */
    note?: string;
    /** A fixed value shown in mono, e.g. a status's value. */
    meta?: string;
    removable?: boolean;
    /** Accessible name of the rename field, e.g. "Status label". */
    nameLabel: string;
    editing: boolean;
    lifted: boolean;
    error: string | null;
    ongrab: (e: PointerEvent) => void;
    onswatch: (anchor: HTMLElement) => void;
    onedit: () => void;
    onrename: (label: string) => void;
    oncancel: () => void;
    onremove: () => void;
  } = $props();

  let label = $state('');

  function startEdit() {
    label = name;
    onedit();
  }

  function editKeys(e: KeyboardEvent) {
    if (e.key === 'Enter') onrename(label);
    if (e.key === 'Escape') {
      e.stopPropagation();
      oncancel();
    }
  }
</script>

<div class="srow reveal-on-hover" class:sortable={!pinned} class:lifted class:editing data-key={id}>
  {#if pinned}<span></span>{:else}
    <span class="handle" role="presentation" onpointerdown={ongrab}><Grip /></span>
  {/if}
  {#if color === undefined}<span></span>{:else}
    <button
      class="sw"
      class:ring
      style:--c={color}
      aria-label="Color for {name}"
      title="Color"
      aria-haspopup="listbox"
      onclick={(e) => onswatch(e.currentTarget)}></button>
  {/if}
  {#if editing}
    <input use:plainText class="etxt" aria-label={nameLabel} bind:value={label} onkeydown={editKeys} use:focusAndSelect />
  {:else}
    <span class="lab"><span class="nm">{name}</span><span class="pin">{note}</span></span>
  {/if}
  <span class="sval" title={meta ? "Value (can’t be changed)" : undefined}>{meta}</span>
  <span class="acts" class:on-hover={!editing}>
    {#if removable}
      <button class="icon-btn sm danger" aria-label="Delete {name}" title="Delete" onclick={onremove}>
        <Icon name="trash" size={13} />
      </button>
    {/if}
    {#if editing}
      <button class="icon-btn sm ok" aria-label="Save" title="Save" onclick={() => onrename(label)}><Icon name="check" size={13} stroke={2.25} /></button>
      <button class="icon-btn sm danger" aria-label="Cancel" title="Cancel" onclick={oncancel}><Icon name="close" size={13} stroke={2.25} /></button>
    {:else}
      <button class="icon-btn sm edit" aria-label="Edit {name}" title="Edit" onclick={startEdit}><Icon name="edit" size={13} /></button>
    {/if}
  </span>
  {#if error}<span class="err-line err" role="alert">{error}</span>{/if}
</div>

<style>
  .srow {
    position: relative;
    display: grid;
    grid-template-columns: 16px 14px minmax(0, 1fr) auto 84px;
    align-items: center;
    gap: 12px;
    min-height: 36px;
    padding: 0 16px;
    margin: 0 -16px;
    font: var(--text-ui);
    user-select: none;
  }

  .srow:hover {
    background: var(--c-hover);
  }

  .srow.editing {
    background: transparent;
  }

  .handle {
    position: relative;
    display: block;
    width: 16px;
    height: 28px;
  }

  .handle :global(.grip) {
    left: 0;
  }

  .sw {
    display: block;
    width: 14px;
    height: 14px;
    padding: 0;
    border: 0;
    border-radius: 7px;
    background: var(--c);
    cursor: pointer;
  }

  .sw.ring {
    background: transparent;
    box-shadow: inset 0 0 0 2px var(--c);
  }

  /* No outline or ring (the app never draws one): the swatch grows a little instead. */
  .sw:hover,
  .sw:focus-visible {
    transform: scale(1.2);
  }

  .lab {
    display: flex;
    align-items: baseline;
    gap: 10px;
    min-width: 0;
  }

  .nm {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .pin {
    font: var(--text-tiny);
    color: var(--c-muted-2);
  }

  .sval {
    font: var(--text-mono-meta);
    color: var(--c-muted-2);
    letter-spacing: 0.02em;
  }

  .acts {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 2px;
  }

  .edit {
    color: var(--primary);
  }

  .edit:hover {
    background: var(--primary-soft);
    color: var(--primary);
  }

  .etxt {
    min-width: 0;
    height: 28px;
    padding: 0 8px;
    margin-left: -8px;
    border: 0;
    border-radius: var(--radius-sm);
    background: var(--c-hover);
    outline: none;
    font: var(--text-ui);
  }

  .err {
    grid-column: 3 / -1;
    padding: 0 0 8px;
  }
</style>
