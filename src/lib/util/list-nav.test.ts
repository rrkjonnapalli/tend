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
    expect(moveTo('ArrowDown', -1, 3)).toBe(0);
    expect(moveTo('ArrowUp', -1, 3)).toBeNull();
    expect(moveTo('End', -1, 3)).toBeNull();
    expect(moveTo('ArrowDown', 1, 3)).toBe(2);
    expect(moveTo('ArrowUp', 1, 3)).toBe(0);
    expect(moveTo('Home', 2, 3)).toBe(0);
    expect(moveTo('End', 0, 3)).toBe(2);
    expect(moveTo('ArrowDown', -1, 0)).toBeNull();
    expect(moveTo('x', 1, 3)).toBeNull();
  });
});
