import { describe, expect, it } from 'vitest';
import { canPaste } from './clip.ts';

describe('canPaste', () => {
  it('refuses the folder every item is already in', () => {
    expect(canPaste({ from: ['home', 'home'], target: 'home' })).toBe(false);
  });

  it('allows any other folder, or a mix that is partly elsewhere', () => {
    expect(canPaste({ from: ['home'], target: 'work' })).toBe(true);
    expect(canPaste({ from: ['home', 'work'], target: 'work' })).toBe(true);
    expect(canPaste({ from: ['home'], target: '' })).toBe(true);
  });
});
