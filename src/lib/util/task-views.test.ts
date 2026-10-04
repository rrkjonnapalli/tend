import { describe, expect, it } from 'vitest';
import type { Task } from '#lib/types/task.type.ts';
import { defaultView, inMyDay, viewCounts, viewTasks } from './task-views.ts';
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
    expect(titles(viewTasks({ tasks, folder: 'work', view: 'all', ctx }))).toEqual(['a', 'c', 'd', 'b', 'g']);
  });

  it('Overdue skips done tasks', () => {
    expect(titles(viewTasks({ tasks, folder: 'work', view: 'overdue', ctx }))).toEqual(['c']);
  });

  it('Upcoming sorts by due date and includes subtasks', () => {
    expect(titles(viewTasks({ tasks, folder: '', view: 'upcoming', ctx }))).toEqual(['d', 'f', 'e']);
  });

  it('Range filters due_date between the two days', () => {
    expect(titles(viewTasks({ tasks, folder: 'work', view: 'range', ctx }))).toEqual(['c', 'd']);
  });

  it('My day: due today by default, plus and minus by hand', () => {
    const c = { ...ctx, myDay: new Map([['a', true], ['d', false]]) };
    expect(titles(viewTasks({ tasks, folder: '', view: 'myday', ctx: c }))).toEqual(['a', 'f']);
    expect(inMyDay(tasks[3], ctx)).toBe(true);
  });

  it('counts every view', () => {
    expect(viewCounts({ tasks, folder: 'work', ctx })).toEqual({ all: 5, overdue: 1, myday: 1, upcoming: 2, range: 2 });
  });
});

describe('defaultView', () => {
  it('opens My day on the first visit of a day', () => {
    expect(defaultView({ last: null, today: '2026-10-04' })).toBe('myday');
    expect(defaultView({ last: { view: 'all', day: '2026-10-03' }, today: '2026-10-04' })).toBe('myday');
  });

  it('keeps the view picked earlier today', () => {
    expect(defaultView({ last: { view: 'all', day: '2026-10-04' }, today: '2026-10-04' })).toBe('all');
  });
});
