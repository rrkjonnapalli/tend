// Snaps: stack paths and names, the board's order, a sticky's tint, and the small labels on paper.
import { BOARD_KIND, DROP_KIND, PAPER, type DropKind, type Paper } from '#lib/enums/snap.enum.ts';
import type { Board, BoardColumn, BoardItem, BoardView, DropTarget, Nesting, Packing, Placed, Snap, Stack, StackChoice } from '#lib/types/snap.type.ts';
import { plural } from './plural.ts';
import { textMatches } from './search.ts';

const UNTITLED_STACK = 'Untitled stack';
/** The editor's stack picker choice "+ New stack": one is made where the board is. */
export const NEW_STACK = 'new';
/** The board itself, as the path and the stack picker name it. */
export const BOARD_NAME = 'Snaps';

const STICKY_SATURATION_MAX = 85;

export const stackName = (stack: Stack) => stack.name.trim() || UNTITLED_STACK;

/** The stacks from the board down to `id` (none for the board). Stops after as many steps as
 *  there are stacks, so a cycle (only possible mid-sync) can't loop. */
export function pathOf(id: string | null, stacks: Stack[]): Stack[] {
  const path: Stack[] = [];
  let current = stacks.find((s) => s.id === id);
  for (let step = 0; current && step < stacks.length; step += 1) {
    path.unshift(current);
    const parent = current.parentId;
    current = stacks.find((s) => s.id === parent);
  }
  return path;
}

/** "Snaps" for the board, else the path's names: "Work rules / Code reviews". */
export function whereLabel(id: string | null, stacks: Stack[]): string {
  const path = pathOf(id, stacks);
  if (path.length === 0) return BOARD_NAME;
  return path.map(stackName).join(' / ');
}

/** True when `id` is `ancestor` or sits inside it. */
export function isWithin({ id, ancestor }: Nesting, stacks: Stack[]): boolean {
  return pathOf(id, stacks).some((s) => s.id === ancestor);
}

/** A sticky's paper: the picked colour's hue, at the theme's sheet and band lightness (tokens.css
 *  --sticky-*), so the board's ink always reads on it, light or dark. */
export function stickyTint(hex: string): { band: string; sheet: string } {
  const { h, s } = hueAndSaturation(hex);
  const sat = Math.min(s, STICKY_SATURATION_MAX);
  const saturation = `calc(${sat}% * var(--sticky-sat))`;
  return {
    band: `hsl(${h} ${saturation} var(--sticky-band-l))`,
    sheet: `hsl(${h} ${saturation} var(--sticky-sheet-l))`,
  };
}

function hueAndSaturation(hex: string): { h: number; s: number } {
  const channel = (at: number) => parseInt(hex.slice(at, at + 2), 16) / 255;
  const [r, g, b] = [channel(1), channel(3), channel(5)];
  const max = Math.max(r, g, b);
  const min = Math.min(r, g, b);
  const lightness = (max + min) / 2;
  const range = max - min;
  if (range === 0) return { h: 0, s: 0 };
  const sat = lightness > 0.5 ? range / (2 - max - min) : range / (max + min);
  let sector: number;
  if (max === r) sector = (g - b) / range + (g < b ? 6 : 0);
  else if (max === g) sector = (b - r) / range + 2;
  else sector = (r - g) / range + 4;
  return { h: Math.round(sector * 60), s: Math.round(sat * 100) };
}

/** The first tag and how many more: "#poem +1". */
export function tagLabel(tags: string[]): string {
  if (tags.length === 0) return '';
  const more = tags.length > 1 ? ` +${tags.length - 1}` : '';
  return `#${tags[0]}${more}`;
}

/** The snaps right in a stack (`null`: loose on the board). */
export const snapsIn = (id: string | null, snaps: Snap[]) => snaps.filter((n) => n.stackId === id);
/** The stacks right in a stack (`null`: on the board). */
export const stacksIn = (id: string | null, stacks: Stack[]) => stacks.filter((s) => s.parentId === id);

/** "4 snaps · 1 stack" */
export function stackSummary(stack: Stack, board: Board): string {
  const parts = [plural(snapsIn(stack.id, board.snaps).length, 'snap')];
  const kids = stacksIn(stack.id, board.stacks).length;
  if (kids > 0) parts.push(plural(kids, 'stack'));
  return parts.join(' · ');
}

/** What shows on a stack's top sheet: its latest snap's text, else its sub-stacks' names. */
export function stackPeek(stack: Stack, board: Board): string {
  const top = snapsIn(stack.id, board.snaps)[0];
  if (top) return top.text;
  return stacksIn(stack.id, board.stacks).map(stackName).join('\n');
}

/** What one level of the board holds, or, while searching, every match across all stacks. */
export function boardLevel({ open, needle }: BoardView, board: Board): Board {
  if (needle) {
    return {
      snaps: board.snaps.filter((n) => textMatches([n.text], needle)),
      stacks: board.stacks.filter((s) => textMatches([s.name], needle)),
    };
  }
  return { snaps: snapsIn(open, board.snaps), stacks: stacksIn(open, board.stacks) };
}

/** Where stack `nth` (from 0) of `count` goes among the `rest` unpinned snaps: the end of part
 *  nth+1 when the final run (those snaps and the stacks together) is cut in count+1 equal parts,
 *  but never past the items already there. */
function stackSlot({ nth, count, rest }: { nth: number; count: number; rest: number }): number {
  const run = rest + count;
  const parts = count + 1;
  const even = Math.round(((nth + 1) * (run + 1)) / parts) - 1;
  const placed = rest + nth;
  return Math.max(0, Math.min(placed, even));
}

/** The board's order: snaps as given (pinned first, then the latest edit), with the stacks spread
 *  evenly between the unpinned ones. */
export function boardItems(levelSnaps: Snap[], levelStacks: Stack[]): BoardItem[] {
  const items: BoardItem[] = levelSnaps.map((snap) => ({ kind: BOARD_KIND.snap, snap }));
  const pinned = levelSnaps.filter((n) => n.pinned).length;
  const rest = levelSnaps.length - pinned;
  const count = levelStacks.length;
  for (let nth = 0; nth < count; nth += 1) {
    const at = pinned + stackSlot({ nth, count, rest });
    items.splice(at, 0, { kind: BOARD_KIND.stack, stack: levelStacks[nth] });
  }
  return items;
}

/** A board item's key: its snap's or stack's id. */
export function itemKey(item: BoardItem): string {
  if (item.kind === BOARD_KIND.snap) return item.snap.id;
  return item.stack.id;
}

/** Every stack, depth-first under its parent, for the editor's stack picker. */
export function stackTree(stacks: Stack[]): Stack[] {
  const out: Stack[] = [];
  const walk = (parent: string | null) => {
    for (const stack of stacksIn(parent, stacks)) {
      out.push(stack);
      // No guard needed: stacks in a cycle (only mid-sync) have no path from the board, so they are never reached.
      walk(stack.id);
    }
  };
  walk(null);
  return out;
}

const BOARD_KEY = 'root';

/** The editor's stack picker: the board, a new stack where the board is (`open`), then every
 *  stack by its path. */
export function stackChoices(stacks: Stack[], open: string | null): StackChoice[] {
  const board: StackChoice = { id: BOARD_KEY, stackId: null, label: `${BOARD_NAME} (loose)` };
  const openStack = stacks.find((s) => s.id === open);
  const inside = openStack ? ` in ${stackName(openStack)}` : '';
  const fresh: StackChoice = { id: NEW_STACK, stackId: NEW_STACK, label: `+ New stack${inside}` };
  const nested = stackTree(stacks).map((s) => ({ id: s.id, stackId: s.id, label: whereLabel(s.id, stacks) }));
  return [board, fresh, ...nested];
}

/** True when the choice's line contains the lower-cased query. */
export const stackChoiceMatches = (choice: StackChoice, query: string) => textMatches([choice.label], query);

/** The `data-drop` key of a drop target, as the board's items and the path carry it. */
export function dropKey(kind: DropKind, id: string | null): string {
  return `${kind}:${id ?? BOARD_KEY}`;
}

const DROP_KINDS: readonly string[] = Object.values(DROP_KIND);
const isDropKind = (kind: string): kind is DropKind => DROP_KINDS.includes(kind);

export function parseDrop(key: string): DropTarget | null {
  const [kind, id] = key.split(':');
  if (!isDropKind(kind) || !id) return null;
  return { kind, id: id === BOARD_KEY ? null : id };
}

/** How many columns fit in `width` px (always at least one). */
export function columnCount(width: number, { minWidth, gap }: BoardColumn): number {
  return Math.max(1, Math.floor((width + gap) / (minWidth + gap)));
}

/** How each paper sets text, for estimating heights: line height, average character width and
 *  the paper's padding (px), and its least height. Close enough to pick a column, nothing more. */
const PAPER_METRICS: Record<Paper, { line: number; char: number; padX: number; padY: number; min: number }> = {
  [PAPER.scrap]: { line: 24, char: 8.4, padX: 40, padY: 58, min: 150 },
  [PAPER.slip]: { line: 20, char: 7.8, padX: 36, padY: 40, min: 0 },
  [PAPER.lined]: { line: 24, char: 8.4, padX: 64, padY: 42, min: 170 },
  [PAPER.plain]: { line: 24, char: 8.4, padX: 40, padY: 42, min: 0 },
  [PAPER.sticky]: { line: 28, char: 8.4, padX: 40, padY: 42, min: 220 },
};
/** A snap's date line and the gap above it; a stack's pile with its clip; the space under items. */
const FOOT_PX = 28;
const STACK_PX = 230;
const ITEM_GAP_PX = 44;

/** How many lines `text` takes when a line holds `perLine` characters. */
function wrappedLines(text: string, perLine: number): number {
  let lines = 0;
  for (const line of text.split('\n')) {
    lines += Math.max(1, Math.ceil(line.length / perLine));
  }
  return lines;
}

/** About how tall an item is in a column `width` px wide. */
export function estimateHeight(item: BoardItem, width: number): number {
  if (item.kind === BOARD_KIND.stack) return STACK_PX + ITEM_GAP_PX;
  const m = PAPER_METRICS[item.snap.paper];
  const perLine = Math.max(1, Math.floor((width - m.padX) / m.char));
  const lines = wrappedLines(item.snap.text, perLine);
  const sheet = Math.max(m.min, m.padY + lines * m.line + FOOT_PX);
  return sheet + ITEM_GAP_PX;
}

/** The items in order, each put in the shortest of `count` columns (the leftmost on a tie), so
 *  the board fills evenly and the newest lead the top row. Each keeps its place in the full order
 *  (for its tilt). */
export function packColumns<T>({ items, count, heightOf }: Packing<T>): Placed<T>[][] {
  const columns: Placed<T>[][] = Array.from({ length: count }, () => []);
  const heights = new Array<number>(count).fill(0);
  items.forEach((item, at) => {
    const shortest = heights.indexOf(Math.min(...heights));
    columns[shortest].push({ item, at });
    heights[shortest] += heightOf(item);
  });
  return columns;
}
