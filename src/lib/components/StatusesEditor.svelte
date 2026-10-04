<script lang="ts">
  // Settings → Tasks → Statuses: drag order (not first or last), colour swatch, rename inline,
  // add with an auto UPPER_SNAKE value, delete blocked while in use.
  import Icon from './Icon.svelte';
  import Grip from './Grip.svelte';
  import Popover from './Popover.svelte';
  import Section from './Section.svelte';
  import { statusesAPI } from '#lib/api/statuses.ts';
  import { statusValue, SWATCHES, FIRST_STATUS } from '#lib/util/statuses.ts';
  import type { Status } from '#lib/types/status.type.ts';
  import { isAppError, messageOf } from '#lib/api/core.ts';
  import { tasks } from '#lib/stores/tasks.svelte.ts';
  import { startReorder } from '#lib/util/reorder.ts';

  tasks.loadStatuses();

  let preview = $state<string[] | null>(null);
  let lifted = $state<string | null>(null);
  const list = $derived(
    preview ? preview.map((v) => tasks.statuses.find((s) => s.value === v)!).filter(Boolean) : tasks.statuses,
  );
  const first = $derived(tasks.statuses[0]?.label ?? 'Not started');
  const last = $derived(tasks.statuses.at(-1)?.label ?? 'Done');

  let editing = $state<string | null>(null);
  let edLabel = $state('');
  let rowError = $state<{ value: string; message: string } | null>(null);
  let swatchFor = $state<{ value: string; anchor: HTMLElement } | null>(null);

  let newLabel = $state('');
  let newValue = $state('');
  let valueTouched = $state(false);
  let addError = $state('');

  async function run(value: string | null, action: () => Promise<unknown>): Promise<boolean> {
    try {
      await action();
      await tasks.loadStatuses();
      rowError = null;
      return true;
    } catch (e) {
      const message = isAppError(e) ? e.message : messageOf(e);
      if (value) rowError = { value, message };
      else addError = message;
      return false;
    }
  }

  function startEdit(s: Status) {
    editing = s.value;
    edLabel = s.label;
    rowError = null;
  }

  async function saveEdit() {
    const v = editing;
    if (!v) return;
    if (await run(v, () => statusesAPI.rename({ value: v, label: edLabel }))) editing = null;
  }

  function cancelEdit() {
    editing = null;
    rowError = null;
  }

  async function pickColor(value: string, color: string) {
    swatchFor = null;
    await run(value, () => statusesAPI.recolor({ value, color }));
  }

  async function add() {
    addError = '';
    const label = newLabel.trim();
    const value = statusValue(newValue || label);
    if (!label) return (addError = 'Give the status a label.');
    if (!value) return (addError = 'Give the status a value.');
    const used = new Set(tasks.statuses.map((s) => s.color));
    const color = (SWATCHES.find(([, hex]) => !used.has(hex)) ?? SWATCHES[0])[1];
    if (await run(null, () => statusesAPI.add({ label, value, color }))) {
      newLabel = '';
      newValue = '';
      valueTouched = false;
    }
  }

  function grab(e: PointerEvent, value: string) {
    e.preventDefault();
    const keys = tasks.statuses.map((s) => s.value);
    lifted = value;
    editing = null;
    startReorder(e, {
      keys,
      key: value,
      min: 1,
      max: keys.length - 2,
      preview: (p) => {
        preview = p;
        if (!p) lifted = null;
      },
      drop: (order) => run(value, () => statusesAPI.reorder(order)),
    });
  }

  function addKeys(e: KeyboardEvent) {
    if (e.key === 'Enter') add();
  }

  function editKeys(e: KeyboardEvent) {
    if (e.key === 'Enter') saveEdit();
    if (e.key === 'Escape') {
      e.stopPropagation();
      cancelEdit();
    }
  }

  function focusOnMount(node: HTMLInputElement) {
    node.focus();
    node.select();
  }
</script>

<Section title="Statuses" extra={String(tasks.statuses.length)} indent={false}>
  <span class="caption muted intro">
    Each status is a column on the board; its color is used for the status dot everywhere. {first} is always first and {last} always last; new
    tasks start as {first}.
  </span>
  <div class="rows">
    {#each list as s, i (s.value)}
      {@const pinned = i === 0 || i === list.length - 1}
      {@const isEditing = editing === s.value}
      <div class="srow reveal-on-hover" class:sortable={!pinned} class:lifted={lifted === s.value && preview !== null} class:editing={isEditing} data-key={s.value}>
        {#if pinned}<span></span>{:else}
          <span class="handle" role="presentation" onpointerdown={(e) => grab(e, s.value)}><Grip /></span>
        {/if}
        <button
          class="sw"
          class:ring={s.value === FIRST_STATUS}
          style:--c={s.color}
          aria-label="Color for {s.label}"
          title="Color"
          aria-haspopup="listbox"
          onclick={(e) => (swatchFor = { value: s.value, anchor: e.currentTarget as HTMLElement })}></button>
        {#if isEditing}
          <input class="etxt" aria-label="Status label" bind:value={edLabel} onkeydown={editKeys} use:focusOnMount />
        {:else}
          <span class="lab"><span class="nm">{s.label}</span><span class="pin">{i === 0 ? 'always first' : pinned ? 'always last' : ''}</span></span>
        {/if}
        <span class="sval" title="Value (can’t be changed)">{s.value}</span>
        <span class="acts" class:on-hover={!isEditing}>
          {#if !s.isDefault}
            <button class="icon-btn sm danger" aria-label="Delete {s.label}" title="Delete" onclick={() => run(s.value, () => statusesAPI.remove(s.value))}>
              <Icon name="trash" size={13} />
            </button>
          {/if}
          {#if isEditing}
            <button class="icon-btn sm ok" aria-label="Save" title="Save" onclick={saveEdit}><Icon name="check" size={13} stroke={2.25} /></button>
            <button class="icon-btn sm danger" aria-label="Cancel" title="Cancel" onclick={cancelEdit}><Icon name="close" size={13} stroke={2.25} /></button>
          {:else}
            <button class="icon-btn sm edit" aria-label="Edit {s.label}" title="Edit" onclick={() => startEdit(s)}><Icon name="edit" size={13} /></button>
          {/if}
        </span>
        {#if rowError?.value === s.value}<span class="err" role="alert">{rowError.message}</span>{/if}
      </div>
    {/each}
  </div>
  <div class="addrow">
    <span class="plus">+</span>
    <span></span>
    <input aria-label="New status label" placeholder="add status" bind:value={newLabel} oninput={() => !valueTouched && (newValue = statusValue(newLabel))} onkeydown={addKeys} />
    <input
      class="vin"
      aria-label="New status value"
      placeholder="VALUE"
      bind:value={newValue}
      oninput={() => ((valueTouched = true), (newValue = newValue.toUpperCase()))}
      onkeydown={addKeys} />
    <span class="acts">
      {#if newLabel.trim()}<button class="icon-btn sm ok" aria-label="Add status" title="Add" onclick={add}><Icon name="check" size={13} stroke={2.25} /></button>{/if}
    </span>
  </div>
  {#if addError}<span class="adderr" role="alert">{addError}</span>{/if}
</Section>

{#if swatchFor}
  {@const target = tasks.statuses.find((s) => s.value === swatchFor!.value)}
  <Popover anchor={swatchFor.anchor} role="listbox" label="Status color" onclose={() => (swatchFor = null)}>
    <div class="swpop">
      {#each SWATCHES as [name, hex] (hex)}
        <button
          class="swopt"
          class:on={target?.color.toLowerCase() === hex}
          style:background={hex}
          role="option"
          aria-selected={target?.color.toLowerCase() === hex}
          aria-label={name}
          title={name}
          onclick={() => pickColor(swatchFor!.value, hex)}></button>
      {/each}
    </div>
  </Popover>
{/if}

<style>
  .intro {
    display: block;
    margin-bottom: 4px;
  }

  .rows {
    display: flex;
    flex-direction: column;
    padding-top: 4px;
  }

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

  .addrow {
    display: grid;
    grid-template-columns: 16px 14px minmax(0, 1fr) 150px 84px;
    align-items: center;
    gap: 12px;
    min-height: 36px;
    font: 400 14px/20px var(--font-ui);
    color: var(--c-muted-2);
  }

  .plus {
    text-align: center;
  }

  .addrow input {
    min-width: 0;
    border: 0;
    outline: none;
    background: transparent;
    font: 400 14px/20px var(--font-ui);
    color: var(--text);
  }

  .addrow input::placeholder {
    color: var(--c-muted-2);
  }

  .addrow .vin {
    font: 400 12px/16px var(--font-mono);
    color: var(--text-muted);
    letter-spacing: 0.02em;
  }

  .addrow .vin::placeholder {
    color: var(--c-faint);
  }

  .adderr {
    padding: 0 0 0 54px;
    font: 400 12px/16px var(--font-ui);
    color: var(--danger);
  }

  .swpop {
    display: grid;
    grid-template-columns: repeat(5, 18px);
    gap: 8px;
    padding: 10px;
  }

  .swopt {
    width: 18px;
    height: 18px;
    padding: 0;
    border: 0;
    border-radius: 9px;
    cursor: pointer;
  }

  .swopt:hover {
    outline: 2px solid var(--c-faint);
    outline-offset: 2px;
  }

  .swopt.on {
    outline: 2px solid var(--text);
    outline-offset: 2px;
  }
</style>
