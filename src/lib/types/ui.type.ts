export type Popup = 'feedback' | 'timers' | null;

/** What a delete confirmation can be turned off for. */
export type DeleteKind = 'note' | 'task' | 'person' | 'word' | 'snap' | 'reference';

export type QuickKind = 'note' | 'task' | 'folder' | 'timer';

/** The two folder trees: notes and tasks. */
export type Tree = 'notes' | 'tasks';

export type PageAction = 'note.mode' | 'note.tag' | 'page.width' | 'page.search' | 'task.status' | 'task.due' | 'task.myday';
