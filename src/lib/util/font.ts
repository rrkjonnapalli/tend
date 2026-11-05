// The App font, Editor font and text size from Settings → General → Fonts, applied as CSS variables
// on the root (see tokens.css). A family is one installed on this device, or empty for Tend's own.
import type { FontStackInput, Fonts, TextSize } from '#lib/types/font.type.ts';

/** How much each text size multiplies the type tokens by. */
export const TEXT_SCALE: Record<TextSize, number> = { small: 0.9, default: 1, large: 1.15 };

/** A family name as a quoted CSS string, falling back to `fallback` (a CSS value) when it isn't installed. */
export function fontStack({ family, fallback }: FontStackInput): string {
  const quoted = family.replace(/\\/g, '\\\\').replace(/"/g, '\\"');
  return `"${quoted}", ${fallback}`;
}

/** Sets the root's font variables; an empty family goes back to Tend's own. */
export function applyFonts({ app, editor, size }: Fonts) {
  const root = document.documentElement.style;
  function set(name: string, value: string | null) {
    if (value) root.setProperty(name, value);
    else root.removeProperty(name);
  }
  const appStack = app ? fontStack({ family: app, fallback: 'var(--font-own-ui)' }) : null;
  const editorStack = editor ? fontStack({ family: editor, fallback: 'var(--font-own-editor)' }) : null;
  const scale = size === 'default' ? null : String(TEXT_SCALE[size] ?? 1);
  set('--font-ui', appStack);
  set('--font-heading', appStack);
  set('--font-editor', editorStack);
  set('--text-scale', scale);
}
