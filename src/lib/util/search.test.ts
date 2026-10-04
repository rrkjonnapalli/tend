import { expect, it } from 'vitest';
import { searchNeedle, textMatches } from './search.ts';

it('matches any field, ignoring case and outer spaces', () => {
  const needle = searchNeedle('  Rel ');
  expect(textMatches(['release_checklist', 'work'], needle)).toBe(true);
  expect(textMatches(['groceries', 'home'], needle)).toBe(false);
  expect(textMatches(['anything'], searchNeedle(''))).toBe(true);
});
