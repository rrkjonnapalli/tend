// The paper a snap is written on (`snaps.paper`). Same values as src-tauri/src/enums `Paper`.
export const PAPER = {
  scrap: 'SCRAP',
  slip: 'SLIP',
  lined: 'LINED',
  plain: 'PLAIN',
  sticky: 'STICKY',
} as const;

export type Paper = (typeof PAPER)[keyof typeof PAPER];

/** What sits on the board, and so what can be dragged. */
export const BOARD_KIND = {
  snap: 'snap',
  stack: 'stack',
} as const;

export type BoardKind = (typeof BOARD_KIND)[keyof typeof BOARD_KIND];

/** What can be dropped on: a snap (pair them), a stack (put it in), a step of the path (move up). */
export const DROP_KIND = {
  ...BOARD_KIND,
  crumb: 'crumb',
} as const;

export type DropKind = (typeof DROP_KIND)[keyof typeof DROP_KIND];

/** How big a paper is drawn: on the board, in the editor (fixed size), or small on Home. */
export const PAPER_SIZE = {
  board: 'board',
  editor: 'editor',
  mini: 'mini',
} as const;

export type PaperSize = (typeof PAPER_SIZE)[keyof typeof PAPER_SIZE];

/** The snap editor's pickers; one is open at a time. */
export const SNAP_PICKER = {
  stack: 'stack',
  color: 'color',
} as const;

export type SnapPicker = (typeof SNAP_PICKER)[keyof typeof SNAP_PICKER];
