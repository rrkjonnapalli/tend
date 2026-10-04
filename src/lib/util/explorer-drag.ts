// Pointer dragging for explorer rows: starts after a small travel so clicks stay clicks,
// reports moves once per animation frame, and ends on release.

/** How far (|dx| + |dy|, in px) the pointer travels before a press becomes a drag. */
const DRAG_THRESHOLD_PX = 5;

type DragOptions = {
  /** The pointerdown that may start a drag. */
  start: PointerEvent;
  /** The pointer has travelled far enough: the drag begins. */
  onstart: (e: PointerEvent) => void;
  /** The latest pointer position while dragging, at most once per frame. */
  onmove: (e: PointerEvent) => void;
  /** The pointer was released, whether or not a drag began. */
  onend: () => void;
};

export function trackDrag({ start, onstart, onmove, onend }: DragOptions) {
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

  const up = () => {
    window.removeEventListener('pointermove', move);
    window.removeEventListener('pointerup', up);
    if (frame) cancelAnimationFrame(frame);
    onend();
  };

  window.addEventListener('pointermove', move);
  window.addEventListener('pointerup', up);
}

/** The `data-drop` folder under a point, or null when there is none. */
export function dropTargetAt(e: PointerEvent): string | null {
  const hit = document.elementFromPoint(e.clientX, e.clientY)?.closest<HTMLElement>('[data-drop]');
  return hit?.dataset.drop ?? null;
}
