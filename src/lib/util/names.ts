// The name rule for folders and note files (same as src-tauri/src/names.rs).

const RULE = /^[a-z0-9]+(_[a-z0-9]+)*$/;

export const nameErrors = {
  empty: 'Name can’t be empty',
  chars: 'Only a–z and 0–9, with single _ between them',
  clash: 'Something with this name already exists here',
  title: 'A note needs a title',
} as const;

export const isValidName = (name: string) => RULE.test(name);

/** The message to show under a name field, or '' when the name is fine. */
export function nameError(name: string, siblings: Iterable<string> = []): string {
  if (!name) return nameErrors.empty;
  if (!isValidName(name)) return nameErrors.chars;
  for (const sibling of siblings) {
    if (sibling === name) return nameErrors.clash;
  }
  return '';
}

/** Every segment of a folder path follows the rule ('' is the root). */
export const isValidPath = (path: string) => path === '' || path.split('/').every(isValidName);

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
