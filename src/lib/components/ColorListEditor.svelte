<script lang="ts">
  // A colour list in Settings → Tasks (statuses, labels): drag to order, colour swatch, rename
  // inline, add with an UPPER_SNAKE value, delete (with a confirm when `ops.removeWarning` says so).
  import { onDestroy } from 'svelte';
  import Section from './Section.svelte';
  import ColorListRow from './ColorListRow.svelte';
  import ColorListAddRow from './ColorListAddRow.svelte';
  import SwatchPicker from './SwatchPicker.svelte';
  import ConfirmDialog from './ConfirmDialog.svelte';
  import type { ColorItem, ColorListOps } from '#lib/types/color-list.type.ts';
  import { messageOf } from '#lib/errors/app-error.ts';
  import { startReorder } from '#lib/util/reorder.ts';
  import { freeSwatch } from '#lib/util/swatches.ts';
  import { statusValue } from '#lib/util/statuses.ts';

  let {
    title,
    intro,
    what,
    items,
    ops,
    pinEnds = false,
  }: {
    title: string;
    intro: string;
    /** "status" or "label": used in placeholders and accessible names. */
    what: string;
    items: ColorItem[];
    ops: ColorListOps;
    /** The first and last rows stay where they are when others are dragged. */
    pinEnds?: boolean;
  } = $props();

  let preview = $state<string[] | null>(null);
  let lifted = $state<string | null>(null);
  let editing = $state<string | null>(null);
  let rowError = $state<{ key: string; message: string } | null>(null);
  let swatchFor = $state<{ key: string; anchor: HTMLElement } | null>(null);
  let deleting = $state<{ item: ColorItem; warning: string } | null>(null);

  /** The rows in the order being dragged, or as saved. */
  function inOrder(order: string[]): ColorItem[] {
    const found = order.map((key) => items.find((item) => item.key === key));
    return found.filter((item) => item !== undefined);
  }

  const list = $derived(preview ? inOrder(preview) : items);
  const noun = $derived(what.charAt(0).toUpperCase() + what.slice(1));
  const swatchColor = $derived(items.find((item) => item.key === swatchFor?.key)?.color);

  /** Runs a change and reloads the list; returns why it failed, or null. */
  async function apply(action: () => Promise<unknown>): Promise<string | null> {
    try {
      await action();
      await ops.reload();
      rowError = null;
      return null;
    } catch (e) {
      return messageOf(e);
    }
  }

  /** A change to one row; a failure shows under that row. */
  async function runRow(key: string, action: () => Promise<unknown>): Promise<boolean> {
    const failure = await apply(action);
    if (failure) rowError = { key, message: failure };
    return failure === null;
  }

  function add({ name, value }: { name: string; value: string }) {
    const color = freeSwatch(items.map((item) => item.color));
    return apply(() => ops.add({ name, value, color }));
  }

  function startEdit(key: string) {
    editing = key;
    rowError = null;
  }

  async function rename(key: string, name: string) {
    const ok = await runRow(key, () => ops.rename({ key, name }));
    if (ok) editing = null;
  }

  function cancelEdit() {
    editing = null;
    rowError = null;
  }

  async function pickColor(color: string) {
    if (!swatchFor) return;
    const { key } = swatchFor;
    swatchFor = null;
    await runRow(key, () => ops.recolor({ key, color }));
  }

  /** A row with nothing depending on it goes at once; one in use asks first. */
  async function askRemove(item: ColorItem) {
    const warning = ops.removeWarning ? await ops.removeWarning(item.key) : null;
    if (warning === null) {
      await runRow(item.key, () => ops.remove(item.key));
      return;
    }
    deleting = { item, warning };
  }

  async function confirmRemove() {
    if (!deleting) return;
    const { key } = deleting.item;
    deleting = null;
    await runRow(key, () => ops.remove(key));
  }

  function showPreview(order: string[] | null) {
    preview = order;
    if (!order) lifted = null;
  }

  let stopReorder: (() => void) | null = null;
  onDestroy(() => stopReorder?.());

  function grab(e: PointerEvent, key: string) {
    e.preventDefault();
    const keys = items.map((item) => item.key);
    lifted = key;
    editing = null;
    const bounds = pinEnds ? { min: 1, max: keys.length - 2 } : {};
    stopReorder = startReorder(e, {
      keys,
      key,
      ...bounds,
      preview: showPreview,
      drop: (order) => runRow(key, () => ops.reorder(order)),
    });
  }
</script>

<Section {title} extra={String(items.length)} indent={false}>
  <span class="caption muted intro">{intro}</span>
  <div class="rows">
    {#each list as item (item.key)}
      <ColorListRow
        id={item.key}
        name={item.name}
        color={item.color}
        ring={item.ring}
        pinned={item.pinned}
        note={item.note}
        meta={item.meta}
        removable={item.removable}
        nameLabel="{noun} name"
        editing={editing === item.key}
        lifted={lifted === item.key && preview !== null}
        error={rowError?.key === item.key ? rowError.message : null}
        ongrab={(e) => grab(e, item.key)}
        onswatch={(anchor) => (swatchFor = { key: item.key, anchor })}
        onedit={() => startEdit(item.key)}
        onrename={(name) => rename(item.key, name)}
        oncancel={cancelEdit}
        onremove={() => askRemove(item)} />
    {/each}
  </div>
  <ColorListAddRow {what} toValue={statusValue} onadd={add} />
</Section>

{#if swatchFor}
  <SwatchPicker anchor={swatchFor.anchor} color={swatchColor} label="{noun} color" onpick={pickColor} onclose={() => (swatchFor = null)} />
{/if}

{#if deleting}
  <ConfirmDialog
    title="Delete “{deleting.item.name}”?"
    body={deleting.warning}
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
