/** One line in the ⌘K palette. */
export type PaletteItem = {
  key: string;
  group: string;
  icon: string;
  label: string;
  sub?: string;
  keys?: string;
  isNew?: boolean;
  run: () => void | Promise<void>;
};

/** Palette items under one heading, each with its index in the full list. */
export type PaletteGroup = { name: string; items: { item: PaletteItem; index: number }[] };
