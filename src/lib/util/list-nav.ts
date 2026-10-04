// The highlighted row in a keyboard-driven list (↑↓), kept inside the list.

/** The row below `index`, stopping at the last of `count` rows. */
export const nextIndex = (index: number, count: number) => Math.min(index + 1, count - 1);

/** The row above `index`, stopping at the first. */
export const prevIndex = (index: number) => Math.max(index - 1, 0);
