// The dictionaries a word can be looked up in. Same values as Rust `words::dictionary::Source`.
export const DICTIONARY_SOURCE = {
  dictionaryapiDev: 'DICTIONARYAPI_DEV',
  freedictionaryapiCom: 'FREEDICTIONARYAPI_COM',
} as const;

export type DictionarySource = (typeof DICTIONARY_SOURCE)[keyof typeof DICTIONARY_SOURCE];
