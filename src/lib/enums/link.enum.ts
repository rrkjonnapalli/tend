// The record a link starts from (`tend_links.source_type`). Same values as src-tauri/src/enums.
export const SOURCE_TYPE = {
  task: 'TASK',
  note: 'NOTE',
  comment: 'COMMENT',
  person: 'PERSON',
} as const;

export type SourceType = (typeof SOURCE_TYPE)[keyof typeof SOURCE_TYPE];
