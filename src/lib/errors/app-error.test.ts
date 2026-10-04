import { expect, it } from 'vitest';
import { AppError, messageOf, toAppError } from './app-error.ts';
import { errors } from './registry.ts';

it('keeps the key and message the backend sent', () => {
  const e = toAppError({ key: 'NAME_TAKEN', message: 'Something with this name already exists here' });
  expect(e.is('NAME_TAKEN')).toBe(true);
  expect(e.message).toBe('Something with this name already exists here');
});

it('uses the registry message unless the case gives its own', () => {
  expect(new AppError(errors.TIMEOUT).message).toBe(errors.TIMEOUT.message);
  expect(new AppError(errors.UNKNOWN, 'Disk full').message).toBe('Disk full');
});

it('turns anything else thrown into UNKNOWN', () => {
  expect(toAppError(new Error('boom')).is(errors.UNKNOWN)).toBe(true);
  expect(messageOf('plain text')).toBe('plain text');
  expect(messageOf(42)).toBe(errors.UNKNOWN.message);
});
