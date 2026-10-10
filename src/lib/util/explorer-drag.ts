// Pointer dragging for explorer rows: starts after a small travel so clicks stay clicks, reports
// moves once per animation frame, and ends on release or cancel. Built on `trackPointer`, so every
// listener comes off however it ends.
import { swallowReleaseClick, trackPointer } from './drag-session.ts';

/** How far (|dx| + |dy|, in px) the pointer travels before a press becomes a drag. */
const DRAG_THRESHOLD_PX = 5;

type DragOptions = {
  /** The pointerdown that may start a drag. */
  start: PointerEvent;
  /** The pointer has travelled far enough: the drag begins. */
  onstart: (e: PointerEvent) => void;
  /** The latest pointer position while dragging, at most once per frame. */
  onmove: (e: PointerEvent) => void;
  /** The pointer was released (`dropped`) or the drag was cancelled; also when no drag began. */
  onend: (dropped: boolean) => void;
};

/** Returns `stop`, which ends tracking without calling `onend` (the explorer went away). */
export function trackDrag({ start, onstart, onmove, onend }: DragOptions): () => void {
  let dragging = false;
  let frame = 0;
  let last: PointerEvent | null = null;

  const travelled = (e: PointerEvent) => Math.abs(e.clientX - start.clientX) + Math.abs(e.clientY - start.clientY);

  const step = () => {
    frame = 0;
    if (last && dragging) onmove(last);
  };

  const move = (e: PointerEvent) => {
    if (!dragging) {
      if (travelled(e) < DRAG_THRESHOLD_PX) return;
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
