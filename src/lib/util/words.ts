// Words: the kind a title suggests, the title as saved (same rule as Rust `words::saved_title`)
// and the list grouped by kind.
import { WORD_KIND } from '#lib/enums/word.enum.ts';
import type { Word, WordGroup, WordKind } from '#lib/types/word.type.ts';
import { textMatches } from './search.ts';

const LETTER = /\p{L}/u;

/** All capitals (`NASA`, `IT`), no spaces: an abbreviation; no spaces: a word; else a phrase. */
export function guessKind(title: string): string {
  const t = title.trim();
  if (/\s/.test(t)) return WORD_KIND.phrase;
  const allCapitals = LETTER.test(t) && t === t.toUpperCase() && t.length > 1;
  return allCapitals ? WORD_KIND.abbr : WORD_KIND.word;
}

/** Trimmed, and lowercase for a `WORD`: what the title is saved as. */
export function savedTitle({ title, kind }: { title: string; kind: string }): string {
  const t = title.trim();
  return kind === WORD_KIND.word ? t.toLowerCase() : t;
}

/** The first line of some markdown, without its leading markers, for one-line previews. */
export function firstLine(markdown: string): string {
  const line = markdown.split('\n').find((l) => l.trim() !== '') ?? '';
  return line.replace(/^\s*([-*>#]+\s+)+/, '').trim();
}

/** The words matching `needle` (title or meaning), grouped by kind in kind order; empty kinds
 *  are left out. Words whose kind isn't listed go to the end under their own value. */
export function groupWords({ words, kinds, needle }: { words: Word[]; kinds: WordKind[]; needle: string }): WordGroup[] {
  const shown = words
    .filter((w) => textMatches([w.title, w.meaning], needle))
    .sort((a, b) => a.title.localeCompare(b.title, undefined, { sensitivity: 'base' }));
  const known = new Set(kinds.map((k) => k.value));
  const strays = [...new Set(shown.map((w) => w.kind).filter((k) => !known.has(k)))];
  const all = [...kinds, ...strays.map((value) => ({ value, label: value.toLowerCase(), position: 0, lookup: false, isDefault: false }))];
  const groups = all.map((kind) => ({ kind, words: shown.filter((w) => w.kind === kind.value) }));
  return groups.filter((g) => g.words.length > 0);
}
