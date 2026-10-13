// The records the UI attaches files to (`tend_links.source_type`). Same values as src-tauri/src/enums,
// which also has WORD, SNAP and REF.
export const SOURCE_TYPE = {
  task: 'TASK',
  note: 'NOTE',
  comment: 'COMMENT',
  person: 'PERSON',
} as const;

export type SourceType = (typeof SOURCE_TYPE)[keyof typeof SOURCE_TYPE];
