import { describe, expect, it } from 'vitest';
import { buildRows, canDrop, siblingNames, withAncestors } from './explorer-rows.ts';

const tree = {
  folders: ['work', 'work/tauri', 'home'],
  leaves: [
    { id: 'b', folder: 'work', name: 'plan' },
    { id: 'a', folder: 'work', name: 'ideas' },
    { id: 'c', folder: '', name: 'inbox' },
  ],
};

describe('buildRows', () => {
  it('lists open folders depth first, folders before leaves', () => {
    const rows = buildRows({ tree, rootName: 'notes', expanded: { '': true, work: true }, adding: null });
    expect(rows.map((r) => `${r.depth}:${r.name}`)).toEqual(['0:notes', '1:home', '1:work', '2:tauri', '2:ideas', '2:plan', '1:inbox']);
  });

  it('puts a new folder row first and a new leaf row after the subfolders', () => {
    const folderAdd = buildRows({ tree, rootName: 'notes', expanded: { '': true }, adding: { kind: 'folder', folder: '' } });
    expect(folderAdd.map((r) => r.key)).toEqual(['f:', 'new', 'f:home', 'f:work', 'l:c']);
    const leafAdd = buildRows({ tree, rootName: 'notes', expanded: { '': true }, adding: { kind: 'leaf', folder: '' } });
    expect(leafAdd.map((r) => r.key)).toEqual(['f:', 'f:home', 'f:work', 'new', 'l:c']);
  });
});

describe('siblingNames', () => {
  it('lists the subfolders of a folder, minus the excepted row', () => {
    expect(siblingNames({ tree, folder: 'work' })).toEqual(['tauri']);
    expect(siblingNames({ tree, folder: 'work', exceptKey: 'f:work/tauri' })).toEqual([]);
  });
});

describe('canDrop', () => {
  const rows = buildRows({ tree, rootName: 'notes', expanded: { '': true, work: true }, adding: null });
  const row = (name: string) => rows.find((r) => r.name === name)!;

  it('refuses the current folder, the folder itself and its insides', () => {
    expect(canDrop({ tree, row: row('work'), target: '' })).toBe(false);
    expect(canDrop({ tree, row: row('work'), target: 'work/tauri' })).toBe(false);
    expect(canDrop({ tree, row: row('plan'), target: 'work' })).toBe(false);
  });

  it('allows a free name elsewhere', () => {
    expect(canDrop({ tree, row: row('work'), target: 'home' })).toBe(true);
    expect(canDrop({ tree, row: row('plan'), target: '' })).toBe(true);
  });
});

describe('withAncestors', () => {
  it('walks up to the root', () => {
    expect(withAncestors('a/b/c')).toEqual(['a/b/c', 'a/b', 'a', '']);
    expect(withAncestors('')).toEqual(['']);
  });
});
