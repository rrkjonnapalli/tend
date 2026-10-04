import { describe, expect, it } from 'vitest';
import { nextIndex, prevIndex } from './list-nav.ts';

describe('list-nav', () => {
  it('moves down and stops at the last row', () => {
    expect(nextIndex(0, 3)).toBe(1);
    expect(nextIndex(2, 3)).toBe(2);
  });

  it('moves up and stops at the first row', () => {
    expect(prevIndex(2)).toBe(1);
    expect(prevIndex(0)).toBe(0);
  });
});
