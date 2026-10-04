import { describe, expect, it } from 'vitest';
import { birthdayHint, birthdayLabel, personBirthday } from './birthdays.ts';

const NOW = new Date(2026, 9, 3, 12, 0); // 3 Oct 2026

describe('birthdays', () => {
  it('labels the list column', () => {
    expect(birthdayLabel({ year: 1995, month: 10, day: 9 }, NOW)).toEqual({ text: '9 Oct · in 6 days · turns 31', soon: true, days: 6 });
    expect(birthdayLabel({ year: null, month: 10, day: 3 }, NOW)).toEqual({ text: '3 Oct · today', soon: true, days: 0 });
    expect(birthdayLabel({ year: null, month: 10, day: 4 }, NOW).text).toBe('4 Oct · tomorrow');
    expect(birthdayLabel({ year: 1991, month: 1, day: 17 }, NOW).text).toBe('17 Jan · in 106 days · turns 36');
    expect(birthdayLabel({ year: 1991, month: 1, day: 17 }, NOW).soon).toBe(false);
  });

  it('labels a person only when a birthday is set', () => {
    expect(personBirthday({ dobYear: null, dobMonth: null, dobDay: null }, NOW)).toBeNull();
    expect(personBirthday({ dobYear: 1995, dobMonth: 10, dobDay: 9 }, NOW)?.days).toBe(6);
  });

  it('builds the person page hint', () => {
    expect(birthdayHint({ year: 1995, month: 10, day: 9 }, NOW)).toBe('next birthday 9 Oct, turns 31 · in 6 days');
  });
});
