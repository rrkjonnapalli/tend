import { describe, expect, it } from 'vitest';
import { onBeforeQuit, runBeforeQuit } from './before-quit.ts';

describe('before quit', () => {
  it('runs every registered hook, even after one fails, and not after it is removed', async () => {
    const ran: string[] = [];
    const stopA = onBeforeQuit(() => {
      ran.push('a');
      throw new Error('disk');
    });
    onBeforeQuit(async () => {
      ran.push('b');
    });
    await runBeforeQuit();
    expect(ran).toEqual(['a', 'b']);
    stopA();
    await runBeforeQuit();
    expect(ran).toEqual(['a', 'b', 'b']);
  });
});
