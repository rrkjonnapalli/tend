// The App font, Editor font and text size from Settings → General → Fonts, applied as CSS variables
// on the root (see tokens.css). A family is one installed on this device, or empty for Tend's own.
import type { TextSize } from '#lib/types/font.type.ts';

/** How much each text size multiplies the type tokens by. */
export const TEXT_SCALE: Record<TextSize, number> = { small: 0.9, default: 1, large: 1.15 };

/** A family name as a quoted CSS string, falling back to `fallback` (a CSS value) when it isn't installed. */
export function fontStack(family: string, fallback: string): string {
  const quoted = family.replace(/\\/g, '\\\\').replace(/"/g, '\\"');
  return `"${quoted}", ${fallback}`;
}

type Fonts = { app: string; editor: string; size: TextSize };

/** Sets the root's font variables; an empty family goes back to Tend's own. */
export function applyFonts({ app, editor, size }: Fonts) {
  const root = document.documentElement.style;
  const set = (name: string, value: string | null) => (value ? root.setProperty(name, value) : root.removeProperty(name));
  const appStack = app ? fontStack(app, 'var(--font-own-ui)') : null;
  set('--font-ui', appStack);
  set('--font-heading', appStack);
  set('--font-editor', editor ? fontStack(editor, 'var(--font-own-editor)') : null);
  set('--text-scale', size === 'default' ? null : String(TEXT_SCALE[size] ?? 1));
}
