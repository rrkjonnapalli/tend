import type { NewTask, Task, TaskData, TaskDetail, TaskPatch } from '#lib/types/task.type.ts';
import { call } from './core.ts';

export const tasksAPI = {
  data: (day: string) => call<TaskData>('tasks_data', { args: { day } }),
  get: (id: string) => call<TaskDetail>('task_get', { args: { id } }),
  create: (task: NewTask) => call<Task>('task_create', { args: { task } }),
  update: (input: { id: string; patch: TaskPatch }) => call<Task>('task_update', { args: input }),
  move: (input: { id: string; target: string }) => call<Task>('task_move', { args: input }),
  remove: (id: string) => call<void>('task_delete', { args: { id } }),
  setTags: (input: { id: string; tags: string[] }) => call<string[]>('task_tags_set', { args: input }),
  /** `included: null` drops the manual choice, so the default (due today) applies again. */
  setMyDay: (input: { id: string; day: string; included: boolean | null }) => call<void>('my_day_set', { args: input }),
  /** Resolves to the created path (a taken name gets a suffix). */
  createFolder: (path: string) => call<string>('task_folder_create', { args: { path } }),
  renameFolder: (input: { path: string; name: string }) => call<string>('task_folder_rename', { args: input }),
  moveFolder: (input: { path: string; target: string }) => call<string>('task_folder_move', { args: input }),
  removeFolder: (path: string) => call<void>('task_folder_delete', { args: { path } }),
};
