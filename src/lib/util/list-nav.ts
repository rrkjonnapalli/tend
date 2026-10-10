// The highlighted row in a keyboard-driven list (↑↓), kept inside the list.

/** The row below `index`, stopping at the last of `count` rows. */
export const nextIndex = (index: number, count: number) => Math.min(index + 1, count - 1);

/** The row above `index`, stopping at the first. */
export const prevIndex = (index: number) => Math.max(index - 1, 0);

/** Where a key moves the keyboard cursor in a list of `count` rows, from row `at` (-1 when the
 *  cursor is in a field above the list). `null` when the key doesn't move it. */
export function moveTo({ key, at, count }: { key: string; at: number; count: number }): number | null {
  if (count === 0) return null;
  const inField = at < 0;
  if (key === 'ArrowDown') return inField ? 0 : nextIndex(at, count);
  if (inField) return null;
  if (key === 'ArrowUp') return prevIndex(at);
  if (key === 'Home') return 0;
  if (key === 'End') return count - 1;
  return null;
}
