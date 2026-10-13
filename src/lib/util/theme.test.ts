import { describe, expect, it } from 'vitest';
import { resolveTheme, toggledTheme } from './theme.ts';

describe('resolveTheme', () => {
  it('uses the picked theme, or the OS for "system"', () => {
    expect(resolveTheme('dark', false)).toBe('dark');
    expect(resolveTheme('light', true)).toBe('light');
    expect(resolveTheme('system', true)).toBe('dark');
    expect(resolveTheme('system', false)).toBe('light');
  });
});

describe('toggledTheme', () => {
  it('switches to the theme not shown now, from a picked one or from the OS', () => {
    expect(toggledTheme('dark', false)).toBe('light');
    expect(toggledTheme('light', true)).toBe('dark');
    expect(toggledTheme('system', true)).toBe('light');
    expect(toggledTheme('system', false)).toBe('dark');
  });
});
