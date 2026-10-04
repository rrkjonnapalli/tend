import { expect, it } from 'vitest';
import { freeSwatch, SWATCHES } from './swatches.ts';

it('offers twenty distinct swatches', () => {
  expect(SWATCHES).toHaveLength(20);
  expect(new Set(SWATCHES.map(([, hex]) => hex)).size).toBe(20);
});

it('picks the first unused swatch', () => {
  expect(freeSwatch([])).toBe(SWATCHES[0][1]);
  expect(freeSwatch([SWATCHES[0][1]])).toBe(SWATCHES[1][1]);
  expect(freeSwatch(SWATCHES.map(([, hex]) => hex))).toBe(SWATCHES[0][1]);
});
