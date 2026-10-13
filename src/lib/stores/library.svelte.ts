// The Library bar's state, shared with the area below it: the search text, the + presses and
// the text Home's add box hands to an area.
import { untrack } from 'svelte';

class Library {
  /** The bar's search; each area filters by it. */
  query = $state('');
  /** Counts presses of the bar's +; the open area opens its own add when it changes. */
  adds = $state(0);

  /** Text from Home's add box, waiting for the area it was sent to. */
  #handed: string | null = null;

  handOff = (text: string) => {
    this.#handed = text;
  };

  /** The text handed over, once: the area opens its add with it filled in. */
  takeHanded = (): string | null => {
    const text = this.#handed;
    this.#handed = null;
    return text;
  };

  requestAdd = () => {
    this.adds += 1;
  };

  clearQuery = () => {
    this.query = '';
  };
}

export const library = new Library();

/** Runs `add` each time the bar's + is pressed while the calling page is open; presses from before
 *  it opened don't count. Call it once, in the page's script. */
export function onLibraryAdd(add: () => void) {
  let seen = library.adds;
  $effect(() => {
    const adds = library.adds;
    if (adds === seen) return;
    seen = adds;
    untrack(add);
  });
}
