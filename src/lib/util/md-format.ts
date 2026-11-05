// The phone's formatting bar (MD-Note-Edit): Markdown edits on a selection. Each returns the text
// to put in place of [from, to] and where the selection lands, so the editor applies it as one change.

import type { MdEdit, MdFormat, MdFormatInput } from '#lib/types/md-format.type.ts';

const WRAPS: Partial<Record<MdFormat, [string, string]>> = {
  bold: ['**', '**'],
  italic: ['_', '_'],
  code: ['`', '`'],
  link: ['[', ']()'],
};

const PREFIXES: Partial<Record<MdFormat, string>> = { heading: '## ', list: '- ', check: '- [ ] ' };

/** The edit for `kind` on `doc` with the selection [from, to]. */
export function mdFormat({ doc, from, to, kind }: MdFormatInput): MdEdit {
  const wrap = WRAPS[kind];
  if (wrap) return wrapped({ doc, from, to }, { marks: wrap, isLink: kind === 'link' });
  return prefixed({ doc, from, to }, PREFIXES[kind]!);
}

type Selection = { doc: string; from: number; to: number };

/** Wraps the selection; with nothing selected the caret lands between the marks. A link puts the
 *  caret in its (url). */
function wrapped({ doc, from, to }: Selection, { marks: [open, close], isLink }: { marks: [string, string]; isLink: boolean }): MdEdit {
  const text = doc.slice(from, to);
  const insert = open + text + close;
  const inner = from + open.length;
  if (isLink) {
    const url = from + insert.length - 1;
    return { from, to, insert, anchor: url, head: url };
  }
  return { from, to, insert, anchor: inner, head: inner + text.length };
}

/** Adds the prefix to every line the selection touches, or takes it off when they all have it. */
function prefixed({ doc, from, to }: Selection, prefix: string): MdEdit {
  const start = doc.lastIndexOf('\n', from - 1) + 1;
  const endBreak = doc.indexOf('\n', to);
  const end = endBreak === -1 ? doc.length : endBreak;
  const lines = doc.slice(start, end).split('\n');
  const allHave = lines.every((l) => l.startsWith(prefix));
  const changed = lines.map((l) => (allHave ? l.slice(prefix.length) : prefix + l));
  const insert = changed.join('\n');
  const head = start + insert.length;
  return { from: start, to: end, insert, anchor: head, head };
}
