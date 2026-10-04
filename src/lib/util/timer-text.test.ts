import { describe, expect, it } from 'vitest';
import { timerName, timerStartedText } from './timer-text.ts';

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
