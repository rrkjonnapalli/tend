export const FIRST_STATUS = 'NOT_STARTED';
export const DONE_STATUS = 'DONE';

const MAX_VALUE_LENGTH = 32;

/** "In review" → "IN_REVIEW". Leading digits and stray underscores are dropped. */
export function statusValue(label: string): string {
  const upper = label.toUpperCase();
  const snake = upper.replace(/[^A-Z0-9]+/g, '_');
  const trimmed = snake.replace(/^[_0-9]+|_+$/g, '');
  return trimmed.slice(0, MAX_VALUE_LENGTH);
}
