// The keyboard on the task board: where a key moves the focus from a card, given how many cards
// each column holds.
import type { CardSpot } from '#lib/types/task-board.type.ts';

/** Which way ← and → go across the columns. */
const STEP: Record<string, number> = { ArrowLeft: -1, ArrowRight: 1 };

/** The nearest column from `col` in `step` direction that has cards, or -1. */
function columnWithCards(counts: number[], col: number, step: number): number {
  for (let c = col + step; c >= 0 && c < counts.length; c += step) {
    if (counts[c] > 0) return c;
  }
  return -1;
}

/** ↑↓ within a column, ←→ to the same row (or the last) of the next column with cards. */
export function cardTarget({ key, at, counts }: { key: string; at: CardSpot; counts: number[] }): CardSpot | null {
  const inColumn = counts[at.col] ?? 0;
  if (key === 'ArrowDown') return { col: at.col, idx: Math.min(at.idx + 1, inColumn - 1) };
  if (key === 'ArrowUp') return { col: at.col, idx: Math.max(at.idx - 1, 0) };
  const step = STEP[key];
  if (step === undefined) return null;
  const col = columnWithCards(counts, at.col, step);
  if (col < 0) return null;
  return { col, idx: Math.min(at.idx, counts[col] - 1) };
}

/** ⇧← / ⇧→ moves a card to the neighbouring status column; `null` at either end. */
export function statusTarget({ key, col, columns }: { key: string; col: number; columns: number }): number | null {
  const step = STEP[key];
  if (step === undefined) return null;
  const next = col + step;
  const inside = next >= 0 && next < columns;
  return inside ? next : null;
}
