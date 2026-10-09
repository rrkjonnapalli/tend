import { describe, expect, it } from 'vitest';
import { fixable } from './skipped.ts';

describe('fixable', () => {
  it('keeps the items that have a fix', () => {
    const items = [
      { path: 'Café.md', fix: 'cafe.md' },
      { path: 'x.md', fix: '' },
    ];
    expect(fixable(items)).toEqual([items[0]]);
  });
});
