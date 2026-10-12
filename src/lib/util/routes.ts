// URLs for every page, so links are built in one place.
import type { LibraryArea } from '#lib/enums/library.enum.ts';

/** `?f=<folder>` for a folder view; nothing for the root. */
function folderQuery(path: string): string {
  if (!path) return '';
  return `?f=${encodeURIComponent(path)}`;
}

export const routes = {
  home: '/',
  start: '/start',
  notes: '/notes',
  tasks: '/tasks',
  people: '/people',
  settings: '/settings',
  /** Opens the Library on the area used last. */
  library: '/library',
  libraryArea: (area: LibraryArea) => `/library/${area}`,
  /** A Settings section, such as `sync`. */
  settingsSection: (id: string) => `/settings?s=${id}`,
  notesFolder: (path: string) => `/notes${folderQuery(path)}`,
  note: (id: string, { edit = false }: { edit?: boolean } = {}) => `/notes/${id}${edit ? '?edit=1' : ''}`,
  tasksFolder: (path: string) => `/tasks${folderQuery(path)}`,
  task: (id: string) => `/tasks/${id}`,
  /** `isNew` opens it ready to type, with the first name focused. */
  person: (id: string, { isNew = false }: { isNew?: boolean } = {}) => `/people/${id}${isNew ? '?new=1' : ''}`,
  /** The People page with the add field open. */
  newPerson: '/people?add=1',
};
