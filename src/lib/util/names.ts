// The name rule for folders and note files (same as src-tauri/src/names): lowercase letters and
// digits with single _ between them. Typed folder names and note titles become such names through
// `slug`; a name taken in its folder gets a suffix (plan_1) when saved, so a clash is never an error.

const MAX_CHARS = 200;
const RULE = /^[a-z0-9]+(_[a-z0-9]+)*$/;

export const nameErrors = {
  empty: 'Name can’t be empty',
  chars: 'Use at least one letter or digit',
  title: 'A note needs a title',
} as const;

export const isValidName = (name: string) => RULE.test(name) && name.length <= MAX_CHARS;

/** The name text becomes on disk: accents dropped, lowercase, other runs → _ ("Café notes (Q4)" →
 *  "cafe_notes_q4"). '' when no letter or digit is left. */
export function slug(text: string): string {
  return text
    .normalize('NFD')
    .replace(/\p{M}/gu, '')
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '_')
    .replace(/^_+|_+$/g, '')
    .slice(0, MAX_CHARS)
    .replace(/_+$/, '');
}

/** The message to show under a folder or note name field, or '' when it can be saved. */
export function nameError(typed: string): string {
  if (!typed.trim()) return nameErrors.empty;
  if (!slug(typed)) return nameErrors.chars;
  return '';
}

/** The message to show under a note title field, or ''. */
export const titleError = (typed: string) => (typed.trim() ? '' : nameErrors.title);

/** A typed folder path, each segment as it will be saved. */
export const cleanPath = (value: string) =>
  value
    .trim()
    .replace(/^\/+|\/+$/g, '')
    .split('/')
    .map(slug)
    .join('/');

/** The first problem with any segment of a typed folder path, or '' ('' is the root). */
export function pathError(typed: string): string {
  const path = typed.trim().replace(/^\/+|\/+$/g, '');
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
