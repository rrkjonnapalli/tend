import type { IconName } from '#lib/types/icon.type.ts';

/** An item as cut and paste sees it: a task with its parent, or a note (never a parent). */
export type TreeItem = { id: string; parentId: string | null };

/** The list a phone selection or cut belongs to: notes, tasks, or the snaps board. */
export type ClipTree = 'notes' | 'tasks' | 'snaps';

/** The action bar's middle button: Rename for notes and tasks, Tag for snaps. */
export type ClipAction = { label: string; icon: IconName; disabled?: boolean; onclick: () => void };

/** What was cut: the rows of one list, and the folder they came from (for the paste bar). */
export type Cut = { tree: ClipTree; ids: string[]; from: string };
