import { expect, it } from 'vitest';
import { statusValue } from './statuses.ts';

it('makes an UPPER_SNAKE value from a label', () => {
  expect(statusValue('In review')).toBe('IN_REVIEW');
  expect(statusValue('  blocked / waiting!  ')).toBe('BLOCKED_WAITING');
  expect(statusValue('2nd pass')).toBe('ND_PASS');
  expect(statusValue('')).toBe('');
});

