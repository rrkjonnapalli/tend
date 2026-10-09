export type Task = {
  id: string;
  path: string;
  parentId: string | null;
  title: string;
  status: string;
  dueDate: string | null;
  dueTime: string | null;
  assigneeId: string | null;
  orderKey: string;
  createdAt: number;
  completedAt: number | null;
};

/** `labels` lists every (task id, label id) pair. */
export type TaskData = { folders: string[]; tasks: Task[]; myDay: [string, boolean][]; labels: [string, string][] };
export type TaskDetail = Task & { description: string; tags: string[]; labels: string[] };
export type NewTask = { path: string; title: string; parentId?: string; status?: string; orderKey?: string; dueDate?: string };

/** `null` clears an optional field; a missing field is left alone. */
export type TaskPatch = Partial<{
  title: string;
  status: string;
  description: string;
  dueDate: string | null;
  dueTime: string | null;
  assigneeId: string | null;
  orderKey: string;
}>;
