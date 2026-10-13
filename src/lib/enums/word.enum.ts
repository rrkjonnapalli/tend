// The word kinds the UI guesses between (`tend_enums` WORD_KIND values). Tend comes with more
// (CONCEPT, TERM, OTHER), and users can add their own in Settings.
export const WORD_KIND = {
  word: 'WORD',
  idiom: 'IDIOM',
  phrase: 'PHRASE',
  abbr: 'ABBR',
} as const;
