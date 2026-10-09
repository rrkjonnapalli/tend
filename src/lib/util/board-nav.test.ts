import { describe, expect, it } from 'vitest';
import { cardTarget, statusTarget } from './board-nav.ts';

describe('cardTarget', () => {
  const counts = [3, 0, 1, 2];

  it('moves up and down inside a column and stops at its ends', () => {
    expect(cardTarget('ArrowDown', { col: 0, idx: 1 }, counts)).toEqual({ col: 0, idx: 2 });
    expect(cardTarget('ArrowDown', { col: 0, idx: 2 }, counts)).toEqual({ col: 0, idx: 2 });
    expect(cardTarget('ArrowUp', { col: 0, idx: 0 }, counts)).toEqual({ col: 0, idx: 0 });
  });

  it('crosses to the next column with cards, keeping the row or taking its last card', () => {
    expect(cardTarget('ArrowRight', { col: 0, idx: 2 }, counts)).toEqual({ col: 2, idx: 0 });
    expect(cardTarget('ArrowRight', { col: 2, idx: 0 }, counts)).toEqual({ col: 3, idx: 0 });
    expect(cardTarget('ArrowLeft', { col: 2, idx: 0 }, counts)).toEqual({ col: 0, idx: 0 });
    expect(cardTarget('ArrowRight', { col: 3, idx: 1 }, counts)).toBeNull();
    expect(cardTarget('Enter', { col: 0, idx: 0 }, counts)).toBeNull();
  });
});

describe('statusTarget', () => {
  it('moves one status left or right, not past the ends', () => {
    expect(statusTarget('ArrowRight', 0, 4)).toBe(1);
    expect(statusTarget('ArrowLeft', 0, 4)).toBeNull();
    expect(statusTarget('ArrowRight', 3, 4)).toBeNull();
    expect(statusTarget('ArrowUp', 1, 4)).toBeNull();
  });
});
