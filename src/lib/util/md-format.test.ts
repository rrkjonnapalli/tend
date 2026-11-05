import { describe, expect, it } from 'vitest';
import { mdFormat } from './md-format.ts';

/** Applies an edit and marks the selection with [ and ] (| for a caret). */
function run(doc: string, from: number, to: number, kind: Parameters<typeof mdFormat>[3]) {
  const e = mdFormat(doc, from, to, kind);
  const out = doc.slice(0, e.from) + e.insert + doc.slice(e.to);
  if (e.anchor === e.head) return out.slice(0, e.anchor) + '|' + out.slice(e.anchor);
  return out.slice(0, e.anchor) + '[' + out.slice(e.anchor, e.head) + ']' + out.slice(e.head);
}

describe('mdFormat', () => {
  it('wraps a selection in bold, italic or code and keeps it selected', () => {
    expect(run('a word here', 2, 6, 'bold')).toBe('a **[word]** here');
    expect(run('a word here', 2, 6, 'italic')).toBe('a _[word]_ here');
    expect(run('a word here', 2, 6, 'code')).toBe('a `[word]` here');
  });

  it('puts the caret between the marks when nothing is selected', () => {
    expect(run('ab', 1, 1, 'bold')).toBe('a**|**b');
  });

  it('makes a link and puts the caret where the url goes', () => {
    expect(run('see docs', 4, 8, 'link')).toBe('see [docs](|)');
  });

  it('adds a line prefix to every selected line, or removes it when all have it', () => {
    expect(run('one\ntwo', 1, 5, 'list')).toBe('- one\n- two|');
    expect(run('- one\n- two', 0, 11, 'list')).toBe('one\ntwo|');
    expect(run('buy milk', 3, 3, 'check')).toBe('- [ ] buy milk|');
    expect(run('Title', 0, 0, 'heading')).toBe('## Title|');
  });
});
