// The swatches offered for status and label colors, in picker order (five per row). Stored as
// hex in the database; the same values as tokens.css --sw-*.

export const SWATCHES: [string, string][] = [
  ['Grey', '#7d8995'],
  ['Slate', '#475569'],
  ['Blue', '#1259d3'],
  ['Sky', '#429ff5'],
  ['Indigo', '#3f51b5'],
  ['Violet', '#6f5ee4'],
  ['Lilac', '#b1b2ff'],
  ['Purple', '#9c36b5'],
  ['Magenta', '#c2255c'],
  ['Pink', '#f67280'],
  ['Red', '#da0037'],
  ['Orange', '#d57149'],
  ['Amber', '#e8890c'],
  ['Yellow', '#d4b106'],
  ['Olive', '#7a8f2a'],
  ['Green', '#1e7a55'],
  ['Mint', '#62d2a2'],
  ['Teal', '#0f8a8a'],
  ['Cyan', '#22a6c7'],
  ['Brown', '#8d6e63'],
];

/** True for a `#rrggbb` color, the form the database stores. */
export const isHexColor = (text: string) => /^#[0-9a-f]{6}$/i.test(text);

/** The first swatch the list doesn't use yet, or the first swatch when all are taken. */
export function freeSwatch(usedColors: string[]): string {
  const used = new Set(usedColors);
  const free = SWATCHES.find(([, hex]) => !used.has(hex)) ?? SWATCHES[0];
  return free[1];
}
