// Fractional order keys (base 62). Sorting keys as plain strings gives the order, and a key
// can always be made between two neighbours, so a move rewrites only the moved item.

const DIGITS = '0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz';
const BASE = DIGITS.length;

/** A key that sorts after `a` and before `b`. `null` means "no neighbour on that side". */
export function keyBetween(a: string | null, b: string | null): string {
  const lo = a ?? '';
  if (b !== null && lo >= b) throw new Error(`keyBetween: ${a} is not before ${b}`);
  let out = '';
  let open = b === null;
  for (let i = 0; ; i++) {
    const da = i < lo.length ? DIGITS.indexOf(lo[i]) : 0;
    const db = open ? BASE : i < b!.length ? DIGITS.indexOf(b![i]) : BASE;
    if (da === db) {
      out += DIGITS[da];
      continue;
    }
    if (db - da > 1) return out + DIGITS[Math.floor((da + db) / 2)];
    out += DIGITS[da];
    open = true;
  }
}

/** Key for moving an item to `index` in `keys` (sorted, the item itself left out). */
export function keyAt(keys: string[], index: number): string {
  return keyBetween(keys[index - 1] ?? null, keys[index] ?? null);
}
