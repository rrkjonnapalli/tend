// Every word and word kind, for the Words list and the kind pickers.
import { wordsAPI } from '#lib/api/words.ts';
import type { Word, WordKind } from '#lib/types/word.type.ts';

class WordsStore {
  words = $state<Word[]>([]);
  kinds = $state<WordKind[]>([]);
  /** False until the first load, so an empty list isn't shown as "Nothing here yet". */
  loaded = $state(false);

  load = async () => {
    const [words, kinds] = await Promise.all([wordsAPI.list(), wordsAPI.kinds()]);
    this.words = words;
    this.kinds = kinds;
    this.loaded = true;
  };

  byId = (id: string | null) => (id ? this.words.find((w) => w.id === id) : undefined);

  /** Another word with exactly this title (`except`: the one being edited). */
  withTitle = (title: string, except?: string) => this.words.find((w) => w.title === title && w.id !== except);

  kindOf = (value: string) => this.kinds.find((k) => k.value === value);
}

export const words = new WordsStore();
