// The explorer tree as a flat list of rows, plus the rules for naming and dropping into it.
import type { ExplorerItem, ExplorerRow, ExplorerTree, Leaf, NewKind } from '#lib/types/explorer.type.ts';
import { baseOf, isUnder, parentOf } from './names.ts';

/** Row key of the root folder (path ''). */
export const ROOT_KEY = 'f:';
/** Row key of the row being named before it is created. */
export const NEW_KEY = 'new';

const INDENT_BASE_PX = 16;
const INDENT_STEP_PX = 16;

export function keyOf(item: ExplorerItem): string {
  if (item.kind === 'folder') return `f:${item.path}`;
  return `l:${item.id}`;
}

/** A folder's path, or undefined for a leaf. */
export function folderPath(item: ExplorerItem): string | undefined {
  if (item.kind === 'folder') return item.path;
  return undefined;
}

export const isRoot = (row: ExplorerRow) => row.item.kind === 'folder' && row.item.path === '';

export const indentOf = (row: ExplorerRow) => INDENT_BASE_PX + row.depth * INDENT_STEP_PX;

/** The folder a drop on this row lands in; undefined for the new-item row. */
export function dropPath(row: ExplorerRow): string | undefined {
  if (row.isNew) return undefined;
  if (row.item.kind === 'folder') return row.item.path;
  return row.folder;
}

/** `path` and every folder above it, up to the root (''). */
export function withAncestors(path: string): string[] {
  const chain = [path];
  let current = path;
  while (current !== '') {
    current = parentOf(current);
    chain.push(current);
  }
  return chain;
}

function folderRow({ path, depth, rootName }: { path: string; depth: number; rootName: string }): ExplorerRow {
  const name = path === '' ? rootName : baseOf(path);
  return { key: `f:${path}`, item: { kind: 'folder', path }, name, depth, folder: parentOf(path), isFolder: true };
}

function leafRow(leaf: Leaf, depth: number): ExplorerRow {
  return { key: `l:${leaf.id}`, item: { kind: 'leaf', id: leaf.id }, name: leaf.name, depth, folder: leaf.folder, isFolder: false };
}

function newRow({ kind, folder, depth }: { kind: NewKind; folder: string; depth: number }): ExplorerRow {
  const isFolder = kind === 'folder';
  const item: ExplorerItem = isFolder ? { kind: 'folder', path: '' } : { kind: 'leaf', id: '' };
  return { key: NEW_KEY, item, name: '', depth, folder, isFolder, isNew: kind };
}

const subfoldersOf = (tree: ExplorerTree, path: string) => tree.folders.filter((f) => parentOf(f) === path).sort();

const leavesIn = (tree: ExplorerTree, path: string) =>
  tree.leaves.filter((l) => l.folder === path).sort((a, b) => a.name.localeCompare(b.name));

type RowsOptions = {
  tree: ExplorerTree;
  rootName: string;
  expanded: Record<string, boolean>;
  /** Where a new row is being named, if anywhere. */
  adding: { kind: NewKind; folder: string } | null;
};

/** Visible rows, depth first: a folder, then its new-folder row, subfolders, new-leaf row and leaves. */
export function buildRows({ tree, rootName, expanded, adding }: RowsOptions): ExplorerRow[] {
  const rows: ExplorerRow[] = [];
  const addsHere = (kind: NewKind, path: string) => adding?.kind === kind && adding.folder === path;

  const walk = (path: string, depth: number) => {
    rows.push(folderRow({ path, depth, rootName }));
    if (!expanded[path]) return;
    const childDepth = depth + 1;
    if (addsHere('folder', path)) rows.push(newRow({ kind: 'folder', folder: path, depth: childDepth }));
    for (const child of subfoldersOf(tree, path)) walk(child, childDepth);
    if (addsHere('leaf', path)) rows.push(newRow({ kind: 'leaf', folder: path, depth: childDepth }));
    for (const leaf of leavesIn(tree, path)) rows.push(leafRow(leaf, childDepth));
  };

  walk('', 0);
  return rows;
}

/** A row can move into `target` unless it is already there or would land inside itself. A name
 *  taken in the target gets a suffix when the move is saved. */
export function canDrop({ row, target }: { row: ExplorerRow; target: string }): boolean {
  if (row.item.kind === 'folder') {
    const path = row.item.path;
    return !isUnder(target, path) && parentOf(path) !== target;
  }
  return row.folder !== target;
}
