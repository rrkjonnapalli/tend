import type { LibraryArea, SaveArea } from '#lib/enums/library.enum.ts';

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

/** One group of Home's search results. */
export type LibraryGroup = { area: SaveArea; label: string; hits: LibraryHit[] };
