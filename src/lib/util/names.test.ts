import { describe, expect, it } from 'vitest';
import { baseOf, isUnder, isValidPath, join, nameError, nameErrors, parentOf, relativeTo } from './names.ts';

describe('nameError', () => {
  it('accepts lowercase, digits and underscore', () => {
    expect(nameError('weekly_plan_2')).toBe('');
  });

  it('rejects empty names, spaces, capitals, dashes and dots', () => {
    expect(nameError('')).toBe(nameErrors.empty);
    for (const bad of ['Weekly Plan', 'notes-2026', 'a.b', 'Work']) expect(nameError(bad)).toBe(nameErrors.chars);
  });

  it('rejects a name already used in the folder', () => {
    expect(nameError('ideas', ['ideas', 'plan'])).toBe(nameErrors.clash);
  });
});

describe('paths', () => {
  it('splits and joins', () => {
    expect(join('', 'a')).toBe('a');
    expect(join('a', 'b')).toBe('a/b');
    expect(parentOf('a/b/c')).toBe('a/b');
    expect(parentOf('a')).toBe('');
    expect(baseOf('a/b/c')).toBe('c');
    expect(relativeTo('work/tauri', 'work')).toBe('tauri');
    expect(relativeTo('work', 'work')).toBe('');
    expect(relativeTo('work/x', '')).toBe('work/x');
  });

  it('checks containment without prefix tricks', () => {
    expect(isUnder('work/a', 'work')).toBe(true);
    expect(isUnder('workshop', 'work')).toBe(false);
    expect(isUnder('x', '')).toBe(true);
  });

  it('validates every segment', () => {
    expect(isValidPath('')).toBe(true);
    expect(isValidPath('work/tend_v0_1')).toBe(true);
    expect(isValidPath('work/Tend')).toBe(false);
    expect(isValidPath('work//x')).toBe(false);
  });
});
