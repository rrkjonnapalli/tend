/** The text size setting (Settings → General → Fonts). */
export type TextSize = 'small' | 'default' | 'large';

/** The three font settings as the root applies them; a family is '' for Tend's own. */
export type Fonts = { app: string; editor: string; size: TextSize };

/** A family name, and the CSS value to fall back on when it isn't installed. */
export type FontStackInput = { family: string; fallback: string };
