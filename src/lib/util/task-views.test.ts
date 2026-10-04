import { describe, expect, it } from 'vitest';
import type { Task } from '#lib/types/task.type.ts';
import { inMyDay, viewCounts, viewTasks } from './task-views.ts';
import type { ViewContext } from '#lib/types/task-view.type.ts';

const base: Task = {
  id: '',
  path: 'work',
  parentId: null,
  title: '',
  status: 'NOT_STARTED',
  dueDate: null,
  dueTime: null,
  assigneeId: null,
  orderKey: 'V',
  createdAt: 0,
  completedAt: null,
};
const t = (o: Partial<Task>): Task => ({ ...base, ...o, title: o.id ?? '' });

const tasks = [
  t({ id: 'a', orderKey: 'b' }),
  t({ id: 'b', orderKey: 'a', status: 'DONE' }),
  t({ id: 'c', orderKey: 'c', dueDate: '2026-10-01' }),
  t({ id: 'd', orderKey: 'd', dueDate: '2026-10-04' }),
  t({ id: 'e', orderKey: 'e', dueDate: '2026-10-10', parentId: 'a' }),
  t({ id: 'f', orderKey: 'f', path: 'home', dueDate: '2026-10-04' }),
  t({ id: 'g', orderKey: 'g', dueDate: '2026-09-01', status: 'DONE' }),
];
const ctx: ViewContext = { today: '2026-10-04', myDay: new Map(), range: ['2026-10-01', '2026-10-04'] };
const titles = (xs: Task[]) => xs.map((x) => x.id);

describe('views', () => {
  it('All: top-level tasks in manual order, done last, scoped to the folder', () => {
    expect(titles(viewTasks(tasks, 'work', 'all', ctx))).toEqual(['a', 'c', 'd', 'b', 'g']);
  });

  it('Overdue skips done tasks', () => {
    expect(titles(viewTasks(tasks, 'work', 'overdue', ctx))).toEqual(['c']);
  });

  it('Upcoming sorts by due date and includes subtasks', () => {
    expect(titles(viewTasks(tasks, '', 'upcoming', ctx))).toEqual(['d', 'f', 'e']);
  });

  it('Range filters due_date between the two days', () => {
    expect(titles(viewTasks(tasks, 'work', 'range', ctx))).toEqual(['c', 'd']);
  });

  it('My day: due today by default, plus and minus by hand', () => {
    const c = { ...ctx, myDay: new Map([['a', true], ['d', false]]) };
    expect(titles(viewTasks(tasks, '', 'myday', c))).toEqual(['a', 'f']);
    expect(inMyDay(tasks[3], ctx.today, ctx.myDay)).toBe(true);
  });

  it('counts every view', () => {
    expect(viewCounts(tasks, 'work', ctx)).toEqual({ all: 5, overdue: 1, myday: 1, upcoming: 2, range: 2 });
  });
});
