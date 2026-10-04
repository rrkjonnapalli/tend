// Tasks, task folders, statuses and today's My day choices, loaded once and patched in place.
import { tasksAPI } from '#lib/api/tasks.ts';
import type { NewTask, Task, TaskPatch } from '#lib/types/task.type.ts';
import { statusesAPI } from '#lib/api/statuses.ts';
import { DONE_STATUS } from '#lib/util/statuses.ts';
import type { Status } from '#lib/types/status.type.ts';
import { today } from '#lib/util/dates.ts';
import { isUnder } from '#lib/util/names.ts';

class TasksStore {
  folders = $state<string[]>([]);
  tasks = $state<Task[]>([]);
  statuses = $state<Status[]>([]);
  myDay = $state(new Map<string, boolean>());
  /** The day My day was loaded for, so a new day reloads it. */
  day = $state(today());

  load = async () => {
    const day = today();
    const [data, statuses] = await Promise.all([tasksAPI.data(day), statusesAPI.list()]);
    this.folders = data.folders;
    this.tasks = data.tasks;
    this.myDay = new Map(data.myDay);
    this.statuses = statuses;
    this.day = day;
  };

  loadStatuses = async () => {
    this.statuses = await statusesAPI.list();
  };

  byId = (id: string) => this.tasks.find((t) => t.id === id);
  subtasksOf = (id: string) => this.tasks.filter((t) => t.parentId === id).sort((a, b) => (a.orderKey < b.orderKey ? -1 : 1));
  status = (value: string) => this.statuses.find((s) => s.value === value);
  topLevelUnder = (folder: string) => this.tasks.filter((t) => t.parentId === null && isUnder(t.path, folder));

  /** "1/3" done subtasks, or '' when there are none. */
  subtaskCount = (id: string) => {
    const subs = this.subtasksOf(id);
    return subs.length ? `${subs.filter((s) => s.status === DONE_STATUS).length}/${subs.length}` : '';
  };

  #replace = (task: Task) => {
    const i = this.tasks.findIndex((t) => t.id === task.id);
    if (i >= 0) this.tasks[i] = task;
    else this.tasks.push(task);
  };

  create = async (task: NewTask) => {
    const created = await tasksAPI.create(task);
    await this.load();
    return created;
  };

  update = async (id: string, patch: TaskPatch) => {
    const task = await tasksAPI.update({ id, patch });
    this.#replace(task);
    return task;
  };

  move = async (id: string, target: string) => {
    await tasksAPI.move({ id, target });
    await this.load();
  };

  remove = async (id: string) => {
    await tasksAPI.remove(id);
    await this.load();
  };

  setMyDay = async (id: string, included: boolean | null) => {
    await tasksAPI.setMyDay({ id, day: this.day, included });
    const next = new Map(this.myDay);
    if (included === null) next.delete(id);
    else next.set(id, included);
    this.myDay = next;
  };

  createFolder = async (path: string) => {
    await tasksAPI.createFolder(path);
    await this.load();
  };

  renameFolder = async (path: string, name: string) => {
    const next = await tasksAPI.renameFolder({ path, name });
    await this.load();
    return next;
  };

  moveFolder = async (path: string, target: string) => {
    const next = await tasksAPI.moveFolder({ path, target });
    await this.load();
    return next;
  };

  removeFolder = async (path: string) => {
    await tasksAPI.removeFolder(path);
    await this.load();
  };
}

export const tasks = new TasksStore();
