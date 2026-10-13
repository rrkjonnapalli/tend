import type { LibraryArea, SaveArea } from '#lib/enums/library.enum.ts';
import type { Paper } from '#lib/enums/snap.enum.ts';

/** One area of the Library bar's switch. */
export type LibraryAreaItem = {
  area: LibraryArea;
  label: string;
  /** The search field's placeholder and accessible name in this area. */
  search: string;
};

/** One choice of the add box's Save-to switch. */
export type SaveTarget = { area: SaveArea; label: string };

/** Where the add box thinks typed text belongs, and why. */
export type AreaGuess = { area: SaveArea; why: string };

/** One snap's place on Home's mini board. */
export type MiniSpot = { left: number; top: number; tilt: number };

/** A word, snap or reference as Home lists it: two lines and where it opens. */
export type LibraryHit = {
  id: string;
  title: string;
  sub: string;
  href: string;
  /** Body type for the title (snaps, references); words keep the heading type. */
  plain?: boolean;
  /** Monospace second line (a link). */
  mono?: boolean;
};

/** One paper the snap editor's rotate button cycles through. */
export type PaperChoice = { paper: Paper; name: string };

/** One of Home's three columns: its head, and its list. */
export type HomeColumn = {
  area: SaveArea;
  title: string;
  /** What the column shows of the area: "recent", "latest", "new". */
  note: string;
  /** The link to the whole area: "All →". */
  more: string;
  /** The items listed, with the line shown when there are none; left out by the snaps column,
   *  which shows the mini board. */
  list?: { hits: LibraryHit[]; loaded: boolean; empty: string };
};

/** One group of Home's search results. */
export type LibraryGroup = { area: SaveArea; label: string; hits: LibraryHit[] };
