import type { DictionarySource } from '#lib/enums/dictionary.enum.ts';
/** What the dictionary lookup returned, as a word stores it. */
export type Dictionary = {
  ipa: string | null;
  /** A recording of the word. */
  audio: string | null;
  partOfSpeech: string | null;
  senses: { definition: string; example: string | null }[];
  synonyms: string[];
  /** Missing on an entry fetched before antonyms were kept. */
  antonyms?: string[];
  /** Missing on an entry fetched before sources were recorded: it came from the first one. */
  source?: DictionarySource;
  /** The entry's own page at the source, when it gives one. */
  sourceUrl?: string | null;
  fetchedAt: number;
};

export type Word = {
  id: string;
  title: string;
  /** A word kind value (`WORD`, `IDIOM`, …). */
  kind: string;
  dictionary: Dictionary | null;
  meaning: string;
  notes: string;
  tags: string[];
  createdAt: number;
  updatedAt: number;
};

export type WordKind = {
  value: string;
  label: string;
  position: number;
  /** The dictionary is asked for words of this kind. */
  lookup: boolean;
  isDefault: boolean;
};

export type NewWord = Pick<Word, 'title' | 'kind' | 'dictionary' | 'meaning' | 'notes'>;

/** Fields an edit can change; `dictionary: null` clears the lookup. */
export type WordPatch = Partial<NewWord>;

/** One group of the Words list: a kind and its words, alphabetical. */
export type WordGroup = { kind: WordKind; words: Word[] };
