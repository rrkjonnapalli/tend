import type { DictionarySource } from '#lib/enums/dictionary.enum.ts';
import type { Dictionary, NewWord, Word, WordKind, WordPatch } from '#lib/types/word.type.ts';
import { call } from './core.ts';

export const wordsAPI = {
  list: () => call<Word[]>('words_list'),
  create: (word: NewWord) => call<Word>('word_create', { args: { word } }),
  update: (input: { id: string; patch: WordPatch }) => call<Word>('word_update', { args: input }),
  remove: (id: string) => call<void>('word_delete', { args: { id } }),
  setTags: (input: { id: string; tags: string[] }) => call<string[]>('word_tags_set', { args: input }),
  /** Asks dictionaryapi.dev; `null` when it doesn't have the word (Rust gives up after 5 s). */
  lookup: (input: { title: string; source: DictionarySource }) => call<Dictionary | null>('word_lookup', { args: input }),
  kinds: () => call<WordKind[]>('word_kinds_list'),
  addKind: (input: { label: string; value: string }) => call<WordKind>('word_kind_add', { args: input }),
  renameKind: (input: { value: string; label: string }) => call<WordKind>('word_kind_rename', { args: input }),
  reorderKinds: (values: string[]) => call<void>('word_kind_reorder', { args: { values } }),
  removeKind: (value: string) => call<void>('word_kind_delete', { args: { value } }),
};
