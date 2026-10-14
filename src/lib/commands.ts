// The command registry. One list drives the global shortcuts, the ⌘K palette and
// Settings → Command palette.
import { goto } from '$app/navigation';
import { page } from '$app/state';
import { ui } from '#lib/stores/ui.svelte.ts';
import { sync } from '#lib/stores/sync.svelte.ts';
import { settings } from '#lib/stores/settings.svelte.ts';
import { snaps } from '#lib/stores/snaps.svelte.ts';
import { pageActions } from '#lib/stores/page-actions.svelte.ts';
import { routes } from '#lib/util/routes.ts';
import type { Command } from '#lib/types/command.type.ts';
import type { PageAction, QuickKind } from '#lib/types/ui.type.ts';
import type { ThemePref } from '#lib/types/theme.type.ts';
import { systemIsDark, toggledTheme } from '#lib/util/theme.ts';
import { settingDefaults, settingKeys } from '#lib/config/settings.config.ts';

const onPage = (id: PageAction) => ({ when: () => pageActions.has(id), run: () => pageActions.run(id) });
const onTasksList = () => page.url.pathname === routes.tasks;

/** Notes and Tasks are the sections with an explorer. */
function hasExplorer(): boolean {
  const path = page.url.pathname;
  const inSection = (base: string) => path === base || path.startsWith(`${base}/`);
  return inSection(routes.notes) || inSection(routes.tasks);
}

const openQuick = (kind: QuickKind) => () => {
  ui.quick = kind;
};

/** ⌘S: a blank snap; pressed again while the snap editor is open, its next paper. */
function writeSnap() {
  if (pageActions.has('snap.paper')) pageActions.run('snap.paper');
  else snaps.write();
}

/** Light to dark and back, from whichever is showing; "system" becomes a fixed choice. */
function toggleTheme() {
  const pref = settings.get<ThemePref>(settingKeys.theme, settingDefaults.theme);
  settings.set(settingKeys.theme, toggledTheme(pref, systemIsDark()));
}

function openPalette() {
  ui.palette = true;
}

/** ⌘[: the page's own Back (it knows where to go when there is no history), else history. */
function goBack() {
  const back = document.querySelector<HTMLElement>('.pbar .back');
  if (back) back.click();
  else history.back();
}

function toggleTaskLayout() {
  const isBoard = settings.get<string>(settingKeys.tasksLayout, settingDefaults.tasksLayout) === 'board';
  settings.set(settingKeys.tasksLayout, isBoard ? 'list' : 'board');
}

export const commands: Command[] = [
  { id: 'app.theme', group: 'App', label: 'Toggle light / dark theme', pinned: true, run: toggleTheme },
  { id: 'go.notes', group: 'Go to', label: 'Go to Notes', keys: 'mod+1', run: () => goto(routes.notes) },
  { id: 'go.tasks', group: 'Go to', label: 'Go to Tasks', keys: 'mod+2', run: () => goto(routes.tasks) },
  { id: 'go.people', group: 'Go to', label: 'Go to People', keys: 'mod+3', run: () => goto(routes.people) },
  { id: 'go.library', group: 'Go to', label: 'Go to Library', keys: 'mod+4', run: () => goto(routes.library) },
  { id: 'go.settings', group: 'Go to', label: 'Open Settings', keys: 'mod+,', run: () => goto(routes.settings) },
  { id: 'go.back', group: 'Go to', label: 'Back', keys: 'mod+[', run: goBack },
  { id: 'go.forward', group: 'Go to', label: 'Forward', keys: 'mod+]', run: () => history.forward() },

  { id: 'new.note', group: 'Create', label: 'New note', keys: 'mod+n', example: 'note: Meeting notes', run: openQuick('note') },
  { id: 'new.task', group: 'Create', label: 'New task', keys: 'mod+shift+t', example: 'task: Write README', run: openQuick('task') },
  { id: 'new.folder', group: 'Create', label: 'New folder', keys: 'mod+shift+n', run: openQuick('folder') },
  { id: 'new.timer', group: 'Create', label: 'New timer', keys: 'mod+t', example: 'timer 10m turn off the motor', run: openQuick('timer') },
  { id: 'new.snap', group: 'Create', label: 'New snap', keys: 'mod+s', run: writeSnap },
  { id: 'new.person', group: 'Create', label: 'New person', example: 'person: Ravi Kumar', run: () => goto(routes.newPerson) },

  { id: 'note.mode', group: 'Notes', label: 'Toggle read / edit', keys: 'mod+r', ...onPage('note.mode') },
  { id: 'note.tag', group: 'Notes', label: 'Add tag to note', ...onPage('note.tag') },
  { id: 'page.width', group: 'Notes', label: 'Toggle content width', ...onPage('page.width') },

  { id: 'task.status', group: 'Tasks', label: 'Set status…', ...onPage('task.status') },
  { id: 'task.due', group: 'Tasks', label: 'Set due date…', ...onPage('task.due') },
  { id: 'task.myday', group: 'Tasks', label: 'Add to My day', ...onPage('task.myday') },
  { id: 'task.layout', group: 'Tasks', label: 'Switch list / board', keys: 'mod+r', when: onTasksList, run: toggleTaskLayout },

  { id: 'app.palette', group: 'App', label: 'Open command palette', keys: 'mod+k', run: openPalette },
  { id: 'app.sync', group: 'App', label: 'Sync now', keys: 'mod+shift+arrowdown', when: () => sync.state !== 'off' && sync.state !== 'running' && !sync.outdated, run: sync.now },
  { id: 'page.search', group: 'App', label: 'Search this list', keys: 'mod+l', ...onPage('page.search') },
  { id: 'app.explorer', group: 'App', label: 'Toggle explorer', keys: 'mod+e', when: hasExplorer, run: ui.toggleExplorer },
  { id: 'app.nav', group: 'App', label: 'Toggle dock (bottom bar)', keys: 'mod+d', run: ui.toggleNav },
  { id: 'app.feedback', group: 'App', label: 'Send feedback', run: () => ui.show('feedback') },
  { id: 'app.vault', group: 'App', label: 'Change vault…', run: () => goto(routes.start) },
];

export const available = (c: Command) => !c.when || c.when();

/** Webview defaults that would reload the page or open windows. */
export const blocked = ['mod+r', 'mod+shift+r', 'mod+n', 'mod+t', 'mod+shift+n', 'mod+shift+t', 'mod+p'];
