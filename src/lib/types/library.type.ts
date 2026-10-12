import type { LibraryArea } from '#lib/enums/library.enum.ts';

/** One area of the Library bar's switch. */
export type LibraryAreaItem = {
  area: LibraryArea;
  label: string;
  /** The search field's placeholder and accessible name in this area. */
  search: string;
};
