// Pure helpers for the ⌘K palette: word search and grouping of results.
import type { PaletteGroup, PaletteItem } from '#lib/types/palette.type.ts';

const words = (query: string) => query.toLowerCase().split(/\s+/).filter(Boolean);

/** Every word of the query appears somewhere in `text`, ignoring case. */
export function hasAllWords(text: string, query: string): boolean {
  const haystack = text.toLowerCase();
  return words(query).every((word) => haystack.includes(word));
}

/** Items grouped under their headings, groups in the order they first appear. */
export function groupItems(items: PaletteItem[]): PaletteGroup[] {
  const groups: PaletteGroup[] = [];
  items.forEach((item, index) => {
    let group = groups.find((g) => g.name === item.group);
    if (!group) {
      group = { name: item.group, items: [] };
      groups.push(group);
    }
    group.items.push({ item, index });
  });
  return groups;
}
