/** One row of a list in Settings (statuses, labels, word kinds), as `ColorListEditor` shows it. */
export type ColorItem = {
  /** The id the API knows it by (a status value, a label id). */
  key: string;
  name: string;
  /** Its swatch; a list without colours (word kinds) leaves it out. */
  color?: string;
  /** Its value, shown in capitals; fixed once added. */
  meta: string;
  /** Drawn as a ring, not a filled dot (Not started). */
  ring?: boolean;
  /** Can't be dragged (the first and last status). */
  pinned?: boolean;
  /** A note after the name, like "always first". */
  note?: string;
  removable?: boolean;
};

/** What a colour list does with its rows. */
export type ColorListOps = {
  add(entry: { name: string; value: string; color: string }): Promise<unknown>;
  rename(change: { key: string; name: string }): Promise<unknown>;
  /** Lists without colours leave it out. */
  recolor?(change: { key: string; color: string }): Promise<unknown>;
  reorder(order: string[]): Promise<unknown>;
  remove(key: string): Promise<unknown>;
  /** Reloads the list after a change. */
  reload(): Promise<unknown>;
  /** What removing this row takes with it (it is in use), for its confirm; null when nothing. */
  removeWarning?(key: string): Promise<string | null>;
};
