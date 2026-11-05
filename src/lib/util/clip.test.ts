import { describe, expect, it } from 'vitest';
import { canPaste, selectedRoots } from './clip.ts';

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

describe('selectedRoots', () => {
  it('drops a subtask whose parent is selected too', () => {
    const items = [
      { id: 'p', parentId: null },
      { id: 'c', parentId: 'p' },
      { id: 'lone', parentId: 'other' },
    ];
    expect(selectedRoots(items)).toEqual(['p', 'lone']);
  });
});
