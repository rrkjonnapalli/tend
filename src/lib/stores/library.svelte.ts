// The Library bar's state, shared with the area below it: the search text and the + presses.
class Library {
  /** The bar's search; each area filters by it. */
  query = $state('');
  /** Counts presses of the bar's +; the open area opens its own add when it changes. */
  adds = $state(0);

  requestAdd = () => {
    this.adds += 1;
  };

  clearQuery = () => {
    this.query = '';
  };
}

export const library = new Library();
