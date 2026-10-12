// Every key the app stores in the vault's settings table, in one place.
import type { DeleteKind, QuickKind, Tree } from '#lib/types/ui.type.ts';
import { LIBRARY_AREA } from '#lib/enums/library.enum.ts';

export const settingKeys = {
  navOpen: 'ui.nav_open',
  explorerOpen: 'ui.explorer_open',
  notesWide: 'notes.wide',
  tasksWide: 'tasks.wide',
  tasksLayout: 'tasks.layout',
  /** `{ view, day }`: the task list view picked last and when (see `defaultView`). */
  tasksLastView: 'tasks.last_view',
  tasksRootName: 'tasks.root_name',
  /** The task list's "hide done" toggle. */
  tasksHideDone: 'tasks.hide_done',
  timersSound: 'timers.sound',
  /** The Library area used last (`LibraryArea`); kept on this device. */
  libraryArea: 'ui.library.area',
  /** Light, dark or the OS's (`ThemePref`); kept on this device. */
  theme: 'ui.theme',
  /** The file size limit in MB (1–90), shared through sync: bigger notes aren't indexed, bigger
   *  files can't be attached, and sync never commits one. Checked again in Rust. */
  fileLimitMB: 'files.max_mb',
  /** The folder Quick create last used for each kind; folders keep one per tree. */
  /** Whether deleting a note, task, person, word, snap or reference asks first; kept on this device. */
  confirmDelete: (kind: DeleteKind) => `confirm.delete.${kind}`,
  quickLastPath: (kind: QuickKind, tree?: Tree) =>
    kind === 'folder' && tree ? `quick.last_path.folder.${tree}` : `quick.last_path.${kind}`,
};

/** The value each setting has until it is first changed. */
export const settingDefaults = {
  navOpen: true,
  explorerOpen: true,
  notesWide: false,
  tasksWide: false,
  tasksLayout: 'list',
  tasksRootName: 'tasks',
  tasksHideDone: false,
  timersSound: true,
  libraryArea: LIBRARY_AREA.words,
  theme: 'system',
  fileLimitMB: 30,
  confirmDelete: true,
} as const;

/** The range the file size limit may take (src-tauri/src/files.rs `LIMIT_MB_RANGE`). */
export const fileLimitRange = { min: 1, max: 90 } as const;

/** The choices for the theme setting, in Settings → General. */
export const themeChoices = [
  { value: 'system', label: 'System' },
  { value: 'light', label: 'Light' },
  { value: 'dark', label: 'Dark' },
] as const;

/** The delete confirmations that can be turned off, in Settings → General. Folders, stacks,
 *  statuses, labels and word kinds always ask. */
export const deleteChoices = [
  { kind: 'note', label: 'Ask before deleting a note' },
  { kind: 'task', label: 'Ask before deleting a task' },
  { kind: 'person', label: 'Ask before deleting a person' },
  { kind: 'word', label: 'Ask before deleting a word' },
  { kind: 'snap', label: 'Ask before deleting a snap' },
  { kind: 'reference', label: 'Ask before deleting a reference' },
] as const;
