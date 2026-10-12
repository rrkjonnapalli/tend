// The Library's areas; each is also its URL segment (/library/<area>) and the stored last area.
export const LIBRARY_AREA = {
  home: 'home',
  words: 'words',
  snaps: 'snaps',
  refs: 'references',
} as const;

export type LibraryArea = (typeof LIBRARY_AREA)[keyof typeof LIBRARY_AREA];

/** An area things are saved in: every one but Home. */
export type SaveArea = Exclude<LibraryArea, typeof LIBRARY_AREA.home>;
