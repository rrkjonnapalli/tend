<script lang="ts">
  // One status in Settings → Statuses: drag handle (not first or last), colour swatch, label or
  // inline rename, the fixed value, and edit / delete actions.
  import { focusAndSelect } from '#lib/actions/focus.action.ts';
  import Icon from './Icon.svelte';
  import Grip from './Grip.svelte';
  import { FIRST_STATUS } from '#lib/util/statuses.ts';
  import type { Status, StatusPosition } from '#lib/types/status.type.ts';

  let {
    status,
    position,
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
    status: Status;
    position: StatusPosition;
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

  const PIN_LABELS: Record<StatusPosition, string> = { first: 'always first', middle: '', last: 'always last' };

  const pinned = $derived(position !== 'middle');
  let label = $state('');

  function startEdit() {
    label = status.label;
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

<div class="srow reveal-on-hover" class:sortable={!pinned} class:lifted class:editing data-key={status.value}>
  {#if pinned}<span></span>{:else}
    <span class="handle" role="presentation" onpointerdown={ongrab}><Grip /></span>
  {/if}
  <button
    class="sw"
    class:ring={status.value === FIRST_STATUS}
    style:--c={status.color}
    aria-label="Color for {status.label}"
    title="Color"
    aria-haspopup="listbox"
    onclick={(e) => onswatch(e.currentTarget)}></button>
  {#if editing}
    <input class="etxt" aria-label="Status label" bind:value={label} onkeydown={editKeys} use:focusAndSelect />
  {:else}
    <span class="lab"><span class="nm">{status.label}</span><span class="pin">{PIN_LABELS[position]}</span></span>
  {/if}
  <span class="sval" title="Value (can’t be changed)">{status.value}</span>
  <span class="acts" class:on-hover={!editing}>
    {#if !status.isDefault}
      <button class="icon-btn sm danger" aria-label="Delete {status.label}" title="Delete" onclick={onremove}>
        <Icon name="trash" size={13} />
      </button>
    {/if}
    {#if editing}
      <button class="icon-btn sm ok" aria-label="Save" title="Save" onclick={() => onrename(label)}><Icon name="check" size={13} stroke={2.25} /></button>
      <button class="icon-btn sm danger" aria-label="Cancel" title="Cancel" onclick={oncancel}><Icon name="close" size={13} stroke={2.25} /></button>
    {:else}
      <button class="icon-btn sm edit" aria-label="Edit {status.label}" title="Edit" onclick={startEdit}><Icon name="edit" size={13} /></button>
    {/if}
  </span>
  {#if error}<span class="err" role="alert">{error}</span>{/if}
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
    font: 400 14px/20px var(--font-ui);
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

  .sw:hover {
    outline: 2px solid var(--border);
    outline-offset: 2px;
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
    font: 400 11px/16px var(--font-ui);
    color: var(--c-faint);
  }

  .sval {
    font: 400 12px/16px var(--font-mono);
    color: var(--c-faint);
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
    font: 400 14px/20px var(--font-ui);
  }

  .err {
    grid-column: 3 / -1;
    padding: 0 0 8px;
    font: 400 12px/16px var(--font-ui);
    color: var(--danger);
  }
</style>
