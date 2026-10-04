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

/** "mod+shift+t" → ["⌘", "⇧", "T"] on macOS, ["Ctrl", "Shift", "T"] elsewhere: one key cap each. */
export function keyParts(chord: string, mac = isMac): string[] {
  return chord.split('+').map((part) => {
    if (mac) return glyphs[part] ?? part.toUpperCase();
    if (part === 'mod') return 'Ctrl';
    return capitalise(part);
  });
}

function capitalise(word: string): string {
  return word[0].toUpperCase() + word.slice(1);
}

/** "mod+shift+t" → "⌘⇧T" on macOS, "Ctrl+Shift+T" elsewhere. */
export function label(chord: string, mac = isMac): string {
  return keyParts(chord, mac).join(mac ? '' : '+');
}

/** True while focus is in a text field or the note editor, where keys belong to typing. */
export function isTypingTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  const isField = /^(INPUT|TEXTAREA|SELECT)$/.test(target.tagName);
  const inEditor = target.closest('.cm-editor') !== null;
  return target.isContentEditable || isField || inEditor;
}
