import { describe, expect, it } from 'vitest';
import { birthdayHint, birthdayLabel } from './birthdays.ts';

const NOW = new Date(2026, 9, 3, 12, 0); // 3 Oct 2026

describe('birthdays', () => {
  it('labels the list column', () => {
    expect(birthdayLabel({ year: 1995, month: 10, day: 9 }, NOW)).toEqual({ text: '9 Oct · in 6 days · turns 31', soon: true, days: 6 });
    expect(birthdayLabel({ year: null, month: 10, day: 3 }, NOW)).toEqual({ text: '3 Oct · today', soon: true, days: 0 });
    expect(birthdayLabel({ year: 1991, month: 1, day: 17 }, NOW).text).toBe('17 Jan · in 106 days · turns 36');
    expect(birthdayLabel({ year: 1991, month: 1, day: 17 }, NOW).soon).toBe(false);
  });

  it('builds the person page hint', () => {
    expect(birthdayHint({ year: 1995, month: 10, day: 9 }, NOW)).toBe('next birthday 9 Oct, turns 31 · in 6 days');
  });
});
