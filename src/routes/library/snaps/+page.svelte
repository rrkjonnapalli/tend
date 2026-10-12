<script lang="ts">
  // Snaps: the pinboard. It shows one level (the board, or the open stack, `?s=<id>`); while the
  // Library search has text it shows every match across all stacks instead. + or a snap opens the
  // editor. Drag a snap or stack onto a stack to put it in, a snap onto a snap to make a stack of
  // the two, anything onto a step of the path to move it up.
  import { untrack } from 'svelte';
  import { page } from '$app/state';
  import { goto } from '$app/navigation';
  import LibraryEmpty from '#lib/components/LibraryEmpty.svelte';
  import SnapEditor from '#lib/components/SnapEditor.svelte';
  import SnapItem from '#lib/components/SnapItem.svelte';
  import StackItem from '#lib/components/StackItem.svelte';
  import StackPath from '#lib/components/StackPath.svelte';
  import ConfirmDialog from '#lib/components/ConfirmDialog.svelte';
  import { snapsAPI } from '#lib/api/snaps.ts';
  import { boardColumn, boardTilts } from '#lib/config/library.config.ts';
  import { PAPER } from '#lib/enums/snap.enum.ts';
  import { library } from '#lib/stores/library.svelte.ts';
  import { settings } from '#lib/stores/settings.svelte.ts';
  import { snaps } from '#lib/stores/snaps.svelte.ts';
  import { toast } from '#lib/stores/toast.svelte.ts';
  import type { DragGhost } from '#lib/types/drag.type.ts';
  import type { Snap, SnapDraft, Stack } from '#lib/types/snap.type.ts';
  import { clearDragging, markDragging, pointOf } from '#lib/util/drag-session.ts';
  import { liftGhost } from '#lib/util/drag-ghost.ts';
  import { dropTargetAt, trackDrag } from '#lib/util/explorer-drag.ts';
  import { plural } from '#lib/util/plural.ts';
  import { routes } from '#lib/util/routes.ts';
  import { searchNeedle } from '#lib/util/search.ts';
  import {
    boardItems,
    boardLevel,
    columnCount,
    estimateHeight,
    packColumns,
    dropKey,
    isWithin,
    itemKey,
    NEW_STACK,
    parseDrop,
    pathOf,
    stackName,
    stackPeek,
    stackSummary,
    whereLabel,
    type DropTarget,
  } from '#lib/util/snaps.ts';

  /** How close to the board's top or bottom edge (px) a drag scrolls it, and by how much. */
  const EDGE_PX = 48;
  const SCROLL_STEP_PX = 14;

  snaps.load();

  const needle = $derived(searchNeedle(library.query));
  const requested = $derived(page.url.searchParams.get('s'));
  /** The open stack; none is the board. A stack that is gone (deleted elsewhere) shows the board. */
  const open = $derived(snaps.stacks.some((s) => s.id === requested) ? requested : null);
  const level = $derived(boardLevel(open, snaps.snaps, snaps.stacks, needle));
  const items = $derived(boardItems(level.snaps, level.stacks));
  const path = $derived(pathOf(open, snaps.stacks));

  /** The board's width inside its side gaps, and the columns that fit in it. */
  let surfaceWidth = $state(0);
  const columns = $derived.by(() => {
    const inner = surfaceWidth - 2 * boardColumn.gap;
    const count = columnCount(inner, boardColumn.minWidth, boardColumn.gap);
    const width = (inner - (count - 1) * boardColumn.gap) / count;
    return packColumns(items, count, (item) => estimateHeight(item, width));
  });

  const empty = $derived.by(() => {
    if (needle) return { title: 'Nothing matches', hint: 'Try other words.' };
    if (open) return { title: 'Nothing in this stack yet', hint: 'Press + to add a snap here, or drag snaps onto this stack.' };
    return { title: 'Nothing on the board yet', hint: 'Press + to pin your first snap.' };
  });

  let editing = $state<{ snap: Snap | null } | null>(null);
  let renaming = $state<string | null>(null);
  let deleting = $state<Snap | null>(null);
  let removing = $state<Stack | null>(null);
  let drag = $state<{ kind: 'snap' | 'stack'; id: string } | null>(null);
  let over = $state<string | null>(null);
  let board = $state<HTMLElement>();

  const where = (id: string | null) => whereLabel(id, snaps.stacks);

  // The bar's + opens a blank paper; presses from before this page opened don't.
  let seenAdds = library.adds;
  $effect(() => {
    const adds = library.adds;
    if (adds === seenAdds) return;
    seenAdds = adds;
    untrack(() => (editing = { snap: null }));
  });

  function openStack(id: string | null) {
    goto(routes.libraryStack(id));
    if (board) board.scrollTop = 0;
  }

  // ---- the editor

  /** The stack a saved snap goes in: the one picked, or a new one where the board is. */
  async function targetOf(draft: SnapDraft): Promise<string | null> {
    if (draft.stackId !== NEW_STACK) return draft.stackId;
    const made = await snapsAPI.createStack({ parent: open, name: '' });
    return made.id;
  }

  async function saveDraft(draft: SnapDraft) {
    const before = editing?.snap ?? null;
    const stackId = await targetOf(draft);
    const color = draft.paper === PAPER.sticky ? draft.color : null;
    const input = { text: draft.text, stackId, paper: draft.paper, color, pinned: draft.pinned };
    const saved = before ? await snapsAPI.update({ id: before.id, patch: input }) : await snapsAPI.create(input);
    const tagsChanged = draft.tags.join(' ') !== saved.tags.join(' ');
    if (tagsChanged) await snapsAPI.setTags({ id: saved.id, tags: draft.tags });
    editing = null;
    await snaps.load();
    if (draft.stackId === NEW_STACK) {
      // Name the new stack right there.
      renaming = stackId;
      return;
    }
    const moved = before !== null && before.stackId !== stackId;
    if (!before) toast.show(`Added to ${where(stackId)}`);
    else if (moved) toast.show(`Saved and moved to ${where(stackId)}`);
    else toast.show('Saved');
  }

  // ---- snaps and stacks

  async function togglePin(snap: Snap) {
    await snapsAPI.update({ id: snap.id, patch: { pinned: !snap.pinned } });
    await snaps.load();
    toast.show(snap.pinned ? 'Unpinned' : 'Pinned to the top');
  }

  function askDelete(snap: Snap) {
    if (settings.asksDelete('snap')) deleting = snap;
    else deleteSnap(snap);
  }

  function deleteEditing() {
    const snap = editing?.snap;
    if (snap) askDelete(snap);
  }

  async function deleteSnap(snap: Snap) {
    editing = null;
    await snapsAPI.remove(snap.id);
    await snaps.load();
    toast.show('Snap deleted');
  }

  async function rename(stack: Stack, name: string) {
    renaming = null;
    if (name.trim() === stack.name) return;
    await snapsAPI.renameStack({ id: stack.id, name });
    await snaps.load();
  }

  async function removeStack(stack: Stack) {
    const inside = snaps.snaps.filter((n) => n.stackId === stack.id).length;
    await snapsAPI.removeStack(stack.id);
    await snaps.load();
    const moved = inside > 0 ? `, ${plural(inside, 'snap')} moved to ${where(stack.parentId)}` : '';
    toast.show(`Stack removed${moved}`);
  }

  // ---- moving

  async function moveTo(src: { kind: 'snap' | 'stack'; id: string }, stackId: string | null) {
    if (src.kind === 'stack' && stackId && isWithin(stackId, src.id, snaps.stacks)) {
      toast.show('A stack can’t go inside itself');
      return;
    }
    if (src.kind === 'snap') await snapsAPI.update({ id: src.id, patch: { stackId } });
    else await snapsAPI.moveStack({ id: src.id, parent: stackId });
    await snaps.load();
    toast.show(`Moved to ${where(stackId)}`);
  }

  /** A snap dropped on a snap: both go into a new stack, named right there. */
  async function pair(a: string, b: string) {
    const stack = await snapsAPI.createStack({ parent: open, name: '' });
    for (const id of [a, b]) await snapsAPI.update({ id, patch: { stackId: stack.id } });
    await snaps.load();
    renaming = stack.id;
  }

  function drop(src: { kind: 'snap' | 'stack'; id: string }, target: DropTarget) {
    if (target.kind === 'snap' && target.id) pair(src.id, target.id);
    else moveTo(src, target.id);
  }

  // ---- dragging

  /** The drop target under the pointer, if what is held may go there. */
  function allowed(key: string | null): string | null {
    if (!key || !drag) return null;
    if (key === dropKey(drag.kind, drag.id)) return null;
    const target = parseDrop(key);
    const pairsOnlySnaps = target?.kind === 'snap' && drag.kind !== 'snap';
    return pairsOnlySnaps ? null : key;
  }

  /** Held near the board's top or bottom edge, the board scrolls. */
  function scrollNear(e: PointerEvent) {
    if (!board) return;
    const r = board.getBoundingClientRect();
    if (e.clientY > r.bottom - EDGE_PX) board.scrollTop += SCROLL_STEP_PX;
    else if (e.clientY < r.top + EDGE_PX) board.scrollTop -= SCROLL_STEP_PX;
  }

  /** A press on a snap or stack: one that moves 5 px or more lifts a copy that follows the
   *  pointer; less stays a click. Off while searching or writing. */
  function press(kind: 'snap' | 'stack', id: string) {
    return (e: PointerEvent) => {
      const blocked = e.button !== 0 || needle !== '' || editing !== null;
      const card = (e.currentTarget as HTMLElement).closest<HTMLElement>('[data-drop]');
      if (blocked || !card) return;
      let ghost: DragGhost | null = null;
      trackDrag({
        start: e,
        onstart: (ev) => {
          ghost = liftGhost(card, pointOf(e));
          ghost.move(ev);
          markDragging();
          drag = { kind, id };
        },
        onmove: (ev) => {
          ghost?.move(ev);
          over = allowed(dropTargetAt(ev));
          scrollNear(ev);
        },
        onend: (dropped) => {
          ghost?.remove();
          clearDragging();
          const src = drag;
          const target = over ? parseDrop(over) : null;
          drag = null;
          over = null;
          if (dropped && src && target) drop(src, target);
        },
      });
    };
  }

  const isDragging = (kind: 'snap' | 'stack', id: string) => drag?.kind === kind && drag.id === id;
  const tiltAt = (i: number) => boardTilts[i % boardTilts.length];
</script>

<div class="snaps">
  <div class="board" bind:this={board}>
    {#if path.length && !needle}
      <StackPath {path} {over} onopen={openStack} />
    {/if}
    {#if snaps.loaded && items.length === 0}
      <div class="empty"><LibraryEmpty title={empty.title} hint={empty.hint} board /></div>
    {/if}
    <div class="surface" bind:clientWidth={surfaceWidth} style:gap="{boardColumn.gap}px" style:padding-inline="{boardColumn.gap}px">
      {#each columns as column, c (c)}
        <div class="column">
          {#each column as { item, at: i } (itemKey(item))}
            {#if item.kind === 'snap'}
              {@const snap = item.snap}
              <SnapItem
                {snap}
                tilt={tiltAt(i)}
                where={needle ? where(snap.stackId) : null}
                dragging={isDragging('snap', snap.id)}
                pairing={over === dropKey('snap', snap.id)}
                onopen={() => (editing = { snap })}
                onpress={press('snap', snap.id)}
                onpin={() => togglePin(snap)}
                ondelete={() => askDelete(snap)} />
            {:else}
              {@const stack = item.stack}
              <StackItem
                {stack}
                tilt={tiltAt(i)}
                where={needle ? where(stack.parentId) : null}
                summary={stackSummary(stack, snaps.snaps, snaps.stacks)}
                peek={stackPeek(stack, snaps.snaps, snaps.stacks)}
                dragging={isDragging('stack', stack.id)}
                over={over === dropKey('stack', stack.id)}
                renaming={renaming === stack.id}
                onopen={() => openStack(stack.id)}
                onpress={press('stack', stack.id)}
                onstartrename={() => (renaming = stack.id)}
                onrename={(name) => rename(stack, name)}
                onremove={() => (removing = stack)} />
            {/if}
          {/each}
        </div>
      {/each}
    </div>
  </div>

  {#if editing}
    {#key editing}
      <SnapEditor snap={editing.snap} {open} onsave={saveDraft} ondelete={deleteEditing} onclose={() => (editing = null)} />
    {/key}
  {/if}
</div>

{#if deleting}
  {@const snap = deleting}
  <ConfirmDialog
    title="Delete this snap?"
    body="It goes from the board."
    confirm="Delete"
    danger
    onconfirm={() => deleteSnap(snap)}
    onclose={() => (deleting = null)} />
{/if}
{#if removing}
  {@const stack = removing}
  <ConfirmDialog
    title="Remove “{stackName(stack)}”?"
    body="Its snaps and stacks stay: they move up to {where(stack.parentId)}."
    confirm="Remove stack"
    onconfirm={() => removeStack(stack)}
    onclose={() => (removing = null)} />
{/if}

<style>
  .snaps {
    position: relative;
    flex: 1 1 auto;
    display: flex;
    min-height: 0;
  }

  .board {
    position: relative;
    flex: 1 1 auto;
    min-height: 0;
    overflow-x: hidden;
    overflow-y: auto;
    background-color: var(--board-bg);
    background-image: radial-gradient(var(--board-dot) 1px, transparent 1.3px);
    background-size: 22px 22px;
    background-attachment: local;
  }

  /* The board's own scrollbar: its background scrolls with the content (attachment local), so
     the app's transparent track would show black; this one stays warm. */
  .board::-webkit-scrollbar-track {
    background: var(--board-bg);
  }

  .board::-webkit-scrollbar-thumb {
    background: var(--board-dot);
    border-color: var(--board-bg);
  }

  .board::-webkit-scrollbar-thumb:hover {
    background: var(--board-hint);
  }

  /* Real columns (not CSS multi-column, where WebKit splits and ghosts rotated items). */
  .surface {
    display: flex;
    align-items: flex-start;
    padding-top: 40px;
    padding-bottom: 16px;
  }

  .column {
    flex: 1 1 0;
    min-width: 0;
  }

  .empty {
    position: absolute;
    inset: 0;
    display: flex;
    pointer-events: none;
  }
</style>
