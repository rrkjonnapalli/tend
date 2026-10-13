import { describe, expect, it } from 'vitest';
import { addDays, ago, dayOfMonth, dueLabel, joinTime, monthBounds, monthGrid, mondayIndex, calendarStart, clampYear, dobLabel, monthDay, monthDays, nextBirthday, parseDOB, parseTime, pastYears, planYears, stepMonth, shortDate, splitTime, timeLabel, timestamp } from './dates.ts';

const NOW = new Date(2026, 9, 4, 12, 0); // Sun 4 Oct 2026, noon

describe('calendar', () => {
  it('builds a Monday-first month grid', () => {
    const cells = monthGrid(2026, 9); // October 2026 starts on a Thursday
    expect(cells.slice(0, 4)).toEqual([null, null, null, '2026-10-01']);
    expect(cells.at(-1)).toBe('2026-10-31');
  });

  it('counts weekdays from Monday and finds month bounds', () => {
    expect(mondayIndex(new Date(2026, 9, 5))).toBe(0); // Monday
    expect(mondayIndex(new Date(2026, 9, 4))).toBe(6); // Sunday
    expect(monthBounds('2026-02-14')).toEqual(['2026-02-01', '2026-02-28']);
    expect(dayOfMonth('2026-10-09')).toBe(9);
  });

  it('adds days across months', () => {
    expect(addDays('2026-10-31', 1)).toBe('2026-11-01');
  });

  it('labels dates', () => {
    expect(shortDate('2026-10-10', NOW)).toBe('10 Oct');
    expect(shortDate('2027-01-02', NOW)).toBe('2 Jan 2027');
    expect(dueLabel('2026-10-04', NOW)).toBe('Today');
    expect(dueLabel('2026-10-05', NOW)).toBe('Tomorrow');
    expect(dueLabel('2026-10-03', NOW)).toBe('Yesterday');
    expect(dueLabel('2026-10-07', NOW)).toBe('Wed');
    expect(dueLabel('2026-10-20', NOW)).toBe('20 Oct');
  });

  it('says how long ago', () => {
    const t = NOW.getTime();
    expect(ago(t - 10_000, NOW)).toBe('now');
    expect(ago(t - 120_000, NOW)).toBe('2m');
    expect(ago(t - 3 * 3_600_000, NOW)).toBe('3h');
    expect(ago(t - 24 * 3_600_000, NOW)).toBe('Yesterday');
    expect(ago(new Date(2026, 8, 24).getTime(), NOW)).toBe('24 Sep');
  });
});

describe('times', () => {
  it('parses free-form times', () => {
    expect(parseTime('5pm')).toBe('17:00');
    expect(parseTime('5:30 pm')).toBe('17:30');
    expect(parseTime('12am')).toBe('00:00');
    expect(parseTime('17:30')).toBe('17:30');
    expect(parseTime('25:00')).toBeNull();
    expect(parseTime('13pm')).toBeNull();
  });

  it('round-trips 12-hour parts', () => {
    expect(splitTime('17:05')).toEqual({ h: 5, m: 5, pm: true });
    expect(splitTime('00:15')).toEqual({ h: 12, m: 15, pm: false });
    expect(joinTime({ h: 12, m: 15, pm: false })).toBe('00:15');
    expect(joinTime({ h: 12, m: 0, pm: true })).toBe('12:00');
    expect(timeLabel('09:30')).toBe('9:30 AM');
  });
});

describe('birthdays', () => {
  it('parses full and year-less dates', () => {
    expect(parseDOB('1995-10-10')).toEqual({ year: 1995, month: 10, day: 10 });
    expect(parseDOB('10-10')).toEqual({ year: null, month: 10, day: 10 });
    expect(parseDOB('')).toBeNull();
    expect(parseDOB('2023-02-30')).toBe('invalid');
    expect(parseDOB('10/10')).toBe('invalid');
  });

  it('shows a date of birth with its year when it has one', () => {
    expect(dobLabel({ year: 1990, month: 5, day: 12 })).toBe('12 May 1990');
    expect(dobLabel({ year: null, month: 5, day: 2 })).toBe('2 May');
  });

  it('lists the years a calendar offers', () => {
    const now = new Date(2026, 9, 14);
    expect(pastYears(now).slice(0, 3)).toEqual([2026, 2025, 2024]);
    expect(pastYears(now).at(-1)).toBe(1900);
    expect(planYears(now)).toEqual([2025, 2026, 2027, 2028, 2029, 2030, 2031, 2032, 2033, 2034, 2035, 2036]);
    expect(clampYear(1850)).toBe(1900);
    expect(clampYear(2300)).toBe(2139);
  });

  it('steps months round the year', () => {
    expect(stepMonth(4, 1)).toEqual({ month: 5, years: 0 });
    expect(stepMonth(11, 1)).toEqual({ month: 0, years: 1 });
    expect(stepMonth(0, -1)).toEqual({ month: 11, years: -1 });
  });

  it('handles a day whose year is not known', () => {
    expect(monthDay(4, 2)).toBe('05-02');
    expect(monthDays(1)).toHaveLength(29);
    expect(monthDays(3).at(-1)).toBe('04-30');
    expect(calendarStart('1990-05-12')).toEqual({ year: 1990, month: 4 });
    expect(calendarStart('05-12')).toEqual({ year: null, month: 4 });
  });

  it('finds the next birthday and age', () => {
    expect(nextBirthday({ year: 1995, month: 10, day: 10 }, NOW)).toEqual({ date: '2026-10-10', days: 6, age: 31 });
    expect(nextBirthday({ year: null, month: 10, day: 4 }, NOW)).toEqual({ date: '2026-10-04', days: 0, age: null });
    expect(nextBirthday({ year: 2000, month: 1, day: 1 }, NOW).date).toBe('2027-01-01');
  });
});

describe('timestamp', () => {
  it('shows the day and the time to the second', () => {
    const at = (day: string, time: string) => new Date(`${day}T${time}`).getTime();
    expect(timestamp(at('2026-10-04', '17:05:09'), NOW)).toBe('Today, 5:05:09 PM');
    expect(timestamp(at('2026-10-03', '00:00:00'), NOW)).toBe('Yesterday, 12:00:00 AM');
    expect(timestamp(at('2026-09-24', '09:30:01'), NOW)).toBe('24 Sep, 9:30:01 AM');
  });
});
