import { describe, expect, it } from 'vitest';
import { BOARD_KIND, DROP_KIND, PAPER } from '#lib/enums/snap.enum.ts';
import type { Snap, Stack } from '#lib/types/snap.type.ts';
import { boardItems, columnCount, estimateHeight, packColumns, dropKey, parseDrop, boardLevel, isWithin, NEW_STACK, pathOf, stackChoiceMatches, stackChoices, stackPeek, stackSummary, stackTree, stickyTint, itemKey, tagLabel, whereLabel } from './snaps.ts';

const stack = (id: string, parentId: string | null, name = id): Stack => ({ id, parentId, name, updatedAt: 0 });
const snap = (id: string, stackId: string | null, pinned = false, text = id): Snap => ({
  id, stackId, text, paper: PAPER.scrap, color: null, pinned, tags: [], createdAt: 0, updatedAt: 0,
});

const stacks = [stack('work', null, 'Work rules'), stack('reviews', 'work', 'Code reviews'), stack('loose', null, '  ')];

describe('snaps', () => {
  it('walks stack paths', () => {
    expect(pathOf('reviews', stacks).map((s) => s.id)).toEqual(['work', 'reviews']);
    expect(whereLabel('reviews', stacks)).toBe('Work rules / Code reviews');
    expect(whereLabel(null, stacks)).toBe('Snaps');
    expect(whereLabel('loose', stacks)).toBe('Untitled stack');
    expect(isWithin({ id: 'reviews', ancestor: 'work' }, stacks)).toBe(true);
    expect(isWithin({ id: 'work', ancestor: 'reviews' }, stacks)).toBe(false);
    expect(isWithin({ id: 'work', ancestor: 'work' }, stacks)).toBe(true);
    expect(pathOf('gone', stacks)).toEqual([]);
  });

  it('stops on a cycle', () => {
    const loop = [stack('a', 'b'), stack('b', 'a')];
    expect(pathOf('a', loop)).toHaveLength(2);
    expect(stackTree(loop)).toEqual([]);
  });

  it('tints a sticky from the picked colour', () => {
    const tint = (h: number, s: number) => ({
      band: `hsl(${h} calc(${s}% * var(--sticky-sat)) var(--sticky-band-l))`,
      sheet: `hsl(${h} calc(${s}% * var(--sticky-sat)) var(--sticky-sheet-l))`,
    });
    expect(stickyTint('#d4b106')).toEqual(tint(50, 85));
    expect(stickyTint('#7d8995').sheet).toBe(tint(210, 10).sheet);
    expect(stickyTint('#808080').band).toBe(tint(0, 0).band);
    // Green is the strongest channel.
    expect(stickyTint('#20c040').sheet).toBe(tint(132, 71).sheet);
    // Red is, with less green than blue: the hue wraps past 300.
    expect(stickyTint('#c02080').sheet).toBe(tint(324, 71).sheet);
  });

  it('labels tags, stacks and peeks', () => {
    expect(tagLabel(['poem', 'home'])).toBe('#poem +1');
    expect(tagLabel([])).toBe('');
    const all = [snap('n1', 'work', false, 'Ship the boring version'), snap('n2', 'work')];
    expect(stackSummary(stacks[0], { snaps: all, stacks })).toBe('2 snaps · 1 stack');
    expect(stackPeek(stacks[0], { snaps: all, stacks })).toBe('Ship the boring version');
    expect(stackPeek(stacks[0], { snaps: [], stacks })).toBe('Code reviews');
  });

  it('spreads stacks between unpinned snaps, after the pinned ones', () => {
    const level = [snap('p', null, true), snap('a', null), snap('b', null), snap('c', null)];
    const order = boardItems(level, [stack('s1', null), stack('s2', null)]).map(itemKey);
    expect(order).toEqual(['p', 'a', 's1', 'b', 's2', 'c']);
    const crowded = boardItems([snap('p', null, true), snap('a', null)], [stack('s1', null), stack('s2', null), stack('s3', null)]);
    expect(crowded.map(itemKey)).toEqual(['p', 's1', 'a', 's2', 's3']);
    expect(boardItems([], [stack('s', null)]).map((i) => i.kind)).toEqual(['stack']);
    const one = boardItems([snap('a', null), snap('b', null)], [stack('s', null)]);
    expect(one.map((i) => i.kind)).toEqual(['snap', 'stack', 'snap']);
  });

  it('shows one level, or every match while searching', () => {
    const all = [snap('n1', null, false, 'rain'), snap('n2', 'work', false, 'rain again')];
    const board = { snaps: all, stacks };
    expect(boardLevel({ open: null, needle: '' }, board).snaps.map((n) => n.id)).toEqual(['n1']);
    expect(boardLevel({ open: 'work', needle: '' }, board).stacks.map((s) => s.id)).toEqual(['reviews']);
    expect(boardLevel({ open: null, needle: 'rain' }, board).snaps.map((n) => n.id)).toEqual(['n1', 'n2']);
    expect(boardLevel({ open: null, needle: 'code' }, board).stacks.map((s) => s.id)).toEqual(['reviews']);
  });

  it('lists stacks depth-first for the picker', () => {
    expect(stackTree(stacks).map((s) => s.id)).toEqual(['work', 'reviews', 'loose']);
  });

  it('offers the board, a new stack, then every stack by its path', () => {
    const choices = stackChoices(stacks, 'work');
    expect(choices.map((c) => c.label)).toEqual(['Snaps (loose)', '+ New stack in Work rules', 'Work rules', 'Work rules / Code reviews', 'Untitled stack']);
    expect(choices.map((c) => c.stackId)).toEqual([null, NEW_STACK, 'work', 'reviews', 'loose']);
    expect(stackChoices(stacks, null)[1].label).toBe('+ New stack');
    expect(choices.filter((c) => stackChoiceMatches(c, 'code')).map((c) => c.id)).toEqual(['reviews']);
    expect(choices.filter((c) => stackChoiceMatches(c, 'nothing like it'))).toEqual([]);
  });

  it('round-trips drop keys', () => {
    expect(parseDrop(dropKey(DROP_KIND.crumb, null))).toEqual({ kind: DROP_KIND.crumb, id: null });
    expect(parseDrop(dropKey(DROP_KIND.stack, '01ABC'))).toEqual({ kind: DROP_KIND.stack, id: '01ABC' });
    expect(parseDrop('folder:x')).toBeNull();
    expect(parseDrop('snap:')).toBeNull();
    expect(parseDrop('snap')).toBeNull();
  });

  it('fits columns and packs items into the shortest', () => {
    const column = { minWidth: 250, gap: 52 };
    expect(columnCount(250, column)).toBe(1);
    expect(columnCount(551, column)).toBe(1);
    expect(columnCount(552, column)).toBe(2);
    expect(columnCount(0, column)).toBe(1);
    const sizes: Record<string, number> = { tall: 900, a: 100, b: 100, c: 100 };
    const cols = packColumns({ items: ['tall', 'a', 'b', 'c'], count: 2, heightOf: (x) => sizes[x] });
    expect(cols.map((c) => c.map((x) => `${x.item}${x.at}`))).toEqual([['tall0'], ['a1', 'b2', 'c3']]);
  });

  it('estimates taller for longer text and keeps each paper least height', () => {
    const short = estimateHeight({ kind: BOARD_KIND.snap, snap: snap('s', null, false, 'hi') }, 270);
    const long = estimateHeight({ kind: BOARD_KIND.snap, snap: snap('l', null, false, 'word '.repeat(80)) }, 270);
    expect(short).toBe(150 + 44);
    expect(long).toBeGreaterThan(short);
    expect(estimateHeight({ kind: BOARD_KIND.stack, stack: stack('s', null) }, 270)).toBe(274);
  });
});
