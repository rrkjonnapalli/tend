// Pointer drag to reorder a vertical list of equal-height rows. While dragging, `preview`
// receives the keys in their new order; on release `drop` receives the final order (only when
// it changed). Rows can only move within [min, max] (e.g. open tasks stay above done ones).
import type { ReorderOptions } from '#lib/types/reorder.type.ts';
import { clearDragging, markDragging, swallowReleaseClick, trackPointer } from './drag-session.ts';

const START_PX = 4;
const FALLBACK_ROW_PX = 36;

type Rows = { top: number; height: number };

/** The list's first row top and the row height, measured from the grabbed row. */
function measureRows(e: PointerEvent): Rows | null {
  // The handle may sit inside the row; the row is the nearest element with data-key.
  const row = (e.currentTarget as HTMLElement).closest<HTMLElement>('[data-key]');
  const first = row?.parentElement?.querySelector('[data-key]');
  if (!row || !first) return null;
  const top = first.getBoundingClientRect().top;
  const height = row.getBoundingClientRect().height || FALLBACK_ROW_PX;
  return { top, height };
}

const sameOrder = (a: string[], b: string[]) => a.join('|') === b.join('|');

/**
 * Starts a reorder from a pointerdown. Returns a function that stops it without dropping
 * (call it on unmount).
 */
export function startReorder(e: PointerEvent, o: ReorderOptions): () => void {
  const none = () => {};
  if (e.button !== 0) return none;
  const rows = measureRows(e);
  if (!rows) return none;
  const { top, height } = rows;
  const min = o.min ?? 0;
  const max = o.max ?? o.keys.length - 1;
  const startY = e.clientY;
  let order: string[] | null = null;

  function slotAt(y: number): number {
    const slot = Math.floor((y - top) / height);
    return Math.max(min, Math.min(max, slot));
  }

  function orderWithKeyAt(slot: number): string[] {
    const next = o.keys.filter((k) => k !== o.key);
    next.splice(slot, 0, o.key);
    return next;
  }

  function move(ev: PointerEvent) {
    if (!order) {
      if (Math.abs(ev.clientY - startY) < START_PX) return;
      order = o.keys.slice();
      markDragging();
    }
    const next = orderWithKeyAt(slotAt(ev.clientY));
    if (sameOrder(next, order)) return;
    order = next;
    o.preview(next);
  }

  function end(ev: PointerEvent) {
    if (!order) return;
    swallowReleaseClick(ev);
    o.preview(null);
    if (!sameOrder(order, o.keys)) o.drop(order, order.indexOf(o.key));
  }

  return trackPointer({ move, end, cleanup: clearDragging });
}
