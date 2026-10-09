// Local calendar dates as 'YYYY-MM-DD' and times as 'HH:MM' (24h), matching the database.
import type { DayRange, DOB, TimeParts } from '#lib/types/date.type.ts';

const MONTHS = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];
const MONTHS_LONG = ['January', 'February', 'March', 'April', 'May', 'June', 'July', 'August', 'September', 'October', 'November', 'December'];
const WEEKDAYS = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];
const MINUTE_MS = 60_000;
const HOUR_MS = 3_600_000;
const DAY_MS = 86_400_000;
const WEEK_DAYS = 7;

const pad = (n: number) => String(n).padStart(2, '0');

export const iso = (d: Date) => `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`;

export function parseISO(s: string): Date {
  const [y, m, d] = s.split('-').map(Number);
  return new Date(y, m - 1, d);
}

export const today = (now = new Date()) => iso(now);

export function addDays(day: string, n: number): string {
  const d = parseISO(day);
  d.setDate(d.getDate() + n);
  return iso(d);
}

/** Whole days from `a` to `b` (positive when `b` is later). */
export const daysBetween = (a: string, b: string) => Math.round((parseISO(b).getTime() - parseISO(a).getTime()) / DAY_MS);

/** The weekday counted from Monday: Monday is 0, Sunday is 6. */
export function mondayIndex(d: Date): number {
  const sundayFirst = d.getDay();
  return (sundayFirst + WEEK_DAYS - 1) % WEEK_DAYS;
}

/** '2026-10-21' → 21. */
export const dayOfMonth = (day: string) => Number(day.slice(8));

/** The first and last day of the month that `day` falls in. */
export function monthBounds(day: string): DayRange {
  const d = parseISO(day);
  const first = iso(new Date(d.getFullYear(), d.getMonth(), 1));
  const last = iso(new Date(d.getFullYear(), d.getMonth() + 1, 0));
  return [first, last];
}

/** The cells of a month view, Monday first; `null` pads the first week. */
export function monthGrid(year: number, month: number): (string | null)[] {
  const lead = mondayIndex(new Date(year, month, 1));
  const count = new Date(year, month + 1, 0).getDate();
  const cells: (string | null)[] = Array(lead).fill(null);
  for (let d = 1; d <= count; d++) cells.push(iso(new Date(year, month, d)));
  return cells;
}

export const monthTitle = (year: number, month: number) => `${MONTHS_LONG[month]} ${year}`;

/** '2026-10-10' → '10 Oct' (year added when it isn't the current one). */
export function shortDate(day: string, now = new Date()): string {
  const d = parseISO(day);
  const base = `${d.getDate()} ${MONTHS[d.getMonth()]}`;
  return d.getFullYear() === now.getFullYear() ? base : `${base} ${d.getFullYear()}`;
}

/** A due date as the task list shows it: Today, Tomorrow, Yesterday, a weekday this week, or '10 Oct'. */
export function dueLabel(day: string, now = new Date()): string {
  const diff = daysBetween(today(now), day);
  if (diff === 0) return 'Today';
  if (diff === 1) return 'Tomorrow';
  if (diff === -1) return 'Yesterday';
  const isThisWeek = diff > 1 && diff < WEEK_DAYS;
  if (isThisWeek) return WEEKDAYS[parseISO(day).getDay()];
  return shortDate(day, now);
}

/** How long ago a moment was, in the style of the notes list: now, 2m, 3h, Yesterday, Mon, 24 Sep. */
export function ago(ms: number, now = new Date()): string {
  const diff = now.getTime() - ms;
  if (diff < MINUTE_MS) return 'now';
  if (diff < HOUR_MS) return `${Math.floor(diff / MINUTE_MS)}m`;
  const day = iso(new Date(ms));
  const days = daysBetween(day, today(now));
  if (days === 0) return `${Math.floor(diff / HOUR_MS)}h`;
  if (days === 1) return 'Yesterday';
  if (days < WEEK_DAYS) return WEEKDAYS[new Date(ms).getDay()];
  return shortDate(day, now);
}

/** Free-form time → 'HH:MM'. Accepts 5pm, 5:30 pm, 17:30. */
export function parseTime(text: string): string | null {
  const v = text.trim().toLowerCase().replace(/\s+/g, '');
  let m = v.match(/^(\d{1,2})(?::(\d{2}))?(am|pm)$/);
  if (m) {
    const h = Number(m[1]);
    const mi = Number(m[2] ?? 0);
    if (h < 1 || h > 12 || mi > 59) return null;
    return joinTime({ h, m: mi, pm: m[3] === 'pm' });
  }
  m = v.match(/^(\d{1,2}):(\d{2})$/);
  if (m) {
    const h = Number(m[1]);
    const mi = Number(m[2]);
    if (h > 23 || mi > 59) return null;
    return `${pad(h)}:${pad(mi)}`;
  }
  return null;
}

/** '17:05' → { h: 5, m: 5, pm: true }. */
export function splitTime(hhmm: string): TimeParts {
  const [h24, m] = hhmm.split(':').map(Number);
  let h = h24 % 12;
  if (h === 0) h = 12;
  return { h, m, pm: h24 >= 12 };
}

/** 12-hour parts back to 'HH:MM'. */
export function joinTime({ h, m, pm }: TimeParts): string {
  const offset = pm ? 12 : 0;
  const h24 = (h % 12) + offset;
  return `${pad(h24)}:${pad(m)}`;
}

/** '17:05' → '5:05 PM'. */
export function timeLabel(hhmm: string): string {
  const t = splitTime(hhmm);
  return `${t.h}:${pad(t.m)} ${t.pm ? 'PM' : 'AM'}`;
}

/** A moment to the second: 'Today, 5:05:09 PM', 'Yesterday, …', or '10 Oct, …'. */
export function timestamp(ms: number, now = new Date()): string {
  const d = new Date(ms);
  const day = iso(d);
  const diff = daysBetween(day, today(now));
  let date = shortDate(day, now);
  if (diff === 0) date = 'Today';
  if (diff === 1) date = 'Yesterday';
  const time = timeLabel(`${pad(d.getHours())}:${pad(d.getMinutes())}`);
  const [clock, half] = time.split(' ');
  return `${date}, ${clock}:${pad(d.getSeconds())} ${half}`;
}

/** 'yyyy-mm-dd' or 'mm-dd' → parts; '' → null; anything else → 'invalid'. */
export function parseDOB(text: string): DOB | null | 'invalid' {
  const v = text.trim();
  if (!v) return null;
  let m = v.match(/^(\d{4})-(\d{2})-(\d{2})$/);
  let year: number | null = null;
  let month: number;
  let day: number;
  if (m) {
    [year, month, day] = [Number(m[1]), Number(m[2]), Number(m[3])];
  } else {
    m = v.match(/^(\d{2})-(\d{2})$/);
    if (!m) return 'invalid';
    [month, day] = [Number(m[1]), Number(m[2])];
  }
  // A leap year, so 02-29 is valid when the year is unknown.
  const probe = new Date(year ?? 2000, month - 1, day);
  const rolledOver = probe.getMonth() !== month - 1 || probe.getDate() !== day;
  if (rolledOver) return 'invalid';
  return { year, month, day };
}

export function formatDOB(d: DOB): string {
  const monthDay = `${pad(d.month)}-${pad(d.day)}`;
  if (!d.year) return monthDay;
  return `${d.year}-${monthDay}`;
}

/** The next birthday on or after today: its date, days until it, and the age turned (when the year is known). */
export function nextBirthday(d: DOB, now = new Date()): { date: string; days: number; age: number | null } {
  const t = today(now);
  const year = now.getFullYear();
  let next = iso(new Date(year, d.month - 1, d.day));
  if (next < t) next = iso(new Date(year + 1, d.month - 1, d.day));
  const nextYear = parseISO(next).getFullYear();
  const age = d.year ? nextYear - d.year : null;
  return { date: next, days: daysBetween(t, next), age };
}
