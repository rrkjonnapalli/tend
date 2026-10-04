// The floating copy of a dragged card: it follows the pointer, slightly tilted, above everything.
import type { DragGhost, Point } from '#lib/types/drag.type.ts';

const GHOST_TILT = 'rotate(1.5deg)';
const GHOST_Z_INDEX = '1000';

/** Clones `card` onto the page, keeping the pointer at the spot it grabbed (`grab`). */
export function liftGhost(card: HTMLElement, grab: Point): DragGhost {
  const r = card.getBoundingClientRect();
  const offset = { x: grab.x - r.left, y: grab.y - r.top };
  const el = card.cloneNode(true) as HTMLElement;
  el.classList.add('ghost');
  el.removeAttribute('data-card');
  Object.assign(el.style, {
    position: 'fixed',
    left: '0',
    top: '0',
    margin: '0',
    zIndex: GHOST_Z_INDEX,
    pointerEvents: 'none',
    width: `${r.width}px`,
  });
  document.body.appendChild(el);

  function move(e: PointerEvent) {
    const x = e.clientX - offset.x;
    const y = e.clientY - offset.y;
    el.style.transform = `translate(${x}px, ${y}px) ${GHOST_TILT}`;
  }

  return { move, remove: () => el.remove() };
}
