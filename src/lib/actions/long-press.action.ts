// Long-press on a touch screen (tend-mobile-spec.md §8): `onpress` runs after the finger rests
// HOLD_MS without moving, and the tap that ends it doesn't also open the row.
const HOLD_MS = 500;
const SLOP_PX = 8;

export function longPress(node: HTMLElement, onpress: () => void) {
  let timer: ReturnType<typeof setTimeout> | undefined;
  let start: { x: number; y: number } | null = null;
  let fired = false;

  const cancel = () => {
    clearTimeout(timer);
    start = null;
  };

  function down(e: PointerEvent) {
    fired = false;
    start = { x: e.clientX, y: e.clientY };
    timer = setTimeout(() => {
      fired = true;
      onpress();
    }, HOLD_MS);
  }

  function move(e: PointerEvent) {
    if (start && Math.hypot(e.clientX - start.x, e.clientY - start.y) > SLOP_PX) cancel();
  }

  /** The click after a long press only ended it. */
  function click(e: MouseEvent) {
    if (!fired) return;
    e.preventDefault();
    e.stopPropagation();
    fired = false;
  }

  /** The system's own long-press menu stays away. */
  const menu = (e: Event) => e.preventDefault();

  node.addEventListener('pointerdown', down);
  node.addEventListener('pointermove', move);
  node.addEventListener('pointerup', cancel);
  node.addEventListener('pointercancel', cancel);
  node.addEventListener('click', click, true);
  node.addEventListener('contextmenu', menu);
  return {
    update(next: () => void) {
      onpress = next;
    },
    destroy() {
      cancel();
      node.removeEventListener('pointerdown', down);
      node.removeEventListener('pointermove', move);
      node.removeEventListener('pointerup', cancel);
      node.removeEventListener('pointercancel', cancel);
      node.removeEventListener('click', click, true);
      node.removeEventListener('contextmenu', menu);
    },
  };
}
