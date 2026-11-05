import { describe, expect, it } from 'vitest';
import { repoLabel } from './sync.ts';

describe('repoLabel', () => {
  it('drops the scheme and .git', () => {
    expect(repoLabel('https://github.com/you/notes.git')).toBe('github.com/you/notes');
    expect(repoLabel(' http://example.com/a/b ')).toBe('example.com/a/b');
  });
});
