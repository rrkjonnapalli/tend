import { expect, it } from 'vitest';
import { displayName, initials } from './people.ts';

const priya = { firstName: 'Priya', lastName: 'Nair', nickname: null };

it('uses the nickname when set', () => {
  expect(displayName({ ...priya, nickname: ' Pri ' })).toBe('Pri');
  expect(displayName(priya)).toBe('Priya Nair');
  expect(displayName({ ...priya, lastName: '' })).toBe('Priya');
});

it('takes first and last initials', () => {
  expect(initials(priya)).toBe('PN');
  expect(initials({ ...priya, lastName: '' })).toBe('P');
  expect(initials({ firstName: '', lastName: '', nickname: null })).toBe('');
});
