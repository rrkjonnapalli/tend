// Order keys: sorting them as plain strings gives the order, and a key can always be made between
// two neighbours, so a move rewrites only the moved item. `fractional-indexing` makes them; its
// keys stay short however many items are added at the top or the bottom.
import { generateKeyBetween } from 'fractional-indexing';

/** A key that sorts after `a` and before `b`. `null` means "no neighbour on that side". */
export function keyBetween(a: string | null, b: string | null): string {
  // The library doesn't check the order and would return a key outside the two.
  if (a !== null && b !== null && a >= b) throw new Error(`keyBetween: ${a} is not before ${b}`);
  return generateKeyBetween(a, b);
}

/** A key after `prev` and before the first of `next` (in list order) that sorts above it. Keys can
 *  tie (two devices inserting at the same spot, then syncing); skipping the ties keeps a drop
 *  between equal keys from failing, and lands it right after them. */
export function keyAfter(prev: string | null, next: string[]): string {
  const above = next.find((k) => prev === null || k > prev) ?? null;
  return keyBetween(prev, above);
}

/** Key for moving an item to `index` in `keys` (sorted, the item itself left out). */
export function keyAt(keys: string[], index: number): string {
  return keyAfter(keys[index - 1] ?? null, keys.slice(index));
}
