export const FIRST_STATUS = 'NOT_STARTED';
export const DONE_STATUS = 'DONE';

/** The ten swatches offered for a status colour. Stored as hex in the DB; same values as tokens.css --sw-*. */
export const SWATCHES: [string, string][] = [
  ['Grey', '#7d8995'],
  ['Blue', '#1259d3'],
  ['Sky', '#429ff5'],
  ['Violet', '#6f5ee4'],
  ['Lilac', '#b1b2ff'],
  ['Green', '#1e7a55'],
  ['Mint', '#62d2a2'],
  ['Orange', '#d57149'],
  ['Red', '#da0037'],
  ['Pink', '#f67280'],
];

const MAX_VALUE_LENGTH = 32;

/** "In review" → "IN_REVIEW". Leading digits and stray underscores are dropped. */
export function statusValue(label: string): string {
  const upper = label.toUpperCase();
  const snake = upper.replace(/[^A-Z0-9]+/g, '_');
  const trimmed = snake.replace(/^[_0-9]+|_+$/g, '');
  return trimmed.slice(0, MAX_VALUE_LENGTH);
}
