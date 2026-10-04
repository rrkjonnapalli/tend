<script lang="ts">
  // Settings → Tasks → Statuses: drag order (not first or last), colour swatch, rename inline,
  // add with an auto UPPER_SNAKE value, delete blocked while in use.
  import { onDestroy } from 'svelte';
  import Section from './Section.svelte';
  import ColorListRow from './ColorListRow.svelte';
  import ColorListAddRow from './ColorListAddRow.svelte';
  import SwatchPicker from './SwatchPicker.svelte';
  import { statusesAPI } from '#lib/api/statuses.ts';
  import type { Status, StatusPosition } from '#lib/types/status.type.ts';
  import { messageOf } from '#lib/errors/app-error.ts';
  import { tasks } from '#lib/stores/tasks.svelte.ts';
  import { startReorder } from '#lib/util/reorder.ts';
  import { FIRST_STATUS, statusValue } from '#lib/util/statuses.ts';
  import { freeSwatch } from '#lib/util/swatches.ts';

  tasks.loadStatuses();

  let preview = $state<string[] | null>(null);
  let lifted = $state<string | null>(null);
  let editing = $state<string | null>(null);
  let rowError = $state<{ value: string; message: string } | null>(null);
  let swatchFor = $state<{ value: string; anchor: HTMLElement } | null>(null);

  /** The statuses in the order being dragged, or as saved. */
  function inOrder(order: string[]): Status[] {
    const found = order.map((v) => tasks.statuses.find((s) => s.value === v));
    return found.filter((s) => s !== undefined);
  }

  const list = $derived(preview ? inOrder(preview) : tasks.statuses);
  const first = $derived(tasks.statuses[0]?.label ?? 'Not started');
  const last = $derived(tasks.statuses.at(-1)?.label ?? 'Done');
  const swatchColor = $derived(tasks.statuses.find((s) => s.value === swatchFor?.value)?.color);

  const PIN_NOTES: Record<StatusPosition, string> = { first: 'always first', middle: '', last: 'always last' };

  function positionOf(index: number): StatusPosition {
    if (index === 0) return 'first';
    if (index === list.length - 1) return 'last';
    return 'middle';
  }

  /** Runs a change and reloads the statuses; returns why it failed, or null. */
  async function apply(action: () => Promise<unknown>): Promise<string | null> {
    try {
      await action();
      await tasks.loadStatuses();
      rowError = null;
      return null;
    } catch (e) {
      return messageOf(e);
    }
  }

  /** A change to one row; a failure shows under that row. */
  async function runRow(value: string, action: () => Promise<unknown>): Promise<boolean> {
    const failure = await apply(action);
    if (failure) rowError = { value, message: failure };
    return failure === null;
  }

  function addStatus({ name, value }: { name: string; value: string }) {
    const color = freeSwatch(tasks.statuses.map((s) => s.color));
    return apply(() => statusesAPI.add({ label: name, value, color }));
  }

  function startEdit(value: string) {
    editing = value;
    rowError = null;
  }

  async function rename(value: string, label: string) {
    const ok = await runRow(value, () => statusesAPI.rename({ value, label }));
    if (ok) editing = null;
  }

  function cancelEdit() {
    editing = null;
    rowError = null;
  }

  async function pickColor(hex: string) {
    if (!swatchFor) return;
    const { value } = swatchFor;
    swatchFor = null;
    await runRow(value, () => statusesAPI.recolor({ value, color: hex }));
  }

  function showPreview(order: string[] | null) {
    preview = order;
    if (!order) lifted = null;
  }

  let stopReorder: (() => void) | null = null;
  onDestroy(() => stopReorder?.());

  function grab(e: PointerEvent, value: string) {
    e.preventDefault();
    const keys = tasks.statuses.map((s) => s.value);
    lifted = value;
    editing = null;
    stopReorder = startReorder(e, {
      keys,
      key: value,
      min: 1,
      max: keys.length - 2,
      preview: showPreview,
      drop: (order) => runRow(value, () => statusesAPI.reorder(order)),
    });
  }
</script>

<Section title="Statuses" extra={String(tasks.statuses.length)} indent={false}>
  <span class="caption muted intro">
    Each status is a column on the board; its color is used for the status dot everywhere. {first} is always first and {last} always last; new
    tasks start as {first}.
  </span>
  <div class="rows">
    {#each list as s, i (s.value)}
      <ColorListRow
        id={s.value}
        name={s.label}
        color={s.color}
        ring={s.value === FIRST_STATUS}
        pinned={positionOf(i) !== 'middle'}
        note={PIN_NOTES[positionOf(i)]}
        meta={s.value}
        removable={!s.isDefault}
        nameLabel="Status label"
        editing={editing === s.value}
        lifted={lifted === s.value && preview !== null}
        error={rowError?.value === s.value ? rowError.message : null}
        ongrab={(e) => grab(e, s.value)}
        onswatch={(anchor) => (swatchFor = { value: s.value, anchor })}
        onedit={() => startEdit(s.value)}
        onrename={(label) => rename(s.value, label)}
        oncancel={cancelEdit}
        onremove={() => runRow(s.value, () => statusesAPI.remove(s.value))} />
    {/each}
  </div>
  <ColorListAddRow what="status" toValue={statusValue} onadd={addStatus} />
</Section>

{#if swatchFor}
  <SwatchPicker anchor={swatchFor.anchor} color={swatchColor} label="Status color" onpick={pickColor} onclose={() => (swatchFor = null)} />
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
</style>
