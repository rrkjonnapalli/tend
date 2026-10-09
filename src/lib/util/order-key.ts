// Fractional order keys (base 62). Sorting keys as plain strings gives the order, and a key
// can always be made between two neighbours, so a move rewrites only the moved item.
// Same scheme as src-tauri/src/order/: keep the two in step.

const DIGITS = '0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz';
const BASE = DIGITS.length;

/** The digit value at `i`, or `past` once the key has run out. */
function digitAt(key: string, i: number, past: number): number {
  if (i >= key.length) return past;
  return DIGITS.indexOf(key[i]);
}

/** A key that sorts after `a` and before `b`. `null` means "no neighbour on that side". */
export function keyBetween(a: string | null, b: string | null): string {
  const lo = a ?? '';
  const hi = b ?? '';
  if (b !== null && lo >= b) throw new Error(`keyBetween: ${a} is not before ${b}`);
  let out = '';
  // Once the key is known to sort below `b`, any upper digit is free.
  let open = b === null;
  for (let i = 0; ; i++) {
    const da = digitAt(lo, i, 0);
    const db = open ? BASE : digitAt(hi, i, BASE);
    if (da === db) {
      out += DIGITS[da];
      continue;
    }
    const hasRoom = db - da > 1;
    if (hasRoom) {
      const mid = Math.floor((da + db) / 2);
      return out + DIGITS[mid];
    }
    out += DIGITS[da];
    open = true;
  }
}

/** A key after `prev` and before the first of `next` (in list order) that sorts above it. Keys can
 *  tie (two devices inserting at the same spot, then syncing); skipping the ties keeps a drop
 *  between equal keys from failing, and lands it right after them. */
export function keyBefore(prev: string | null, next: string[]): string {
  const above = next.find((k) => prev === null || k > prev) ?? null;
  return keyBetween(prev, above);
}

/** Key for moving an item to `index` in `keys` (sorted, the item itself left out). */
export function keyAt(keys: string[], index: number): string {
  return keyBefore(keys[index - 1] ?? null, keys.slice(index));
}
