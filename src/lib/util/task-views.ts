// Which tasks each view of the task list shows, and in what order.
import type { Task } from '#lib/api/tasks.ts';
import { DONE_STATUS } from '#lib/api/statuses.ts';
import { isUnder } from './names.ts';

export type View = 'all' | 'overdue' | 'myday' | 'upcoming' | 'range';

export const isDone = (t: Task) => t.status === DONE_STATUS;
export const isOverdue = (t: Task, today: string) => !isDone(t) && !!t.dueDate && t.dueDate < today;

/** In My day: added by hand, or due today and not removed by hand. */
export function inMyDay(t: Task, today: string, manual: Map<string, boolean>): boolean {
  const m = manual.get(t.id);
  return m ?? t.dueDate === today;
}

export type ViewContext = { today: string; myDay: Map<string, boolean>; range: [string, string] };

const filters: Record<View, (t: Task, c: ViewContext) => boolean> = {
  all: (t) => t.parentId === null,
  overdue: (t, c) => isOverdue(t, c.today),
  myday: (t, c) => inMyDay(t, c.today, c.myDay),
  upcoming: (t, c) => !isDone(t) && !!t.dueDate && t.dueDate >= c.today,
  range: (t, c) => !!t.dueDate && t.dueDate >= c.range[0] && t.dueDate <= c.range[1],
};

/** All and My day follow the manual order; the date views sort by due date. Done tasks sink. */
export const isManual = (v: View) => v === 'all' || v === 'myday';

const byKey = (a: Task, b: Task) => (a.orderKey < b.orderKey ? -1 : a.orderKey > b.orderKey ? 1 : 0);

export function viewTasks(tasks: Task[], folder: string, view: View, c: ViewContext): Task[] {
  const shown = tasks.filter((t) => isUnder(t.path, folder) && filters[view](t, c));
  const sinkDone = (a: Task, b: Task) => Number(isDone(a)) - Number(isDone(b));
  if (isManual(view)) return shown.sort((a, b) => sinkDone(a, b) || byKey(a, b));
  return shown.sort((a, b) => sinkDone(a, b) || (a.dueDate ?? '').localeCompare(b.dueDate ?? '') || byKey(a, b));
}

export function viewCounts(tasks: Task[], folder: string, c: ViewContext): Record<View, number> {
  const scoped = tasks.filter((t) => isUnder(t.path, folder));
  const count = (v: View) => scoped.filter((t) => filters[v](t, c)).length;
  return { all: count('all'), overdue: count('overdue'), myday: count('myday'), upcoming: count('upcoming'), range: count('range') };
}
