/** An item as cut and paste sees it: a task with its parent, or a note (never a parent). */
export type TreeItem = { id: string; parentId: string | null };
