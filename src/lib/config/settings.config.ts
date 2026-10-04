// Every key the app stores in the vault's settings table, in one place.
import type { QuickKind } from '#lib/types/ui.type.ts';

export const settingKeys = {
  navOpen: 'ui.nav_open',
  explorerOpen: 'ui.explorer_open',
  notesWide: 'notes.wide',
  tasksWide: 'tasks.wide',
  tasksLayout: 'tasks.layout',
  tasksRootName: 'tasks.root_name',
  timersSound: 'timers.sound',
  /** The folder Quick create last used for each kind. */
  quickLastPath: (kind: QuickKind) => `quick.last_path.${kind}`,
};
