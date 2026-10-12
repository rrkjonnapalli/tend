import { describe, expect, it } from 'vitest';
import { PAPER } from '#lib/enums/snap.enum.ts';
import type { Snap, Stack } from '#lib/types/snap.type.ts';
import { boardItems, columnCount, estimateHeight, packColumns, dropKey, parseDrop, boardLevel, isWithin, pathOf, stackPeek, stackSummary, stackTree, stickyTint, tagLabel, whereLabel } from './snaps.ts';

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
    expect(isWithin('reviews', 'work', stacks)).toBe(true);
    expect(isWithin('work', 'reviews', stacks)).toBe(false);
  });

  it('stops on a cycle', () => {
    const loop = [stack('a', 'b'), stack('b', 'a')];
    expect(pathOf('a', loop)).toHaveLength(2);
    expect(stackTree(loop)).toEqual([]);
  });

  it('tints a sticky from the picked colour', () => {
    expect(stickyTint('#d4b106')).toEqual({ band: 'hsl(50 85% 79%)', sheet: 'hsl(50 85% 87%)' });
    expect(stickyTint('#7d8995').sheet).toBe('hsl(210 10% 87%)');
    expect(stickyTint('#808080').band).toBe('hsl(0 0% 79%)');
  });

  it('labels tags, stacks and peeks', () => {
    expect(tagLabel(['poem', 'home'])).toBe('#poem +1');
    expect(tagLabel([])).toBe('');
    const all = [snap('n1', 'work', false, 'Ship the boring version'), snap('n2', 'work')];
    expect(stackSummary(stacks[0], all, stacks)).toBe('2 snaps · 1 stack');
    expect(stackPeek(stacks[0], all, stacks)).toBe('Ship the boring version');
    expect(stackPeek(stacks[0], [], stacks)).toBe('Code reviews');
  });

  it('spreads stacks between unpinned snaps, after the pinned ones', () => {
    const level = [snap('p', null, true), snap('a', null), snap('b', null), snap('c', null)];
    const order = boardItems(level, [stack('s1', null), stack('s2', null)]).map((i) => (i.kind === 'snap' ? i.snap.id : i.stack.id));
    expect(order).toEqual(['p', 'a', 's1', 'b', 's2', 'c']);
    expect(boardItems([], [stack('s', null)]).map((i) => i.kind)).toEqual(['stack']);
    const one = boardItems([snap('a', null), snap('b', null)], [stack('s', null)]);
    expect(one.map((i) => i.kind)).toEqual(['snap', 'stack', 'snap']);
  });

  it('shows one level, or every match while searching', () => {
    const all = [snap('n1', null, false, 'rain'), snap('n2', 'work', false, 'rain again')];
    expect(boardLevel(null, all, stacks, '').snaps.map((n) => n.id)).toEqual(['n1']);
    expect(boardLevel(null, all, stacks, 'rain').snaps.map((n) => n.id)).toEqual(['n1', 'n2']);
    expect(boardLevel(null, all, stacks, 'code').stacks.map((s) => s.id)).toEqual(['reviews']);
  });

  it('lists stacks depth-first for the picker', () => {
    expect(stackTree(stacks).map((o) => [o.stack.id, o.depth])).toEqual([['work', 1], ['reviews', 2], ['loose', 1]]);
  });

  it('round-trips drop keys', () => {
    expect(parseDrop(dropKey('crumb', null))).toEqual({ kind: 'crumb', id: null });
    expect(parseDrop(dropKey('stack', '01ABC'))).toEqual({ kind: 'stack', id: '01ABC' });
    expect(parseDrop('folder:x')).toBeNull();
  });

  it('fits columns and packs items into the shortest', () => {
    expect(columnCount(250, 250, 52)).toBe(1);
    expect(columnCount(551, 250, 52)).toBe(1);
    expect(columnCount(552, 250, 52)).toBe(2);
    expect(columnCount(0, 250, 52)).toBe(1);
    const sizes: Record<string, number> = { tall: 900, a: 100, b: 100, c: 100 };
    const cols = packColumns(['tall', 'a', 'b', 'c'], 2, (x) => sizes[x]);
    expect(cols.map((c) => c.map((x) => `${x.item}${x.at}`))).toEqual([['tall0'], ['a1', 'b2', 'c3']]);
  });

  it('estimates taller for longer text and keeps each paper least height', () => {
    const short = estimateHeight({ kind: 'snap', snap: snap('s', null, false, 'hi') }, 270);
    const long = estimateHeight({ kind: 'snap', snap: snap('l', null, false, 'word '.repeat(80)) }, 270);
    expect(short).toBe(150 + 44);
    expect(long).toBeGreaterThan(short);
    expect(estimateHeight({ kind: 'stack', stack: stack('s', null) }, 270)).toBe(274);
  });
});
