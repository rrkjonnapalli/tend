// The command registry. One list drives the global shortcuts, the ⌘K palette and Settings → Command palette.
import { goto } from '$app/navigation';
import { ui } from './stores/ui.svelte.ts';

export type CommandGroup = 'Go to' | 'Create' | 'Commands';

export type Command = {
  id: string;
  group: CommandGroup;
  label: string;
  /** Chord such as "mod+shift+t" (see util/keys). */
  keys?: string;
  /** What to type in ⌘K to reach it. */
  example?: string;
  run: () => void;
};

export const commands: Command[] = [
  { id: 'go.notes', group: 'Go to', label: 'Notes', keys: 'mod+1', run: () => goto('/notes') },
  { id: 'go.tasks', group: 'Go to', label: 'Tasks', keys: 'mod+2', run: () => goto('/tasks') },
  { id: 'go.people', group: 'Go to', label: 'People', keys: 'mod+3', run: () => goto('/people') },
  { id: 'go.settings', group: 'Go to', label: 'Settings', keys: 'mod+,', run: () => goto('/settings') },
  { id: 'ui.explorer', group: 'Commands', label: 'Toggle explorer', keys: 'mod+\\', run: ui.toggleExplorer },
  { id: 'ui.nav', group: 'Commands', label: 'Toggle sections bar', run: ui.toggleNav },
];

/** Webview defaults that would reload the page or open windows. */
export const blocked = ['mod+r', 'mod+shift+r', 'mod+n', 'mod+t', 'mod+shift+n', 'mod+shift+t', 'mod+p'];
