import { describe, expect, it } from 'vitest';
import { fontChoices, fontStack } from './font.ts';

describe('fontStack', () => {
  it('quotes the family and falls back to the given font', () => {
    expect(fontStack({ family: 'Iowan Old Style', fallback: 'var(--font-own-editor)' })).toBe('"Iowan Old Style", var(--font-own-editor)');
  });

  it('escapes quotes and backslashes so a name cannot end the string', () => {
    expect(fontStack({ family: 'A"b\\c', fallback: 'serif' })).toBe('"A\\"b\\\\c", serif');
  });
});

describe('fontChoices', () => {
  it('lists Tend’s own first, then the installed families', () => {
    expect(fontChoices({ installed: ['Inter'], picked: '', own: 'Lato' })).toEqual([
      { id: '', label: 'Tend’s own (Lato)' },
      { id: 'Inter', label: 'Inter' },
    ]);
  });

  it('keeps a picked family that is gone, marked once the list has loaded', () => {
    expect(fontChoices({ installed: ['Inter'], picked: 'Gone', own: 'Lato' })[1]).toEqual({ id: 'Gone', label: 'Gone (not installed)' });
    expect(fontChoices({ installed: [], picked: 'Gone', own: 'Lato' })[1]).toEqual({ id: 'Gone', label: 'Gone' });
  });
});
