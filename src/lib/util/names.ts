// The name rule for folders and notes (same as src-tauri/src/names): free text that is safe as
// one file or folder name everywhere, unique in its folder ignoring case.

const MAX_CHARS = 200;
const FORBIDDEN = /[/\\:\p{Cc}]/u;

export const nameErrors = {
  empty: 'Name can’t be empty',
  spaces: 'No spaces at the start or end',
  chars: 'No / \\ or :, and no dot at the start',
  long: 'Keep names under 200 characters',
  clash: 'Something with this name already exists here',
} as const;

/** Case folding as the database and macOS do it: ASCII letters only. */
const fold = (name: string) => name.replace(/[A-Z]/g, (c) => c.toLowerCase());

/** The message to show under a name field, or '' when the name is fine. */
export function nameError(name: string, siblings: Iterable<string> = []): string {
  if (!name) return nameErrors.empty;
  if (name.trim() !== name) return nameErrors.spaces;
  if (FORBIDDEN.test(name) || name.startsWith('.')) return nameErrors.chars;
  if ([...name].length > MAX_CHARS) return nameErrors.long;
  const folded = fold(name);
  for (const sibling of siblings) {
    if (fold(sibling) === folded) return nameErrors.clash;
  }
  return '';
}

/** The first problem with any segment of a folder path, or '' ('' is the root). */
export function pathError(path: string): string {
  if (path === '') return '';
  for (const segment of path.split('/')) {
    const problem = nameError(segment);
    if (problem) return problem;
  }
  return '';
}

export const join = (folder: string, name: string) => (folder ? `${folder}/${name}` : name);
export const parentOf = (path: string) => (path.includes('/') ? path.slice(0, path.lastIndexOf('/')) : '');
export const baseOf = (path: string) => path.slice(path.lastIndexOf('/') + 1);
/** `path` is `root` or inside it. '' contains everything. */
export const isUnder = (path: string, root: string) => root === '' || path === root || path.startsWith(root + '/');
/** `path` relative to `root`, '' when they are the same. */
export function relativeTo(path: string, root: string): string {
  if (path === root) return '';
  if (root === '') return path;
  return path.slice(root.length + 1);
}

/** A typed folder path without surrounding spaces or trailing slashes. */
export const cleanPath = (value: string) => value.trim().replace(/\/+$/, '');
