// The paper a snap is written on (`snaps.paper`). Same values as src-tauri/src/enums `Paper`.
export const PAPER = {
  scrap: 'SCRAP',
  slip: 'SLIP',
  lined: 'LINED',
  plain: 'PLAIN',
  sticky: 'STICKY',
} as const;

export type Paper = (typeof PAPER)[keyof typeof PAPER];
