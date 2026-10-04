// The command registry. One list drives the global shortcuts, the ⌘K palette and
// Settings → Command palette.
import { goto } from '$app/navigation';
import { page } from '$app/state';
import { ui } from './stores/ui.svelte.ts';
import { settings } from './stores/settings.svelte.ts';
import { pageActions, type PageAction } from './stores/page-actions.svelte.ts';

export type CommandGroup = 'Go to' | 'Create' | 'Notes' | 'Tasks' | 'App';

export type Command = {
  id: string;
  group: CommandGroup;
  label: string;
  /** Chord such as "mod+shift+t" (see util/keys). */
  keys?: string;
  /** What to type in ⌘K to reach it. */
  example?: string;
  /** False when the command doesn't apply to the open page. */
  when?: () => boolean;
  run: () => void;
};

const onPage = (id: PageAction) => ({ when: () => pageActions.has(id), run: () => pageActions.run(id) });
const onTasksList = () => page.url.pathname === '/tasks';
/** Notes and Tasks are the sections with an explorer. */
const hasExplorer = () => /^\/(notes|tasks)(\/|$)/.test(page.url.pathname);

export const commands: Command[] = [
  { id: 'go.notes', group: 'Go to', label: 'Go to Notes', keys: 'mod+1', run: () => goto('/notes') },
  { id: 'go.tasks', group: 'Go to', label: 'Go to Tasks', keys: 'mod+2', run: () => goto('/tasks') },
  { id: 'go.people', group: 'Go to', label: 'Go to People', keys: 'mod+3', run: () => goto('/people') },
  { id: 'go.settings', group: 'Go to', label: 'Open Settings', keys: 'mod+,', run: () => goto('/settings') },

  { id: 'new.note', group: 'Create', label: 'New note', keys: 'mod+n', example: 'note: Meeting notes', run: () => (ui.quick = 'note') },
  { id: 'new.task', group: 'Create', label: 'New task', keys: 'mod+shift+t', example: 'task: Write README', run: () => (ui.quick = 'task') },
  { id: 'new.folder', group: 'Create', label: 'New folder', keys: 'mod+shift+n', run: () => (ui.quick = 'folder') },
  { id: 'new.timer', group: 'Create', label: 'New timer', keys: 'mod+t', example: 'timer 10m turn off the motor', run: () => (ui.quick = 'timer') },
  { id: 'new.person', group: 'Create', label: 'New person', example: 'person: Ravi Kumar', run: () => goto('/people?add=1') },

  { id: 'note.mode', group: 'Notes', label: 'Toggle read / edit', keys: 'mod+e', ...onPage('note.mode') },
  { id: 'note.tag', group: 'Notes', label: 'Add tag to note', ...onPage('note.tag') },
  { id: 'page.width', group: 'Notes', label: 'Toggle content width', ...onPage('page.width') },

  { id: 'task.status', group: 'Tasks', label: 'Set status…', ...onPage('task.status') },
  { id: 'task.due', group: 'Tasks', label: 'Set due date…', ...onPage('task.due') },
  { id: 'task.myday', group: 'Tasks', label: 'Add to My day', ...onPage('task.myday') },
  {
    id: 'task.layout',
    group: 'Tasks',
    label: 'Switch list / board',
    when: onTasksList,
    run: () => settings.set('tasks.layout', settings.get<string>('tasks.layout', 'list') === 'board' ? 'list' : 'board'),
  },

  { id: 'app.palette', group: 'App', label: 'Open command palette', keys: 'mod+k', run: () => (ui.palette = true) },
  { id: 'app.explorer', group: 'App', label: 'Toggle explorer', keys: 'mod+b', when: hasExplorer, run: ui.toggleExplorer },
  { id: 'app.nav', group: 'App', label: 'Toggle bottom bar', run: ui.toggleNav },
  { id: 'app.feedback', group: 'App', label: 'Send feedback', run: () => ui.show('feedback') },
  { id: 'app.vault', group: 'App', label: 'Change vault…', run: () => goto('/start') },
];

export const available = (c: Command) => !c.when || c.when();

/** Webview defaults that would reload the page or open windows. */
export const blocked = ['mod+r', 'mod+shift+r', 'mod+n', 'mod+t', 'mod+shift+n', 'mod+shift+t', 'mod+p'];
