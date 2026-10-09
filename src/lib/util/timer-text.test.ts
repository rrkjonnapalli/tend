import { describe, expect, it } from 'vitest';
import { durationWords, timerName, timerStartedText } from './timer-text.ts';

describe('timer text', () => {
  it('names untitled timers "Timer"', () => {
    expect(timerName({ title: 'Tea' })).toBe('Tea');
    expect(timerName({ title: '' })).toBe('Timer');
  });

  it('leaves the title out of the started message when there is none', () => {
    expect(timerStartedText({ label: '10m', title: 'Tea' })).toBe('Timer started · 10m · Tea');
    expect(timerStartedText({ label: '1:00', title: '' })).toBe('Timer started · 1:00');
  });
});

describe('durationWords', () => {
  it('says a duration in minutes and seconds, leaving out a zero part', () => {
    expect(durationWords(5_000)).toBe('5 seconds');
    expect(durationWords(60_000)).toBe('1 minute');
    expect(durationWords(180_000)).toBe('3 minutes');
    expect(durationWords(90_000)).toBe('1 minute 30 seconds');
    expect(durationWords(61_000)).toBe('1 minute 1 second');
  });
});
