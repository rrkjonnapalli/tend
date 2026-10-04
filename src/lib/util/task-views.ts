// Which tasks each view of the task list shows, and in what order.
import type { DayRange } from '#lib/types/date.type.ts';
import type { Task } from '#lib/types/task.type.ts';
import type { MyDayContext, View, ViewContext, ViewScope } from '#lib/types/task-view.type.ts';
import { DONE_STATUS } from '#lib/util/statuses.ts';
import { isUnder } from './names.ts';

export const isDone = (t: Task) => t.status === DONE_STATUS;
export const isOverdue = (t: Task, today: string) => !isDone(t) && !!t.dueDate && t.dueDate < today;

/** In My day: added by hand, or due today and not removed by hand. */
export function inMyDay(t: Task, { today, myDay }: MyDayContext): boolean {
  const manual = myDay.get(t.id);
  return manual ?? t.dueDate === today;
}

const inRange = (day: string, [from, to]: DayRange) => day >= from && day <= to;

const filters: Record<View, (t: Task, c: ViewContext) => boolean> = {
  all: (t) => t.parentId === null,
  overdue: (t, c) => isOverdue(t, c.today),
  myday: (t, c) => inMyDay(t, c),
  upcoming: (t, c) => !isDone(t) && !!t.dueDate && t.dueDate >= c.today,
  range: (t, c) => !!t.dueDate && inRange(t.dueDate, c.range),
};

/** All and My day follow the manual order; the date views sort by due date. Done tasks sink. */
export const isManual = (v: View) => v === 'all' || v === 'myday';

/** Sorts tasks by their order key (plain string order, not locale order). */
export function byOrderKey(a: Task, b: Task): number {
  if (a.orderKey < b.orderKey) return -1;
  if (a.orderKey > b.orderKey) return 1;
  return 0;
}

const doneLast = (a: Task, b: Task) => Number(isDone(a)) - Number(isDone(b));
const byDueDate = (a: Task, b: Task) => (a.dueDate ?? '').localeCompare(b.dueDate ?? '');
const manualOrder = (a: Task, b: Task) => doneLast(a, b) || byOrderKey(a, b);
const dueOrder = (a: Task, b: Task) => doneLast(a, b) || byDueDate(a, b) || byOrderKey(a, b);

export function viewTasks({ tasks, folder, view, ctx }: ViewScope & { view: View }): Task[] {
  const shown = tasks.filter((t) => isUnder(t.path, folder) && filters[view](t, ctx));
  return shown.sort(isManual(view) ? manualOrder : dueOrder);
}

export function viewCounts({ tasks, folder, ctx }: ViewScope): Record<View, number> {
  const scoped = tasks.filter((t) => isUnder(t.path, folder));
  const count = (v: View) => scoped.filter((t) => filters[v](t, ctx)).length;
  return { all: count('all'), overdue: count('overdue'), myday: count('myday'), upcoming: count('upcoming'), range: count('range') };
}
