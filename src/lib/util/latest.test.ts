import { describe, expect, it } from 'vitest';
import { latestOnly } from './latest.ts';

describe('latestOnly', () => {
  it('drops an older reply that lands after a newer one', async () => {
    const replies: ((value: string) => void)[] = [];
    const applied: string[] = [];
    const load = latestOnly(
      () => new Promise<string>((resolve) => replies.push(resolve)),
      (value) => applied.push(value),
    );
    const first = load();
    const second = load();
    replies[1]('new');
    replies[0]('old');
    await Promise.all([first, second]);
    expect(applied).toEqual(['new']);
  });
});
