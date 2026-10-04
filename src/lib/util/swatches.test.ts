import { expect, it } from 'vitest';
import { freeSwatch, isHexColor, SWATCHES } from './swatches.ts';

it('offers twenty distinct swatches', () => {
  expect(SWATCHES).toHaveLength(20);
  expect(new Set(SWATCHES.map(([, hex]) => hex)).size).toBe(20);
});

it('picks the first unused swatch', () => {
  expect(freeSwatch([])).toBe(SWATCHES[0][1]);
  expect(freeSwatch([SWATCHES[0][1]])).toBe(SWATCHES[1][1]);
  expect(freeSwatch(SWATCHES.map(([, hex]) => hex))).toBe(SWATCHES[0][1]);
});

it('accepts only #rrggbb colors', () => {
  expect(isHexColor('#1e7A55')).toBe(true);
  expect(isHexColor('1e7a55')).toBe(false);
  expect(isHexColor('#1e7a5')).toBe(false);
  expect(isHexColor('#gggggg')).toBe(false);
});
