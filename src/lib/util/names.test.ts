import { describe, expect, it } from 'vitest';
import { baseOf, cleanPath, isUnder, join, nameError, nameErrors, parentOf, pathError, relativeTo, slug, titleError } from './names.ts';

describe('names', () => {
  it('slugs text the way Rust does', () => {
    expect(slug('Meeting')).toBe('meeting');
    expect(slug('Café notes (Q4)!')).toBe('cafe_notes_q4');
    expect(slug('  Résumé -- 2024 ')).toBe('resume_2024');
    expect(slug('日本')).toBe('');
    expect(slug('a — b • c')).toBe('a_b_c');
  });

  it('checks typed folder names and note titles', () => {
    expect(nameError('Client Work')).toBe('');
    expect(nameError('  ')).toBe(nameErrors.empty);
    expect(nameError('!!')).toBe(nameErrors.noLetters);
    expect(titleError('日本')).toBe('');
    expect(titleError(' ')).toBe(nameErrors.title);
  });

  it('cleans typed paths', () => {
    expect(cleanPath(' /Work / Q4 Plans/ ')).toBe('work/q4_plans');
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
    expect(pathError('work/!!')).toBe(nameErrors.noLetters);
    expect(pathError('work//x')).toBe(nameErrors.empty);
  });
});
