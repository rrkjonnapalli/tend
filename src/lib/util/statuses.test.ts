import { expect, it } from 'vitest';
import { freeSwatch, statusValue, SWATCHES } from './statuses.ts';

it('makes an UPPER_SNAKE value from a label', () => {
  expect(statusValue('In review')).toBe('IN_REVIEW');
  expect(statusValue('  blocked / waiting!  ')).toBe('BLOCKED_WAITING');
  expect(statusValue('2nd pass')).toBe('ND_PASS');
  expect(statusValue('')).toBe('');
});

it('picks the first unused swatch', () => {
  expect(freeSwatch([])).toBe(SWATCHES[0][1]);
  expect(freeSwatch([SWATCHES[0][1]])).toBe(SWATCHES[1][1]);
  expect(freeSwatch(SWATCHES.map(([, hex]) => hex))).toBe(SWATCHES[0][1]);
});
