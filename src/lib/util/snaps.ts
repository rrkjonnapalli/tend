// Snaps: stack paths and names, the board's order, a sticky's tint, and the small labels on paper.
import { PAPER, type Paper } from '#lib/enums/snap.enum.ts';
import type { BoardItem, Snap, Stack } from '#lib/types/snap.type.ts';
import { plural } from './plural.ts';
import { textMatches } from './search.ts';

export const UNTITLED_STACK = 'Untitled stack';
/** The editor's stack picker choice "+ New stack": one is made where the board is. */
export const NEW_STACK = 'new';
/** The board itself, as the path and the stack picker name it. */
export const BOARD_NAME = 'Snaps';

const STICKY_SATURATION_MAX = 85;
const STICKY_BAND_LIGHTNESS = 79;
const STICKY_SHEET_LIGHTNESS = 87;

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
export function isWithin(id: string | null, ancestor: string, stacks: Stack[]): boolean {
  return pathOf(id, stacks).some((s) => s.id === ancestor);
}

/** A sticky's paper: the picked colour's hue, light enough that dark text always reads. */
export function stickyTint(hex: string): { band: string; sheet: string } {
  const { h, s } = hueAndSaturation(hex);
  const sat = Math.min(s, STICKY_SATURATION_MAX);
  return {
    band: `hsl(${h} ${sat}% ${STICKY_BAND_LIGHTNESS}%)`,
    sheet: `hsl(${h} ${sat}% ${STICKY_SHEET_LIGHTNESS}%)`,
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

const snapsIn = (id: string, snaps: Snap[]) => snaps.filter((n) => n.stackId === id);
const stacksIn = (id: string, stacks: Stack[]) => stacks.filter((s) => s.parentId === id);

/** "4 snaps · 1 stack" */
export function stackSummary(stack: Stack, snaps: Snap[], stacks: Stack[]): string {
  const parts = [plural(snapsIn(stack.id, snaps).length, 'snap')];
  const kids = stacksIn(stack.id, stacks).length;
  if (kids > 0) parts.push(plural(kids, 'stack'));
  return parts.join(' · ');
}

/** What shows on a stack's top sheet: its latest snap's text, else its sub-stacks' names. */
export function stackPeek(stack: Stack, snaps: Snap[], stacks: Stack[]): string {
  const top = snapsIn(stack.id, snaps)[0];
  if (top) return top.text;
  return stacksIn(stack.id, stacks).map(stackName).join('\n');
}

/** What one level of the board holds, or, while searching, every match across all stacks. */
export function boardLevel(open: string | null, snaps: Snap[], stacks: Stack[], needle: string): { snaps: Snap[]; stacks: Stack[] } {
  if (needle) {
    return {
      snaps: snaps.filter((n) => textMatches([n.text], needle)),
      stacks: stacks.filter((s) => textMatches([s.name], needle)),
    };
  }
  return { snaps: snaps.filter((n) => n.stackId === open), stacks: stacks.filter((s) => s.parentId === open) };
}

/** The board's order: snaps as given (pinned first, then the latest edit), with the stacks spread
 *  evenly between the unpinned ones. */
export function boardItems(levelSnaps: Snap[], levelStacks: Stack[]): BoardItem[] {
  const items: BoardItem[] = levelSnaps.map((snap) => ({ kind: 'snap', snap }));
  const pinned = levelSnaps.filter((n) => n.pinned).length;
  const rest = levelSnaps.length - pinned;
  const count = levelStacks.length;
  levelStacks.forEach((stack, i) => {
    // Slot i+1 of count+1 equal parts of the final run (unpinned snaps and stacks together).
    const even = Math.round(((i + 1) * (rest + count + 1)) / (count + 1)) - 1;
    const at = pinned + Math.min(rest + i, even);
    items.splice(Math.max(0, at), 0, { kind: 'stack', stack });
  });
  return items;
}

/** A board item's key: its snap's or stack's id. */
export function itemKey(item: BoardItem): string {
  if (item.kind === 'snap') return item.snap.id;
  return item.stack.id;
}

/** Every stack, depth-first under its parent, for the editor's stack picker. */
export function stackTree(stacks: Stack[]): { stack: Stack; depth: number }[] {
  const out: { stack: Stack; depth: number }[] = [];
  const walk = (parent: string | null, depth: number) => {
    for (const stack of stacks.filter((s) => s.parentId === parent)) {
      out.push({ stack, depth });
      // A cycle (only mid-sync) would recurse forever; no stack appears twice.
      if (out.filter((o) => o.stack.id === stack.id).length === 1) walk(stack.id, depth + 1);
    }
  };
  walk(null, 1);
  return out;
}

/** What can be dropped on: a snap (pair them), a stack (put it in), a step of the path (move up). */
export type DropTarget = { kind: 'snap' | 'stack' | 'crumb'; id: string | null };

const BOARD_KEY = 'root';

/** The `data-drop` key of a drop target, as the board's items and the path carry it. */
export function dropKey(kind: DropTarget['kind'], id: string | null): string {
  return `${kind}:${id ?? BOARD_KEY}`;
}

export function parseDrop(key: string): DropTarget | null {
  const [kind, id] = key.split(':');
  const known = kind === 'snap' || kind === 'stack' || kind === 'crumb';
  if (!known || !id) return null;
  return { kind, id: id === BOARD_KEY ? null : id };
}

/** How many columns of `column` px, `gap` px apart, fit in `width` px (always at least one). */
export function columnCount(width: number, column: number, gap: number): number {
  return Math.max(1, Math.floor((width + gap) / (column + gap)));
}

/** How each paper sets text, for estimating heights: line height, average character width and
 *  the paper's padding (px), and its least height. Close enough to pick a column, nothing more. */
const PAPER_METRICS: Record<Paper, { line: number; char: number; padX: number; padY: number; min: number }> = {
  [PAPER.scrap]: { line: 24, char: 8.4, padX: 40, padY: 58, min: 150 },
  [PAPER.slip]: { line: 20, char: 7.8, padX: 36, padY: 40, min: 0 },
  [PAPER.lined]: { line: 24, char: 8.4, padX: 64, padY: 42, min: 170 },
  [PAPER.plain]: { line: 24, char: 8.4, padX: 40, padY: 42, min: 0 },
  [PAPER.sticky]: { line: 28, char: 10, padX: 40, padY: 42, min: 220 },
};
/** A snap's date line and the gap above it; a stack's pile with its clip; the space under items. */
const FOOT_PX = 28;
const STACK_PX = 230;
const ITEM_GAP_PX = 44;

/** About how tall an item is in a column `width` px wide. */
export function estimateHeight(item: BoardItem, width: number): number {
  if (item.kind === 'stack') return STACK_PX + ITEM_GAP_PX;
  const m = PAPER_METRICS[item.snap.paper];
  const perLine = Math.max(1, Math.floor((width - m.padX) / m.char));
  const lines = item.snap.text.split('\n').reduce((sum, line) => sum + Math.max(1, Math.ceil(line.length / perLine)), 0);
  const sheet = Math.max(m.min, m.padY + lines * m.line + FOOT_PX);
  return sheet + ITEM_GAP_PX;
}

/** The items in order, each put in the shortest of `count` columns (the leftmost on a tie), so
 *  the board fills evenly and the newest lead the top row. Each keeps its place in the full order
 *  (for its tilt). */
export function packColumns<T>(items: T[], count: number, heightOf: (item: T) => number): { item: T; at: number }[][] {
  const columns: { item: T; at: number }[][] = Array.from({ length: count }, () => []);
  const heights = new Array<number>(count).fill(0);
  items.forEach((item, at) => {
    const shortest = heights.indexOf(Math.min(...heights));
    columns[shortest].push({ item, at });
    heights[shortest] += heightOf(item);
  });
  return columns;
}
