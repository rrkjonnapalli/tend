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
