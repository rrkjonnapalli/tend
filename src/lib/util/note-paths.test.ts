import { describe, expect, it } from 'vitest';
import { folderContents, folderCrumbs, folderSummary } from './note-paths.ts';

describe('folderCrumbs', () => {
  it('links the root and every folder segment', () => {
    expect(folderCrumbs('')).toEqual([{ label: 'notes', href: '/notes' }]);
    expect(folderCrumbs('work/tauri')).toEqual([
      { label: 'notes', href: '/notes' },
      { label: 'work', href: '/notes?f=work' },
      { label: 'tauri', href: '/notes?f=work%2Ftauri' },
    ]);
  });
});

describe('folder counts', () => {
  it('summarises notes and subfolders', () => {
    expect(folderSummary({ notes: 3, folders: 0 })).toBe('3 notes');
    expect(folderSummary({ notes: 1, folders: 2 })).toBe('1 note · 2 folders');
  });

  it('describes what a folder delete removes', () => {
    expect(folderContents({ notes: 0, folders: 0 })).toBe('The folder is empty.');
    expect(folderContents({ notes: 2, folders: 1 })).toBe('It contains 2 notes and 1 folder. Everything inside goes to the Trash.');
    expect(folderContents({ notes: 0, folders: 1 })).toBe('It contains 0 notes and 1 folder. Everything inside goes to the Trash.');
  });
});
