// Where a dragged board card would land, measured from the columns ([data-col]) and
// cards ([data-card]) on screen.
import type { BoardSpot } from '#lib/types/task-board.type.ts';

// The gap between cards in TaskBoardColumn's `.cards`.
const CARD_GAP_PX = 6;

/** How far `x` is to the left or right of `el` (0 when over it). */
function distanceOutside(x: number, el: HTMLElement): number {
  const r = el.getBoundingClientRect();
  if (x < r.left) return r.left - x;
  if (x > r.right) return x - r.right;
  return 0;
}

function nearestColumn(x: number): HTMLElement | null {
  const cols = [...document.querySelectorAll<HTMLElement>('[data-col]')];
  if (!cols.length) return null;
  let best = cols[0];
  let bestDistance = Infinity;
  for (const el of cols) {
    const distance = distanceOutside(x, el);
    if (distance < bestDistance) {
      best = el;
      bestDistance = distance;
    }
  }
  return best;
}

/** How many cards of `col` sit above `y`, leaving out the placeholder of card `id`. */
function indexAt({ col, id, y }: { col: HTMLElement; id: string; y: number }): number {
  // Measure as if the placeholder weren't there, so the target doesn't flip back and forth.
  const placeholder = col.querySelector<HTMLElement>(`[data-card="${id}"]`);
  const placeholderRect = placeholder?.getBoundingClientRect();
  const shift = placeholderRect ? placeholderRect.height + CARD_GAP_PX : 0;
  const cards = [...col.querySelectorAll<HTMLElement>('[data-card]')].filter((c) => c !== placeholder);
  const middles = cards.map((c) => {
    const r = c.getBoundingClientRect();
    const middle = r.top + r.height / 2;
    const belowPlaceholder = placeholderRect !== undefined && r.top > placeholderRect.top;
    return belowPlaceholder ? middle - shift : middle;
  });
  return middles.filter((m) => m < y).length;
}

/** The column and index under the pointer for card `id`, or null when the board has no columns. */
export function spotAt(e: PointerEvent, id: string): BoardSpot | null {
  const col = nearestColumn(e.clientX);
  if (!col) return null;
  return { id, col: col.dataset.col!, idx: indexAt({ col, id, y: e.clientY }) };
}

export function sameSpot(a: BoardSpot | null, b: BoardSpot): boolean {
  if (!a) return false;
  return a.col === b.col && a.idx === b.idx;
}
