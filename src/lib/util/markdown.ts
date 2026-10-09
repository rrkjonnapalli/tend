// Markdown to safe HTML for every rendered view.
import { marked } from 'marked';
import DOMPurify from 'dompurify';

marked.use({ gfm: true, breaks: false });

export function renderMarkdown(text: string): string {
  // A form could post the app window away to another site; Markdown never needs one.
  return DOMPurify.sanitize(marked.parse(text, { async: false }), { FORBID_TAGS: ['form'] });
}
