// Pointer dragging for explorer rows: starts after a small travel so clicks stay clicks, reports
// moves once per animation frame, and ends on release or cancel. Built on `trackPointer`, so every
// listener comes off however it ends.
import type { RowDragOptions } from '#lib/types/drag.type.ts';
import { hasTravelled, pointOf, swallowReleaseClick, trackPointer } from './drag-session.ts';

/** Returns `stop`, which ends tracking without calling `onend` (the explorer went away). */
export function trackDrag({ start, onstart, onmove, onend }: RowDragOptions): () => void {
  let dragging = false;
  let frame = 0;
  let last: PointerEvent | null = null;

  const from = pointOf(start);

  const step = () => {
    frame = 0;
    if (last && dragging) onmove(last);
  };

  const move = (e: PointerEvent) => {
    if (!dragging) {
      if (!hasTravelled(from, e)) return;
      dragging = true;
      onstart(e);
    }
    last = e;
    if (!frame) frame = requestAnimationFrame(step);
  };

  const end = (e: PointerEvent) => {
    // The click a release fires after a drag must not also open the row.
    if (dragging) swallowReleaseClick(e);
    onend(e.type === 'pointerup');
  };

  const cleanup = () => {
    if (frame) cancelAnimationFrame(frame);
  };

  return trackPointer({ move, end, cleanup });
}

/** The `data-drop` folder under a point, or null when there is none. */
export function dropTargetAt(e: PointerEvent): string | null {
  const hit = document.elementFromPoint(e.clientX, e.clientY)?.closest<HTMLElement>('[data-drop]');
  return hit?.dataset.drop ?? null;
}
