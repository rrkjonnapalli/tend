// The Library's areas; each is also its URL segment (/library/<area>) and the stored last area.
export const LIBRARY_AREA = {
  words: 'words',
  snaps: 'snaps',
  refs: 'references',
} as const;

export type LibraryArea = (typeof LIBRARY_AREA)[keyof typeof LIBRARY_AREA];
