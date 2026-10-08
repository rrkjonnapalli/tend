import { describe, expect, it } from 'vitest';
import { baseOf, isUnder, join, nameError, nameErrors, parentOf, pathError, relativeTo } from './names.ts';

describe('nameError', () => {
  it('accepts free text', () => {
    for (const ok of ['weekly_plan_2', 'Q4 Plan – draft', 'ideas (old)', 'Café notes']) expect(nameError(ok)).toBe('');
  });

  it('rejects what is unsafe on disk', () => {
    expect(nameError('')).toBe(nameErrors.empty);
    for (const bad of [' a', 'a ']) expect(nameError(bad)).toBe(nameErrors.spaces);
    for (const bad of ['a/b', 'a\\b', 'a:b', '.hidden', 'tab\there']) expect(nameError(bad)).toBe(nameErrors.chars);
    expect(nameError('x'.repeat(201))).toBe(nameErrors.long);
  });

  it('rejects a name already used in the folder, ignoring case', () => {
    expect(nameError('Ideas', ['ideas', 'plan'])).toBe(nameErrors.clash);
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
    expect(pathError('')).toBe('');
    expect(pathError('Work/Tend v0.1')).toBe('');
    expect(pathError('work/.git')).toBe(nameErrors.chars);
    expect(pathError('work//x')).toBe(nameErrors.empty);
  });
});
