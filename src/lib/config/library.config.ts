// The Library's fixed lists: the bar's areas (with each one's search placeholder), what Home
// shows, the snap papers and the board's tilts.
import { LIBRARY_AREA } from '#lib/enums/library.enum.ts';
import { PAPER } from '#lib/enums/snap.enum.ts';
import type { LibraryAreaItem, MiniSpot, PaperChoice, SaveTarget } from '#lib/types/library.type.ts';

export const libraryAreas: LibraryAreaItem[] = [
  { area: LIBRARY_AREA.home, label: 'Home', search: 'Search the library' },
  { area: LIBRARY_AREA.words, label: 'Words', search: 'Search words' },
  { area: LIBRARY_AREA.snaps, label: 'Snaps', search: 'Search snaps' },
  { area: LIBRARY_AREA.refs, label: 'References', search: 'Search references' },
];

/** Home's add box: where a thing can be saved, as its Save-to switch names each. */
export const saveTargets: SaveTarget[] = [
  { area: LIBRARY_AREA.words, label: 'word' },
  { area: LIBRARY_AREA.snaps, label: 'snap' },
  { area: LIBRARY_AREA.refs, label: 'reference' },
];

/** Home: how many of each it shows. */
export const homeCounts = { words: 5, snaps: 4, refs: 5 };

/** Home's mini board: where each of its snaps is pinned (px) and its tilt (degrees). */
export const miniSpots: MiniSpot[] = [
  { left: 22, top: 24, tilt: -2 },
  { left: 192, top: 30, tilt: 2.5 },
  { left: 56, top: 158, tilt: 1.5 },
  { left: 218, top: 170, tilt: -1.5 },
];

/** Home's add box: text this short, on one line, is taken for a word or phrase. */
export const wordGuessLimit = { words: 4, chars: 40 };

/** Words: where the dictionary entries come from, as the entry page names it. */
export const dictionarySource = 'dictionaryapi.dev';

/** Words: the part of speech that dictionary gives an idiom. */
export const idiomPartOfSpeech = 'idiom';

/** References: how long typing must pause before a link's page title is fetched. */
export const titleFetchDelayMs = 650;

/** Snaps: the papers in the order the editor's rotate button cycles through them. */
export const papers: PaperChoice[] = [
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
