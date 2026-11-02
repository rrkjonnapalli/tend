// The note view's reading font: a family installed on this device, or none for Tend's own.

/** The CSS variable the note view reads (MarkdownView `.note`). */
const READING_VAR = '--font-reading';

/** A family name as a quoted CSS string, falling back to Tend's note font when it isn't installed. */
export function fontStack(family: string): string {
  const quoted = family.replace(/\\/g, '\\\\').replace(/"/g, '\\"');
  return `"${quoted}", var(--font-note)`;
}

/** Shows notes in `family`; an empty one goes back to Tend's own font. */
export function applyReadingFont(family: string) {
  const root = document.documentElement.style;
  if (!family) {
    root.removeProperty(READING_VAR);
    return;
  }
  root.setProperty(READING_VAR, fontStack(family));
}
