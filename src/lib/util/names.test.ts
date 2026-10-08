import { describe, expect, it } from 'vitest';
import { baseOf, cleanName, cleanPath, isUnder, join, nameError, nameErrors, parentOf, pathError, relativeTo } from './names.ts';

describe('nameError', () => {
  it('accepts letters, digits, spaces and . , _ -', () => {
    for (const ok of ['weekly_plan_2', 'Q4 Plan - draft', 'notes, v1.2', ' .trimmed. ']) expect(nameError(ok)).toBe('');
  });

  it('rejects empty names, other characters and long names', () => {
    for (const empty of ['', ' . ']) expect(nameError(empty)).toBe(nameErrors.empty);
    for (const bad of ['a/b', 'a:b', 'Café', 'ideas (old)']) expect(nameError(bad)).toBe(nameErrors.chars);
    expect(nameError('x'.repeat(201))).toBe(nameErrors.long);
  });

  it('cleans what it will save', () => {
    expect(cleanName('  .Plan. ')).toBe('Plan');
    expect(cleanPath(' /Work / .Ideas/ ')).toBe('Work/Ideas');
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
    expect(pathError('work/a:b')).toBe(nameErrors.chars);
    expect(pathError('work//x')).toBe(nameErrors.empty);
  });
});
