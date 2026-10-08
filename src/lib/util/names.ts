// The name rule for folders and notes (same as src-tauri/src/names): letters, digits, spaces and
// . , _ - only. Leading and trailing spaces and dots are trimmed; a name taken in its folder gets a
// suffix (Plan_1) when saved, so a clash is never an error.

const MAX_CHARS = 200;
const ALLOWED = /^[A-Za-z0-9 .,_-]*$/;

export const nameErrors = {
  empty: 'Name can’t be empty',
  chars: 'Use letters, digits, spaces and . , _ - only',
  long: 'Keep names under 200 characters',
} as const;

/** A typed name without leading or trailing spaces and dots, as it will be saved. */
export const cleanName = (name: string) => name.replace(/^[ .]+|[ .]+$/g, '');

/** The message to show under a name field, or '' when the name is fine. */
export function nameError(typed: string): string {
  const name = cleanName(typed);
  if (!name) return nameErrors.empty;
  if (!ALLOWED.test(name)) return nameErrors.chars;
  if (name.length > MAX_CHARS) return nameErrors.long;
  return '';
}

/** A typed folder path with each segment cleaned and no slashes at the ends. */
export const cleanPath = (value: string) =>
  value
    .trim()
    .replace(/^\/+|\/+$/g, '')
    .split('/')
    .map(cleanName)
    .join('/');

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
