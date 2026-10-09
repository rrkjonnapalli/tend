import type { DayRange } from '#lib/types/date.type.ts';
import type { Task } from '#lib/types/task.type.ts';

export type View = 'all' | 'overdue' | 'myday' | 'upcoming' | 'range';

/** The view last picked in the task list, and the day it was picked. */
export type LastView = { view: View; day: string };

/** `hideDone`: the list's "hide done" toggle; Done tasks are left out of every view. */
export type ViewContext = { today: string; myDay: Map<string, boolean>; range: DayRange; hideDone?: boolean };

/** My day state: the current day and the tasks added (true) or removed (false) by hand. */
export type MyDayContext = { today: string; myDay: Map<string, boolean> };

export type ViewScope = { tasks: Task[]; folder: string; ctx: ViewContext };
