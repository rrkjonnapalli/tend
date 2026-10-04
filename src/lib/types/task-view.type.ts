import type { DayRange } from '#lib/types/date.type.ts';
import type { Task } from '#lib/types/task.type.ts';

export type View = 'all' | 'overdue' | 'myday' | 'upcoming' | 'range';

export type ViewContext = { today: string; myDay: Map<string, boolean>; range: DayRange };

/** My day state: the current day and the tasks added (true) or removed (false) by hand. */
export type MyDayContext = { today: string; myDay: Map<string, boolean> };

export type ViewScope = { tasks: Task[]; folder: string; ctx: ViewContext };
