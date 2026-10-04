import { expect, it } from 'vitest';
import { displayName, initials, personMatches, splitName } from './people.ts';

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

it('splits a typed name into first and last', () => {
  expect(splitName('  Ravi Kumar Rao ')).toEqual({ first: 'Ravi', last: 'Kumar Rao' });
  expect(splitName('Ravi')).toEqual({ first: 'Ravi', last: '' });
  expect(splitName('   ')).toEqual({ first: '', last: '' });
});

it('matches a query against name, phone and email', () => {
  const row = { id: '1', ...priya, dobYear: null, dobMonth: null, dobDay: null, phone: '+91 98765', email: 'priya@example.com' };
  expect(personMatches(row, '')).toBe(true);
  expect(personMatches(row, 'nair')).toBe(true);
  expect(personMatches(row, '98765')).toBe(true);
  expect(personMatches(row, 'example')).toBe(true);
  expect(personMatches(row, 'ravi')).toBe(false);
});
