<script lang="ts">
  // Settings → Tasks → Labels: the fixed list tasks pick their labels from. Drag to order, color
  // swatch, rename inline, add; deleting a label in use asks first, then takes it off those tasks.
  import { onDestroy } from 'svelte';
  import Section from './Section.svelte';
  import ColorListRow from './ColorListRow.svelte';
  import ColorListAddRow from './ColorListAddRow.svelte';
  import SwatchPicker from './SwatchPicker.svelte';
  import ConfirmDialog from './ConfirmDialog.svelte';
  import { labelsAPI } from '#lib/api/labels.ts';
  import type { Label } from '#lib/types/label.type.ts';
  import { messageOf } from '#lib/errors/app-error.ts';
  import { tasks } from '#lib/stores/tasks.svelte.ts';
  import { startReorder } from '#lib/util/reorder.ts';
  import { freeSwatch } from '#lib/util/swatches.ts';
  import { plural } from '#lib/util/plural.ts';
  import { statusValue } from '#lib/util/statuses.ts';

  tasks.loadLabels();

  let preview = $state<string[] | null>(null);
  let lifted = $state<string | null>(null);
  let editing = $state<string | null>(null);
  let rowError = $state<{ id: string; message: string } | null>(null);
  let swatchFor = $state<{ id: string; anchor: HTMLElement } | null>(null);
  let deleting = $state<{ label: Label; uses: number } | null>(null);

  /** The labels in the order being dragged, or as saved. */
  function inOrder(order: string[]): Label[] {
    const found = order.map((id) => tasks.labels.find((l) => l.id === id));
    return found.filter((l) => l !== undefined);
  }

  const list = $derived(preview ? inOrder(preview) : tasks.labels);
  const swatchColor = $derived(tasks.labels.find((l) => l.id === swatchFor?.id)?.color);

  /** Runs a change and reloads the labels (and the tasks' labels); returns why it failed, or null. */
  async function apply(action: () => Promise<unknown>): Promise<string | null> {
    try {
      await action();
      await tasks.load();
      rowError = null;
      return null;
    } catch (e) {
      return messageOf(e);
    }
  }

  /** A change to one row; a failure shows under that row. */
  async function runRow(id: string, action: () => Promise<unknown>): Promise<boolean> {
    const failure = await apply(action);
    if (failure) rowError = { id, message: failure };
    return failure === null;
  }

  function addLabel({ name, value }: { name: string; value: string }) {
    const color = freeSwatch(tasks.labels.map((l) => l.color));
    return apply(() => labelsAPI.add({ name, value, color }));
  }

  function startEdit(id: string) {
    editing = id;
    rowError = null;
  }

  async function rename(id: string, name: string) {
    const ok = await runRow(id, () => labelsAPI.rename({ id, name }));
    if (ok) editing = null;
  }

  function cancelEdit() {
    editing = null;
    rowError = null;
  }

  async function pickColor(hex: string) {
    if (!swatchFor) return;
    const { id } = swatchFor;
    swatchFor = null;
    await runRow(id, () => labelsAPI.recolor({ id, color: hex }));
  }

  /** A label no task carries goes at once; one in use asks first. */
  async function askRemove(label: Label) {
    const uses = await labelsAPI.usage(label.id);
    if (uses === 0) {
      await runRow(label.id, () => labelsAPI.remove(label.id));
      return;
    }
    deleting = { label, uses };
  }

  async function confirmRemove() {
    if (!deleting) return;
    const { id } = deleting.label;
    deleting = null;
    await runRow(id, () => labelsAPI.remove(id));
  }

  function showPreview(order: string[] | null) {
    preview = order;
    if (!order) lifted = null;
  }

  let stopReorder: (() => void) | null = null;
  onDestroy(() => stopReorder?.());

  function grab(e: PointerEvent, id: string) {
    e.preventDefault();
    lifted = id;
    editing = null;
    stopReorder = startReorder(e, {
      keys: tasks.labels.map((l) => l.id),
      key: id,
      preview: showPreview,
      drop: (order) => runRow(id, () => labelsAPI.reorder(order)),
    });
  }
</script>

<Section title="Labels" extra={String(tasks.labels.length)} indent={false}>
  <span class="caption muted intro">
    Labels mark tasks beyond their status, like “pending approval” or “script”. A task can carry several; it picks them from this list.
  </span>
  <div class="rows">
    {#each list as l (l.id)}
      <ColorListRow
        id={l.id}
        name={l.name}
        color={l.color}
        meta={l.id.toUpperCase()}
        nameLabel="Label name"
        editing={editing === l.id}
        lifted={lifted === l.id && preview !== null}
        error={rowError?.id === l.id ? rowError.message : null}
        ongrab={(e) => grab(e, l.id)}
        onswatch={(anchor) => (swatchFor = { id: l.id, anchor })}
        onedit={() => startEdit(l.id)}
        onrename={(name) => rename(l.id, name)}
        oncancel={cancelEdit}
        onremove={() => askRemove(l)} />
    {/each}
  </div>
  <ColorListAddRow what="label" toValue={statusValue} onadd={addLabel} />
</Section>

{#if swatchFor}
  <SwatchPicker anchor={swatchFor.anchor} color={swatchColor} label="Label color" onpick={pickColor} onclose={() => (swatchFor = null)} />
{/if}

{#if deleting}
  <ConfirmDialog
    title="Delete “{deleting.label.name}”?"
    body="It comes off {plural(deleting.uses, 'task')} that carry it."
    confirm="Delete"
    danger
    onconfirm={confirmRemove}
    onclose={() => (deleting = null)} />
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
