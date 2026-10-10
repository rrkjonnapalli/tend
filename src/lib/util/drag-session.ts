// Window-level pointer tracking shared by list reordering and the task board. Every listener
// comes off however the drag ends: release, cancel, or the owner stopping it (unmount).
import type { Point, PointerDrag } from '#lib/types/drag.type.ts';

const DRAGGING_CLASS = 'reordering';

/** Follows the pointer until it is released or cancelled. Returns `stop`, which ends tracking early without calling `end`. */
export function trackPointer({ move, end, cleanup }: PointerDrag): () => void {
  let active = true;

  function stop() {
    if (!active) return;
    active = false;
    window.removeEventListener('pointermove', move);
    window.removeEventListener('pointerup', finish);
    window.removeEventListener('pointercancel', finish);
    cleanup();
  }

  function finish(e: PointerEvent) {
    stop();
    end(e);
  }

  window.addEventListener('pointermove', move);
  window.addEventListener('pointerup', finish);
  window.addEventListener('pointercancel', finish);
  return stop;
}

/** The grabbing cursor everywhere while a drag runs (see app.css). */
export function markDragging() {
  document.body.classList.add(DRAGGING_CLASS);
}

export function clearDragging() {
  document.body.classList.remove(DRAGGING_CLASS);
}

/** Stops the click that releasing a drag fires. A cancelled pointer fires no click. */
export function swallowReleaseClick(e: PointerEvent) {
  if (e.type === 'pointercancel') return;
  window.addEventListener('click', (c) => c.stopPropagation(), { capture: true, once: true });
}

export const pointOf = (e: PointerEvent): Point => ({ x: e.clientX, y: e.clientY });

/** How far (|dx| + |dy|, in px) the pointer travels before a press becomes a drag; less stays a click. */
const DRAG_START_PX = 5;

export function hasTravelled(from: Point, e: PointerEvent): boolean {
  const distance = Math.abs(e.clientX - from.x) + Math.abs(e.clientY - from.y);
  return distance >= DRAG_START_PX;
}
