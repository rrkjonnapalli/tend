// Every key the app stores in the vault's settings table, in one place.
import type { QuickKind, Tree } from '#lib/types/ui.type.ts';

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
  /** Light, dark or the OS's (`ThemePref`); kept on this device. */
  theme: 'ui.theme',
  /** The file size limit in MB (1–90), shared through sync: bigger notes aren't indexed, bigger
   *  files can't be attached, and sync never commits one. Checked again in Rust. */
  fileLimitMB: 'files.max_mb',
  /** The folder Quick create last used for each kind; folders keep one per tree. */
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
  theme: 'system',
  fileLimitMB: 30,
} as const;

/** The range the file size limit may take (src-tauri/src/files.rs `LIMIT_MB_RANGE`). */
export const fileLimitRange = { min: 1, max: 90 } as const;

/** The choices for the theme setting, in Settings → General. */
export const themeChoices = [
  { value: 'system', label: 'System' },
  { value: 'light', label: 'Light' },
  { value: 'dark', label: 'Dark' },
] as const;
