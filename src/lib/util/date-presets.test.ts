import { describe, expect, it } from 'vitest';
import { rangePresets } from './date-presets.ts';

describe('rangePresets', () => {
  it('builds Monday-first weeks and whole months', () => {
    const presets = Object.fromEntries(rangePresets('2026-10-04').map((p) => [p.name, p.range]));
    expect(presets['Today']).toEqual(['2026-10-04', '2026-10-04']);
    expect(presets['Tomorrow']).toEqual(['2026-10-05', '2026-10-05']);
    expect(presets['This week']).toEqual(['2026-09-28', '2026-10-04']);
    expect(presets['Next 7 days']).toEqual(['2026-10-04', '2026-10-10']);
    expect(presets['This month']).toEqual(['2026-10-01', '2026-10-31']);
  });
});
