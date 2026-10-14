// Markdown to safe HTML for every rendered view.
import { marked } from 'marked';
import DOMPurify from 'dompurify';

// A newline in a paragraph shows as a line break, as typed (Obsidian's default), not a space.
marked.use({ gfm: true, breaks: true });

export function renderMarkdown(text: string): string {
  // A form could post the app window away to another site; Markdown never needs one.
  return DOMPurify.sanitize(marked.parse(text, { async: false }), { FORBID_TAGS: ['form'] });
}
