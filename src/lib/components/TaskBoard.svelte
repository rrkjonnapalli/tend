<script lang="ts">
  // The board: one column per status. Cards move with the pointer; a dashed placeholder shows
  // where they land. Dropping sets the status from the column and the order from the position.
  import type { DragGhost, Point } from '#lib/types/drag.type.ts';
  import type { BoardSpot } from '#lib/types/task-board.type.ts';
  import type { Task, TaskPatch } from '#lib/types/task.type.ts';
  import { onDestroy, tick } from 'svelte';
  import { cardTarget, statusTarget } from '#lib/util/board-nav.ts';
  import TaskBoardColumn from './TaskBoardColumn.svelte';
  import { tasks } from '#lib/stores/tasks.svelte.ts';
  import { people } from '#lib/stores/people.svelte.ts';
  import { byOrderKey } from '#lib/util/task-views.ts';
  import { keyAt } from '#lib/util/order-key.ts';
  import { clearDragging, markDragging, pointOf, swallowReleaseClick, trackPointer } from '#lib/util/drag-session.ts';
  import { liftGhost } from '#lib/util/drag-ghost.ts';
  import { sameSpot, spotAt } from '#lib/util/task-board.ts';

  let { items, folder }: { items: Task[]; folder: string } = $props();

  const DRAG_START_PX = 5;

  people.load();

  /** Where the dragged card would land. */
  let drag = $state<BoardSpot | null>(null);
  /** The status whose column has its add field open. */
  let adding = $state<string | null>(null);
  let stopDrag: (() => void) | null = null;

  onDestroy(() => stopDrag?.());

  const columnTasks = (status: string, leftOut: string | undefined) =>
    items.filter((t) => t.status === status && t.id !== leftOut).sort(byOrderKey);

  /** The column's cards, with the dragged card moved to where it would land. */
  function cardsOf(status: string): Task[] {
    const list = columnTasks(status, drag?.id);
    if (!drag || drag.col !== status) return list;
    const dragId = drag.id;
    const moving = items.find((t) => t.id === dragId);
    if (!moving) return list;
    const at = Math.min(drag.idx, list.length);
    list.splice(at, 0, moving);
    return list;
  }

  /** Manhattan distance, enough to tell a drag from a click. */
  const travel = (a: Point, b: Point) => Math.abs(a.x - b.x) + Math.abs(a.y - b.y);

  function grab(e: PointerEvent, task: Task) {
    if (e.button !== 0) return;
    const card = e.currentTarget as HTMLElement;
    const start = pointOf(e);
    let ghost: DragGhost | null = null;
    let frame = 0;
    let latest: PointerEvent | null = null;

    function lift(): DragGhost {
      const lifted = liftGhost(card, start);
      markDragging();
      const idx = cardsOf(task.status).findIndex((t) => t.id === task.id);
      drag = { id: task.id, col: task.status, idx };
      return lifted;
    }

    function follow() {
      frame = 0;
      if (!latest) return;
      const spot = spotAt(latest, task.id);
      if (spot && !sameSpot(drag, spot)) drag = spot;
    }

    function move(ev: PointerEvent) {
      if (!ghost) {
        if (travel(pointOf(ev), start) < DRAG_START_PX) return;
        ghost = lift();
      }
      ev.preventDefault();
      ghost.move(ev);
      latest = ev;
      // Measure the board at most once a frame.
      if (!frame) frame = requestAnimationFrame(follow);
    }

    function cleanup() {
      if (frame) cancelAnimationFrame(frame);
      ghost?.remove();
      clearDragging();
    }

    function end(ev: PointerEvent) {
      if (!ghost) return;
      swallowReleaseClick(ev);
      const spot = drag;
      drag = null;
      if (spot) drop(task, spot);
    }

    stopDrag = trackPointer({ move, end, cleanup });
  }

  // ---- keyboard: ↑↓ ←→ move between cards, ⇧← ⇧→ move a card to the next status

  const columnsOnScreen = (board: HTMLElement) => [...board.querySelectorAll<HTMLElement>('[data-col]')];
  const cardsIn = (col: HTMLElement) => [...col.querySelectorAll<HTMLElement>('[data-card]')];

  function focusCard(id: string) {
    document.querySelector<HTMLElement>(`[data-card="${CSS.escape(id)}"]`)?.focus();
  }

  async function moveToStatus(task: Task, status: string) {
    const end = columnTasks(status, task.id).length;
    await drop(task, { id: task.id, col: status, idx: end });
    await tick();
    focusCard(task.id);
  }

  function onkeydown(e: KeyboardEvent) {
    const card = (e.target as HTMLElement).closest<HTMLElement>('[data-card]');
    const hasOtherModifier = e.metaKey || e.ctrlKey || e.altKey;
    if (!card || hasOtherModifier || card !== e.target) return;
    const cols = columnsOnScreen(e.currentTarget as HTMLElement);
    const col = cols.findIndex((c) => c.contains(card));
    const task = items.find((t) => t.id === card.dataset.card);
    if (col < 0 || !task) return;
    if (e.shiftKey) {
      const to = statusTarget(e.key, col, cols.length);
      if (to === null) return;
      e.preventDefault();
      moveToStatus(task, tasks.statuses[to].value);
      return;
    }
    const counts = cols.map((c) => cardsIn(c).length);
    const at = { col, idx: cardsIn(cols[col]).indexOf(card) };
    const next = cardTarget(e.key, at, counts);
    if (!next) return;
    e.preventDefault();
    cardsIn(cols[next.col])[next.idx]?.focus();
  }

  async function drop(task: Task, spot: BoardSpot) {
    const others = columnTasks(spot.col, task.id);
    const orderKey = keyAt(others.map((t) => t.orderKey), spot.idx);
    const patch: TaskPatch = spot.col === task.status ? { orderKey } : { orderKey, status: spot.col };
    await tasks.update(task.id, patch);
  }
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="board" {onkeydown}>
  {#each tasks.statuses as s (s.value)}
    <TaskBoardColumn
      status={s}
      cards={cardsOf(s.value)}
      {folder}
      liftedId={drag?.id ?? null}
      adding={adding === s.value}
      onstartadd={() => (adding = s.value)}
      onstopadd={() => (adding = null)}
      ongrab={grab} />
  {/each}
</div>

<style>
  /* Up to --board-fit columns share the width; more scroll sideways at that column width. */
  .board {
    --board-gap: 12px;
    --board-fit: 5;
    flex: 1 1 auto;
    min-height: 0;
    display: flex;
    align-items: stretch;
    gap: var(--board-gap);
    overflow-x: auto;
    padding-bottom: 8px;
    user-select: none;
  }
</style>
