import { expect, it } from 'vitest';
import { bytesLabel } from './bytes.ts';

it('formats sizes', () => {
  expect(bytesLabel(512)).toBe('512 B');
  expect(bytesLabel(245_760)).toBe('240 KB');
  expect(bytesLabel(1_572_864)).toBe('1.5 MB');
});
