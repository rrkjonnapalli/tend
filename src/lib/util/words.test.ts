import { describe, expect, it } from 'vitest';
import { WORD_KIND } from '#lib/enums/word.enum.ts';
import type { Word, WordKind } from '#lib/types/word.type.ts';
import { firstLine, groupWords, guessKind, savedTitle } from './words.ts';

const kind = (value: string, position: number): WordKind => ({ value, label: value.toLowerCase(), position, lookup: true, isDefault: true });
const word = (title: string, kindValue: string, meaning = ''): Word => ({
  id: title, title, kind: kindValue, dictionary: null, meaning, notes: '', tags: [], createdAt: 0, updatedAt: 0,
});

describe('words', () => {
  it('guesses the kind from the title', () => {
    expect(guessKind('ephemeral')).toBe(WORD_KIND.word);
    expect(guessKind('Petrichor')).toBe(WORD_KIND.word);
    expect(guessKind('NASA')).toBe(WORD_KIND.abbr);
    expect(guessKind('I')).toBe(WORD_KIND.word);
    expect(guessKind('bite the bullet')).toBe(WORD_KIND.phrase);
    expect(guessKind('4K')).toBe(WORD_KIND.abbr);
  });

  it('saves a WORD in lowercase and other kinds as typed', () => {
    expect(savedTitle({ title: '  Ephemeral ', kind: WORD_KIND.word })).toBe('ephemeral');
    expect(savedTitle({ title: ' IT ', kind: WORD_KIND.abbr })).toBe('IT');
  });

  it('takes the first line without markdown markers', () => {
    expect(firstLine('\n- **Here** and gone\nmore')).toBe('**Here** and gone');
    expect(firstLine('> ## quoted title')).toBe('quoted title');
    expect(firstLine('')).toBe('');
  });

  it('groups by kind in kind order, alphabetical, filtered', () => {
    const kinds = [kind(WORD_KIND.word, 0), kind(WORD_KIND.idiom, 1), kind(WORD_KIND.phrase, 2)];
    const words = [word('sanguine', 'WORD'), word('Bite the bullet', 'IDIOM'), word('ephemeral', 'WORD', 'brief'), word('gnarly', 'SLANG')];
    const groups = groupWords({ words, kinds, needle: '' });
    expect(groups.map((g) => [g.kind.value, g.words.map((w) => w.title)])).toEqual([
      ['WORD', ['ephemeral', 'sanguine']],
      ['IDIOM', ['Bite the bullet']],
      ['SLANG', ['gnarly']],
    ]);
    expect(groupWords({ words, kinds, needle: 'brief' }).map((g) => g.words[0].title)).toEqual(['ephemeral']);
  });
});
