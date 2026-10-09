// Which tasks each view of the task list shows, and in what order.
import type { DayRange } from '#lib/types/date.type.ts';
import type { Task } from '#lib/types/task.type.ts';
import type { LastView, MyDayContext, View, ViewContext, ViewScope } from '#lib/types/task-view.type.ts';
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

// Overdue never shows Done tasks; every other view does, unless the list hides them.
const filters: Record<View, (t: Task, c: ViewContext) => boolean> = {
  all: (t) => t.parentId === null,
  overdue: (t, c) => isOverdue(t, c.today),
  myday: (t, c) => inMyDay(t, c),
  upcoming: (t, c) => !!t.dueDate && t.dueDate >= c.today,
  range: (t, c) => !!t.dueDate && inRange(t.dueDate, c.range),
};

const inView = (view: View) => (t: Task, c: ViewContext) => !(c.hideDone && isDone(t)) && filters[view](t, c);

/** The view to open without one in the URL: My day on the first visit of a day, after that
 * whichever view was picked last today. */
export function defaultView({ last, today }: { last: LastView | null; today: string }): View {
  if (!last || last.day !== today) return 'myday';
  return last.view;
}

/** All and My day follow the manual order; the date views sort by due date. Done tasks sink. */
export const isManual = (v: View) => v === 'all' || v === 'myday';

/** Sorts tasks by their order key (plain string order, not locale order), then by id, so tied keys
 *  sort the same way on every device. */
export function byOrderKey(a: Task, b: Task): number {
  if (a.orderKey !== b.orderKey) return a.orderKey < b.orderKey ? -1 : 1;
  if (a.id === b.id) return 0;
  return a.id < b.id ? -1 : 1;
}

const doneLast = (a: Task, b: Task) => Number(isDone(a)) - Number(isDone(b));
const byDueDate = (a: Task, b: Task) => (a.dueDate ?? '').localeCompare(b.dueDate ?? '');
const manualOrder = (a: Task, b: Task) => doneLast(a, b) || byOrderKey(a, b);
const dueOrder = (a: Task, b: Task) => doneLast(a, b) || byDueDate(a, b) || byOrderKey(a, b);

export function viewTasks({ tasks, folder, view, ctx }: ViewScope & { view: View }): Task[] {
  const shown = tasks.filter((t) => isUnder(t.path, folder) && inView(view)(t, ctx));
  return shown.sort(isManual(view) ? manualOrder : dueOrder);
}

export function viewCounts({ tasks, folder, ctx }: ViewScope): Record<View, number> {
  const scoped = tasks.filter((t) => isUnder(t.path, folder));
  const count = (v: View) => scoped.filter((t) => inView(v)(t, ctx)).length;
  return { all: count('all'), overdue: count('overdue'), myday: count('myday'), upcoming: count('upcoming'), range: count('range') };
}
