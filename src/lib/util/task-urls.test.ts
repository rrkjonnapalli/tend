import { expect, it } from 'vitest';
import { folderLabel } from './task-urls.ts';

it('names the location from the root down', () => {
  expect(folderLabel({ rootName: 'tasks', path: '' })).toBe('tasks');
  expect(folderLabel({ rootName: 'tasks', path: 'work/tend_v0_1' })).toBe('tasks / work / tend_v0_1');
});
