import { describe, expect, it } from 'vitest';
import { cardTarget, statusTarget } from './board-nav.ts';

describe('cardTarget', () => {
  const counts = [3, 0, 1, 2];

  it('moves up and down inside a column and stops at its ends', () => {
    expect(cardTarget({ key: 'ArrowDown', at: { col: 0, idx: 1 }, counts })).toEqual({ col: 0, idx: 2 });
    expect(cardTarget({ key: 'ArrowDown', at: { col: 0, idx: 2 }, counts })).toEqual({ col: 0, idx: 2 });
    expect(cardTarget({ key: 'ArrowUp', at: { col: 0, idx: 0 }, counts })).toEqual({ col: 0, idx: 0 });
  });

  it('crosses to the next column with cards, keeping the row or taking its last card', () => {
    expect(cardTarget({ key: 'ArrowRight', at: { col: 0, idx: 2 }, counts })).toEqual({ col: 2, idx: 0 });
    expect(cardTarget({ key: 'ArrowRight', at: { col: 2, idx: 0 }, counts })).toEqual({ col: 3, idx: 0 });
    expect(cardTarget({ key: 'ArrowLeft', at: { col: 2, idx: 0 }, counts })).toEqual({ col: 0, idx: 0 });
    expect(cardTarget({ key: 'ArrowRight', at: { col: 3, idx: 1 }, counts })).toBeNull();
    expect(cardTarget({ key: 'Enter', at: { col: 0, idx: 0 }, counts })).toBeNull();
  });
});

describe('statusTarget', () => {
  it('moves one status left or right, not past the ends', () => {
    expect(statusTarget({ key: 'ArrowRight', col: 0, columns: 4 })).toBe(1);
    expect(statusTarget({ key: 'ArrowLeft', col: 0, columns: 4 })).toBeNull();
    expect(statusTarget({ key: 'ArrowRight', col: 3, columns: 4 })).toBeNull();
    expect(statusTarget({ key: 'ArrowUp', col: 1, columns: 4 })).toBeNull();
  });
});
