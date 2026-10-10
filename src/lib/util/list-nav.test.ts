import { describe, expect, it } from 'vitest';
import { moveTo, nextIndex, prevIndex } from './list-nav.ts';

describe('list-nav', () => {
  it('moves down and stops at the last row', () => {
    expect(nextIndex(0, 3)).toBe(1);
    expect(nextIndex(2, 3)).toBe(2);
  });

  it('moves up and stops at the first row', () => {
    expect(prevIndex(2)).toBe(1);
    expect(prevIndex(0)).toBe(0);
  });

  it('moves the row cursor, and only ↓ leaves a field for the first row', () => {
    expect(moveTo({ key: 'ArrowDown', at: -1, count: 3 })).toBe(0);
    expect(moveTo({ key: 'ArrowUp', at: -1, count: 3 })).toBeNull();
    expect(moveTo({ key: 'End', at: -1, count: 3 })).toBeNull();
    expect(moveTo({ key: 'ArrowDown', at: 1, count: 3 })).toBe(2);
    expect(moveTo({ key: 'ArrowUp', at: 1, count: 3 })).toBe(0);
    expect(moveTo({ key: 'Home', at: 2, count: 3 })).toBe(0);
    expect(moveTo({ key: 'End', at: 0, count: 3 })).toBe(2);
    expect(moveTo({ key: 'ArrowDown', at: -1, count: 0 })).toBeNull();
    expect(moveTo({ key: 'x', at: 1, count: 3 })).toBeNull();
  });
});
