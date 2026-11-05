import { describe, expect, it } from 'vitest';
import { keyboardHeight } from './keyboard.ts';

describe('keyboardHeight', () => {
  it('is what the visible area lost to the keyboard', () => {
    expect(keyboardHeight(914, 577.9)).toBe(336);
  });

  it('is never below zero (a page zoomed or the keyboard down)', () => {
    expect(keyboardHeight(914, 914)).toBe(0);
    expect(keyboardHeight(914, 920)).toBe(0);
  });
});
