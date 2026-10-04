// Tasks, task folders, statuses, labels and today's My day choices, loaded once and patched in
// place.
import { tasksAPI } from '#lib/api/tasks.ts';
import type { NewTask, Task, TaskPatch } from '#lib/types/task.type.ts';
import { statusesAPI } from '#lib/api/statuses.ts';
import type { Status } from '#lib/types/status.type.ts';
import { labelsAPI } from '#lib/api/labels.ts';
import type { Label } from '#lib/types/label.type.ts';
import { today } from '#lib/util/dates.ts';
import { isUnder } from '#lib/util/names.ts';
import { byOrderKey, isDone } from '#lib/util/task-views.ts';

/** (task id, label id) pairs → task id → label ids. */
function groupLinks(links: [string, string][]): Map<string, string[]> {
  const byTask = new Map<string, string[]>();
  for (const [taskId, labelId] of links) {
    const ids = byTask.get(taskId) ?? [];
    ids.push(labelId);
    byTask.set(taskId, ids);
  }
  return byTask;
}

class TasksStore {
  folders = $state<string[]>([]);
  tasks = $state<Task[]>([]);
  statuses = $state<Status[]>([]);
  labels = $state<Label[]>([]);
  /** Task id → its label ids. */
  labelIds = $state(new Map<string, string[]>());
  myDay = $state(new Map<string, boolean>());
  /** The day My day was loaded for, so a new day reloads it. */
  day = $state(today());

  load = async () => {
    const day = today();
    const [data, statuses, labels] = await Promise.all([tasksAPI.data(day), statusesAPI.list(), labelsAPI.list()]);
    this.folders = data.folders;
    this.tasks = data.tasks;
    this.myDay = new Map(data.myDay);
    this.labelIds = groupLinks(data.labels);
    this.statuses = statuses;
    this.labels = labels;
    this.day = day;
  };

  loadStatuses = async () => {
    this.statuses = await statusesAPI.list();
  };

  loadLabels = async () => {
    this.labels = await labelsAPI.list();
  };

  /** The task's labels, in the list's order. */
  labelsOf = (taskId: string): Label[] => {
    const ids = new Set(this.labelIds.get(taskId) ?? []);
    return this.labels.filter((l) => ids.has(l.id));
  };

  setLabels = async (taskId: string, ids: string[]) => {
    const saved = await labelsAPI.setForTask({ id: taskId, labels: ids });
    const next = new Map(this.labelIds);
    next.set(taskId, saved);
    this.labelIds = next;
    return saved;
  };

  byId = (id: string) => this.tasks.find((t) => t.id === id);
  subtasksOf = (id: string) => this.tasks.filter((t) => t.parentId === id).sort(byOrderKey);
  status = (value: string) => this.statuses.find((s) => s.value === value);
  topLevelUnder = (folder: string) => this.tasks.filter((t) => t.parentId === null && isUnder(t.path, folder));

  /** "1/3" done subtasks, or '' when there are none. */
  subtaskCount = (id: string) => {
    const subs = this.subtasksOf(id);
    if (!subs.length) return '';
    const done = subs.filter(isDone).length;
    return `${done}/${subs.length}`;
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

  move = async ({ id, target }: { id: string; target: string }) => {
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

  renameFolder = async ({ path, name }: { path: string; name: string }) => {
    const next = await tasksAPI.renameFolder({ path, name });
    await this.load();
    return next;
  };

  moveFolder = async ({ path, target }: { path: string; target: string }) => {
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
