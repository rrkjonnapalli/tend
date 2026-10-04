import { describe, expect, it } from 'vitest';
import { countdown, parseDuration } from './timer-parse.ts';

describe('parseDuration', () => {
  it('reads a bare number as minutes', () => {
    expect(parseDuration('10')).toEqual({ ok: true, ms: 600_000, label: '10:00' });
  });

  it('reads mm:ss with minutes of any length', () => {
    expect(parseDuration('90:30')).toEqual({ ok: true, ms: 5_430_000, label: '90:30' });
    expect(parseDuration(':45')).toEqual({ ok: true, ms: 45_000, label: '0:45' });
  });

  it('rejects zero, bad seconds and junk', () => {
    expect(parseDuration('0').ok).toBe(false);
    expect(parseDuration('00:00').ok).toBe(false);
    expect(parseDuration('1:60').ok).toBe(false);
    expect(parseDuration('ten').ok).toBe(false);
  });
});

describe('countdown', () => {
  it('rounds up to the next second and never goes negative', () => {
    expect(countdown(272_000)).toBe('4:32');
    expect(countdown(271_500)).toBe('4:32');
    expect(countdown(-5)).toBe('0:00');
    expect(countdown(3_600_000)).toBe('60:00');
  });
});
