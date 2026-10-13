export type DOB = { year: number | null; month: number; day: number };

export type DayRange = [string, string];

/** A time of day in 12-hour parts: '17:05' is { h: 5, m: 5, pm: true }. */
export type TimeParts = { h: number; m: number; pm: boolean };

/** A named range offered next to the calendar (Today, This week, …). */
export type DatePreset = { name: string; range: DayRange };

/** DateField's props. Single mode picks one day; range mode one day or two. */
export type DateFieldProps =
  | {
      mode?: 'single';
      /** 'yyyy-mm-dd', or 'mm-dd' when `yearOptional` and the year is unknown. */
      value: string | null;
      onchange: (day: string | null) => void;
      warn?: boolean;
      /** What the day is, for the labels: "due date" (the default), "date of birth". */
      what?: string;
      /** How the button shows the day; the short date when left out. */
      format?: (day: string) => string;
      /** Days before today are greyed; on unless turned off (a date of birth is always past). */
      dimPast?: boolean;
      /** The calendar's year can stay unassigned, which keeps the day as 'mm-dd'. */
      yearOptional?: boolean;
    }
  | { mode: 'range'; range: DayRange; onrange: (r: DayRange) => void; active?: boolean; marked?: Set<string> };
