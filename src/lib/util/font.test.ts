import { describe, expect, it } from 'vitest';
import { fontStack } from './font.ts';

describe('fontStack', () => {
  it('quotes the family and falls back to the given font', () => {
    expect(fontStack({ family: 'Iowan Old Style', fallback: 'var(--font-own-editor)' })).toBe('"Iowan Old Style", var(--font-own-editor)');
  });

  it('escapes quotes and backslashes so a name cannot end the string', () => {
    expect(fontStack({ family: 'A"b\\c', fallback: 'serif' })).toBe('"A\\"b\\\\c", serif');
  });
});
