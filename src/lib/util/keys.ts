// Keyboard chord matching. A chord is written like "mod+shift+t": "mod" is ⌘ on macOS and Ctrl elsewhere.

export const isMac = typeof navigator !== 'undefined' && /Mac|iPhone|iPad/.test(navigator.platform);

type Chord = { mod: boolean; shift: boolean; alt: boolean; key: string };

function parse(chord: string): Chord {
  const parts = chord.toLowerCase().split('+');
  const key = parts[parts.length - 1];
  return {
    mod: parts.includes('mod'),
    shift: parts.includes('shift'),
    alt: parts.includes('alt'),
    key,
  };
}

/** The key a chord refers to, normalised from a KeyboardEvent ("Digit1" → "1", "Comma" → ","). */
function keyOf(e: Pick<KeyboardEvent, 'key' | 'code'>): string {
  if (/^Digit\d$/.test(e.code)) return e.code.slice(5);
  if (/^Key[A-Z]$/.test(e.code)) return e.code.slice(3).toLowerCase();
  if (e.code === 'Comma') return ',';
  return e.key.toLowerCase();
}

export function matches(
  e: Pick<KeyboardEvent, 'key' | 'code' | 'metaKey' | 'ctrlKey' | 'shiftKey' | 'altKey'>,
  chord: string,
  mac = isMac,
): boolean {
  const c = parse(chord);
  const mod = mac ? e.metaKey : e.ctrlKey;
  return mod === c.mod && e.shiftKey === c.shift && e.altKey === c.alt && keyOf(e) === c.key;
}

const glyphs: Record<string, string> = {
  mod: '⌘',
  shift: '⇧',
  alt: '⌥',
  enter: '↵',
  backspace: '⌫',
  escape: 'Esc',
};

/** "mod+shift+t" → "⌘⇧T" on macOS, "Ctrl+Shift+T" elsewhere. */
export function label(chord: string, mac = isMac): string {
  const parts = chord.split('+');
  if (!mac) {
    return parts
      .map((p) => (p === 'mod' ? 'Ctrl' : p.length === 1 ? p.toUpperCase() : p[0].toUpperCase() + p.slice(1)))
      .join('+');
  }
  return parts.map((p) => glyphs[p] ?? p.toUpperCase()).join('');
}
