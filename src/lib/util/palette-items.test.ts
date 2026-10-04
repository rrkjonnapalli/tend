import { describe, expect, it } from 'vitest';
import { groupItems, hasAllWords } from './palette-items.ts';

describe('hasAllWords', () => {
  it('needs every word, in any order and case', () => {
    expect(hasAllWords('Go to Notes', 'notes go')).toBe(true);
    expect(hasAllWords('Go to Notes', 'notes tasks')).toBe(false);
  });
});

describe('groupItems', () => {
  it('keeps first-seen group order and the overall index', () => {
    const item = (key: string, group: string) => ({ key, group, icon: '', label: key, run: () => {} });
    const groups = groupItems([item('a', 'Notes'), item('b', 'Tasks'), item('c', 'Notes')]);
    expect(groups.map((g) => [g.name, g.items.map((x) => x.index)])).toEqual([
      ['Notes', [0, 2]],
      ['Tasks', [1]],
    ]);
  });
});
