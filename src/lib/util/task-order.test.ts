import { describe, expect, it } from 'vitest';
import type { Task } from '#lib/types/task.type.ts';
import { droppedKey, keyAfterLast } from './task-order.ts';

const task = (id: string, orderKey: string) => ({ id, orderKey }) as Task;

describe('droppedKey', () => {
  const tasks = [task('a', 'a0'), task('b', 'a1'), task('c', 'a2'), task('done', 'a0V')];

  it('puts a dropped task between its new neighbours', () => {
    // c dragged between a and b.
    const key = droppedKey({ order: ['a', 'c', 'b'], index: 1, tasks });
    expect(key > 'a0' && key < 'a1').toBe(true);
  });

  it('lands below the last open task even when a done task with a smaller key shows under it', () => {
    // a dragged to the end of the open tasks; the done task (key a0V, shown last) is skipped.
    const key = droppedKey({ order: ['b', 'c', 'a', 'done'], index: 2, tasks });
    expect(key > 'a2').toBe(true);
  });

  it('drops between tied keys (two devices inserted at the same spot) without failing', () => {
    const tied = [task('x', 'a1'), task('y', 'a1'), task('z', 'a2'), task('m', 'a0')];
    const key = droppedKey({ order: ['x', 'y', 'm', 'z'], index: 2, tasks: tied });
    expect(key > 'a1' && key < 'a2').toBe(true);
  });

  it('makes a key after the last task', () => {
    expect(keyAfterLast(tasks.slice(0, 3)) > 'a2').toBe(true);
  });
});
