export type View = 'all' | 'overdue' | 'myday' | 'upcoming' | 'range';

export type ViewContext = { today: string; myDay: Map<string, boolean>; range: [string, string] };
