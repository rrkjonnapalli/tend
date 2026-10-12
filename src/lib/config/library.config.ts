// The Library's fixed lists: the bar's areas (with each one's search placeholder), the snap
// papers and the board's tilts.
import { LIBRARY_AREA } from '#lib/enums/library.enum.ts';
import { PAPER, type Paper } from '#lib/enums/snap.enum.ts';
import type { LibraryAreaItem } from '#lib/types/library.type.ts';

export const libraryAreas: LibraryAreaItem[] = [
  { area: LIBRARY_AREA.words, label: 'Words', search: 'Search words' },
  { area: LIBRARY_AREA.snaps, label: 'Snaps', search: 'Search snaps' },
  { area: LIBRARY_AREA.refs, label: 'References', search: 'Search references' },
];

/** References: how long typing must pause before a link's page title is fetched. */
export const titleFetchDelayMs = 650;

/** Snaps: the papers in the order the editor's rotate button cycles through them. */
export const papers: { paper: Paper; name: string }[] = [
  { paper: PAPER.scrap, name: 'scrap' },
  { paper: PAPER.slip, name: 'slip' },
  { paper: PAPER.lined, name: 'lined paper' },
  { paper: PAPER.plain, name: 'plain card' },
  { paper: PAPER.sticky, name: 'sticky note' },
];

/** A new sticky's colour (the Yellow swatch). */
export const stickyColor = '#d4b106';

/** The board's columns: at least this wide, this far apart, with the gap also at each side. */
export const boardColumn = { minWidth: 250, gap: 52 };

/** Each board item's small fixed tilt, by position, in degrees (never more than 1.8). */
export const boardTilts = [-1.6, 1.2, -0.6, 1.8, -1.2, 0.6, 1.4, -1.8];
