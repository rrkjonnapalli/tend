import { describe, expect, it } from 'vitest';
import { fontStack } from './font.ts';

describe('fontStack', () => {
  it('quotes the family and falls back to the editor font', () => {
    expect(fontStack('Iowan Old Style')).toBe('"Iowan Old Style", var(--font-note)');
  });

  it('escapes quotes and backslashes so a name cannot end the string', () => {
    expect(fontStack('A"b\\c')).toBe('"A\\"b\\\\c", var(--font-note)');
  });
});
