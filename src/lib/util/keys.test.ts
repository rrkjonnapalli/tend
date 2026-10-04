import { describe, expect, it } from 'vitest';
import { keyParts, label, matches } from './keys.ts';

const ev = (o: Partial<KeyboardEvent>) =>
  ({ key: '', code: '', metaKey: false, ctrlKey: false, shiftKey: false, altKey: false, ...o }) as KeyboardEvent;

describe('matches', () => {
  it('maps mod to meta on macOS and ctrl elsewhere', () => {
    expect(matches(ev({ key: '1', code: 'Digit1', metaKey: true }), 'mod+1', true)).toBe(true);
    expect(matches(ev({ key: '1', code: 'Digit1', ctrlKey: true }), 'mod+1', false)).toBe(true);
    expect(matches(ev({ key: '1', code: 'Digit1', ctrlKey: true }), 'mod+1', true)).toBe(false);
  });

  it('requires the exact modifiers', () => {
    expect(matches(ev({ key: 'T', code: 'KeyT', metaKey: true, shiftKey: true }), 'mod+t', true)).toBe(false);
    expect(matches(ev({ key: 'T', code: 'KeyT', metaKey: true, shiftKey: true }), 'mod+shift+t', true)).toBe(true);
  });

  it('reads the comma key', () => {
    expect(matches(ev({ key: ',', code: 'Comma', metaKey: true }), 'mod+,', true)).toBe(true);
  });
});

describe('label', () => {
  it('uses glyphs on macOS', () => {
    expect(label('mod+shift+t', true)).toBe('⌘⇧T');
  });

  it('spells out keys elsewhere', () => {
    expect(label('mod+shift+t', false)).toBe('Ctrl+Shift+T');
  });
});

describe('keyParts', () => {
  it('splits a chord into key caps', () => {
    expect(keyParts('mod+shift+t', true)).toEqual(['⌘', '⇧', 'T']);
    expect(keyParts('mod+,', false)).toEqual(['Ctrl', ',']);
  });
});
