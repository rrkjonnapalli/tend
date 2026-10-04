export type DOB = { year: number | null; month: number; day: number };

export type DayRange = [string, string];

/** A time of day in 12-hour parts: '17:05' is { h: 5, m: 5, pm: true }. */
export type TimeParts = { h: number; m: number; pm: boolean };

/** A named range offered next to the calendar (Today, This week, …). */
export type DatePreset = { name: string; range: DayRange };
