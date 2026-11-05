import { describe, expect, it } from 'vitest';
import { keyboardHeight } from './keyboard.ts';

describe('keyboardHeight', () => {
  it('is what the visible area lost to the keyboard', () => {
    expect(keyboardHeight({ window: 914, visible: 577.9 })).toBe(336);
  });

  it('is never below zero (a page zoomed or the keyboard down)', () => {
    expect(keyboardHeight({ window: 914, visible: 914 })).toBe(0);
    expect(keyboardHeight({ window: 914, visible: 920 })).toBe(0);
  });
});
