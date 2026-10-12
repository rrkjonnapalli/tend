// The Library bar's areas, in switch order, with the search placeholder each one shows.
import { LIBRARY_AREA } from '#lib/enums/library.enum.ts';
import type { LibraryAreaItem } from '#lib/types/library.type.ts';

export const libraryAreas: LibraryAreaItem[] = [
  { area: LIBRARY_AREA.words, label: 'Words', search: 'Search words' },
  { area: LIBRARY_AREA.snaps, label: 'Snaps', search: 'Search snaps' },
  { area: LIBRARY_AREA.refs, label: 'References', search: 'Search references' },
];

/** References: how long typing must pause before a link's page title is fetched. */
export const titleFetchDelayMs = 650;
