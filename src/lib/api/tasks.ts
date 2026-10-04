import { call } from './core.ts';

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

export type TaskData = { folders: string[]; tasks: Task[]; myDay: [string, boolean][] };
export type TaskDetail = Task & { description: string; tags: string[] };
export type NewTask = { path: string; title: string; parentId?: string; status?: string; orderKey?: string };
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

export const tasksApi = {
  data: (day: string) => call<TaskData>('tasks_data', { day }),
  get: (id: string) => call<TaskDetail>('task_get', { id }),
  create: (task: NewTask) => call<Task>('task_create', { task }),
  update: (id: string, patch: TaskPatch) => call<Task>('task_update', { id, patch }),
  move: (id: string, target: string) => call<Task>('task_move', { id, target }),
  remove: (id: string) => call<void>('task_delete', { id }),
  setTags: (id: string, tags: string[]) => call<string[]>('task_tags_set', { id, tags }),
  setMyDay: (id: string, day: string, included: boolean | null) => call<void>('my_day_set', { id, day, included }),
  createFolder: (path: string) => call<void>('task_folder_create', { path }),
  renameFolder: (path: string, name: string) => call<string>('task_folder_rename', { path, name }),
  moveFolder: (path: string, target: string) => call<string>('task_folder_move', { path, target }),
  removeFolder: (path: string) => call<void>('task_folder_delete', { path }),
};
