import { describe, expect, it } from 'vitest';
import { resolveTheme } from './theme.ts';

describe('resolveTheme', () => {
  it('uses the picked theme, or the OS for "system"', () => {
    expect(resolveTheme('dark', false)).toBe('dark');
    expect(resolveTheme('light', true)).toBe('light');
    expect(resolveTheme('system', true)).toBe('dark');
    expect(resolveTheme('system', false)).toBe('light');
  });
});
